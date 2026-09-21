#!/usr/bin/env node
/**
 * Hook PreToolUse de Codex: deniega escrituras en rutas protegidas de NetworkBench.
 *
 * ESTADO: VERIFICADO en Codex CLI 0.155.1: escritura bloqueada y lectura permitida.
 * Declaración en .codex/hooks.json y confianza concedida mediante el diálogo normal.
 * Evidencia y límites: .codex/validacion-hooks-2026-09-21.md.
 *
 * FALLA EN ABIERTO a propósito: si no entiende lo que recibe, permite continuar.
 * Un control secundario que rompe la sesión por no entender su propia entrada es peor
 * que no tenerlo. La garantía que SÍ falla en cerrado es el hook pre-commit de Git
 * (scripts/git-hooks/pre-commit), que no depende de ningún agente.
 *
 * Dos modos, porque una ruta mencionada no es una ruta escrita:
 *
 *   herramienta de fichero (write, edit, create, delete, move):
 *       inspecciona los campos de ruta; fallback conservador al payload.
 *   apply_patch: inspecciona destinos Add/Update/Delete/Move, no el contenido.
 *
 *   herramienta de shell (shell, bash, powershell, exec, run):
 *       deny solo si aparece una ruta protegida JUNTO A un indicio de escritura
 *       (redirección, tee, sed -i, rm, mv, cp, Set-Content, git restore...).
 *       Así `cat Design/README.md` se permite y `echo x >> Design/README.md` no.
 *
 *   resto: allow.
 *
 * Los 14 casos previos validaron el criterio; la integración real está en el informe.
 */
import { readFileSync } from "node:fs";

/** Sin anclaje final: una ruta protegida cuenta aparezca donde aparezca. */
const PROTEGIDAS = [
  [/(^|[^\w.-])\.specify[\/]/i, "gestionado por el CLI de SpecKit (manifiestos SHA-256)"],
  [/(^|[^\w.-])\.agents[\/]skills[\/]speckit-/i, "skill instalada por SpecKit"],
  [/(^|[^\w.-])Design[\/]/i, "entrega del disenador"],
  [/(^|[^\w.-])Especificacion\.md(\W|$)/i, "documento histórico"],
  [/(^|[^\w.-])AUDITORIA_DISENO_V3\.md(\W|$)/i, "documento histórico"],
];

const HERRAMIENTA_FICHERO = ["write", "edit", "patch", "create", "delete", "remove", "move", "rename"];
const HERRAMIENTA_SHELL = ["shell", "bash", "powershell", "pwsh", "exec", "run", "command", "terminal"];

/**
 * Indicios de que un comando de shell escribe, y no solo lee.
 *
 * La redirección exige que el ">" no venga precedido de "-", "=" ni ">", para no
 * confundir una flecha de prosa ("PreToolUse -> guard") con una redirección real.
 * Ese falso positivo bloqueó una edición legítima de documentación el 2026-09-21.
 */
const ESCRIBE = [
  /(^|[^-=>])>>?\s/, /\|\s*tee\b/i,
  /\bsed\b[^|;]*-i\b/i, /\bperl\b[^|;]*-i\b/i,
  /\b(rm|mv|cp|ln|truncate|dd|touch|mkdir|chmod|chown|install)\b/i,
  /\b(Set-Content|Add-Content|Out-File|New-Item|Remove-Item|Move-Item|Copy-Item|Clear-Content)\b/i,
  /\bgit\s+(restore|checkout|apply|rm|mv|clean|reset)\b/i,
  /\bpatch\b/i, /\bapply_patch\b/i,
];

const responder = (decision, razon) => {
  // Codex CLI 0.155.1 rechaza permissionDecision:allow sin updatedInput.
  // Una respuesta vacía deja continuar sin alterar la política de aprobación.
  // Claude Code también continúa cuando no se emite una decisión de bloqueo.
  if (decision === "allow") {
    process.stdout.write("{}");
    process.exit(0);
  }
  process.stdout.write(
    JSON.stringify({
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: decision,
        permissionDecisionReason: razon,
      },
    })
  );
  process.exit(0);
};

const RAZON_DENY = (motivo) =>
  `Ruta protegida de NetworkBench — ${motivo}. Modificarla requiere autorización ` +
  `específica del propietario: informa, propón y espera. No la edites por otra vía. ` +
  `Ver .agents/rules/universal/fuentes-de-verdad.md`;

let datos;
try {
  datos = JSON.parse(readFileSync(0, "utf8"));
} catch {
  responder("allow", "guard: entrada ilegible o no-JSON; se permite y se avisa");
}

const herramienta = String(
  datos.tool_name ?? datos.toolName ?? datos.tool ?? datos.name ?? ""
).toLowerCase();

const esFichero = HERRAMIENTA_FICHERO.some((t) => herramienta.includes(t));
const esShell = HERRAMIENTA_SHELL.some((t) => herramienta.includes(t));

if (herramienta && !esFichero && !esShell) {
  responder("allow", "guard: herramienta que no escribe ficheros");
}

const args = datos.tool_input ?? datos.toolInput ?? datos.input ?? datos.arguments ?? datos;

/** Aplana a texto todas las cadenas de un valor. */
const aplanar = (v, p = 0, acc = []) => {
  if (p > 6) return acc;
  if (typeof v === "string") acc.push(v);
  else if (Array.isArray(v)) v.forEach((x) => aplanar(x, p + 1, acc));
  else if (v && typeof v === "object") Object.values(v).forEach((x) => aplanar(x, p + 1, acc));
  return acc;
};

/**
 * Qué parte del payload se inspecciona. Importa mucho: escribir documentación que
 * MENCIONA una ruta protegida es legítimo, escribir EN ella no lo es.
 *
 *   herramienta de fichero -> los campos de ruta. El contenido que se escribe queda
 *       fuera: si no, editar este mismo fichero sería imposible. Si no hay ningún
 *       campo de ruta reconocible (p. ej. un parche que la lleva dentro), se cae a
 *       todo el payload, porque ahí la ruta sí va en el cuerpo.
 *   herramienta de shell -> solo el comando. La descripción es prosa humana.
 */
const CAMPOS_RUTA = ["file_path", "filePath", "path", "notebook_path", "notebookPath", "paths", "file", "files"];
const CAMPOS_COMANDO = ["command", "cmd", "script", "argv"];

/**
 * apply_patch transporta las rutas en cabeceras, no en campos JSON de fichero.
 * Las líneas de contenido llevan prefijo +, - o espacio y no son destinos.
 * Un formato desconocido conserva la inspección anterior de todo el payload.
 */
const rutasDelParche = (entrada) => {
  const textos = aplanar(entrada);
  if (textos.length !== 1) return null;
  const lineas = textos[0].trim().split(/\r?\n/);
  if (lineas[0] !== "*** Begin Patch" || lineas.at(-1) !== "*** End Patch") return null;
  const rutas = [];
  for (const linea of lineas.slice(1, -1)) {
    const cabecera = /^\*\*\* (?:Add File|Update File|Delete File|Move to): (.+)$/.exec(linea);
    if (cabecera) rutas.push(cabecera[1].replaceAll("\\", "/"));
    else if (linea.startsWith("*** ") && linea !== "*** End of File") return null;
  }
  return rutas.length > 0 ? rutas : null;
};

const esApplyPatch = /(?:^|[._/])apply_patch$/.test(herramienta);
const rutasParche = esApplyPatch ? rutasDelParche(args) : null;

let inspeccionado;
if (rutasParche !== null) {
  inspeccionado = rutasParche;
} else if (esShell) {
  const cmd = CAMPOS_COMANDO.filter((k) => args && args[k] !== undefined).map((k) => args[k]);
  inspeccionado = cmd.length > 0 ? aplanar(cmd) : aplanar(args);
} else {
  const rutas = CAMPOS_RUTA.filter((k) => args && args[k] !== undefined).map((k) => args[k]);
  inspeccionado = rutas.length > 0 ? aplanar(rutas) : aplanar(args);
}

const texto = inspeccionado.join("\n");
const golpe = PROTEGIDAS.find(([patron]) => patron.test(texto));

if (!golpe) responder("allow", "guard: sin rutas protegidas en la operación");

// Herramienta de fichero, o herramienta desconocida: cualquier mención basta.
if (!esShell) responder("deny", RAZON_DENY(golpe[1]));

// Shell: hace falta además un indicio de escritura, para no bloquear lecturas.
if (ESCRIBE.some((p) => p.test(texto))) {
  responder("deny", RAZON_DENY(golpe[1]));
}

responder("allow", "guard: ruta protegida mencionada, pero el comando parece de solo lectura");
