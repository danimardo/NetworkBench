#!/usr/bin/env node
/**
 * Hook PreToolUse de Codex: deniega escrituras en rutas protegidas de NetworkBench.
 *
 * ESTADO: INFERIDO. El formato de hooks.json y el esquema de salida
 * (hookSpecificOutput.permissionDecision) proceden de cadenas del binario de Codex
 * 0.155.1, NO de documentación ejecutada. Ver .codex/README.md.
 *
 * FALLA EN ABIERTO a propósito: si no entiende lo que recibe, permite y lo dice.
 * Un control secundario que rompe la sesión por no entender su propia entrada es peor
 * que no tenerlo. La garantía que SÍ falla en cerrado es el hook pre-commit de Git
 * (scripts/git-hooks/pre-commit), que no depende de ningún agente.
 *
 * Dos modos, porque una ruta mencionada no es una ruta escrita:
 *
 *   herramienta de fichero (write, edit, patch, create, delete, move):
 *       cualquier mención de una ruta protegida -> deny.
 *
 *   herramienta de shell (shell, bash, powershell, exec, run):
 *       deny solo si aparece una ruta protegida JUNTO A un indicio de escritura
 *       (redirección, tee, sed -i, rm, mv, cp, Set-Content, git restore...).
 *       Así `cat Historias.md` se permite y `echo x >> Historias.md` no.
 *
 *   resto: allow.
 *
 * Probado con 10 casos; ver .codex/README.md.
 */
import { readFileSync } from "node:fs";

/** Sin anclaje final: una ruta protegida cuenta aparezca donde aparezca. */
const PROTEGIDAS = [
  [/(^|[^\w.-])\.specify[\/]/i, "gestionado por el CLI de SpecKit (manifiestos SHA-256)"],
  [/(^|[^\w.-])\.agents[\/]skills[\/]speckit-/i, "skill instalada por SpecKit"],
  [/(^|[^\w.-])Design[\/]/i, "entrega del disenador"],
  [/(^|[^\w.-])Historias\.md(\W|$)/i, "requisitos de producto"],
  [/(^|[^\w.-])Especificacion\.md(\W|$)/i, "documento histórico"],
  [/(^|[^\w.-])AUDITORIA_DISENO_V3\.md(\W|$)/i, "documento histórico"],
];

const HERRAMIENTA_FICHERO = ["write", "edit", "patch", "create", "delete", "remove", "move", "rename"];
const HERRAMIENTA_SHELL = ["shell", "bash", "powershell", "pwsh", "exec", "run", "command", "terminal"];

/** Indicios de que un comando de shell escribe, y no solo lee. */
const ESCRIBE = [
  />>?\s/, /\|\s*tee\b/i,
  /\bsed\b[^|;]*-i\b/i, /\bperl\b[^|;]*-i\b/i,
  /\b(rm|mv|cp|ln|truncate|dd|touch|mkdir|chmod|chown|install)\b/i,
  /\b(Set-Content|Add-Content|Out-File|New-Item|Remove-Item|Move-Item|Copy-Item|Clear-Content)\b/i,
  /\bgit\s+(restore|checkout|apply|rm|mv|clean|reset)\b/i,
  /\bpatch\b/i, /\bapply_patch\b/i,
];

const responder = (decision, razon) => {
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

// Todas las cadenas del payload, para no depender de dónde venga la ruta.
const cadenas = [];
const recoger = (v, p = 0) => {
  if (p > 6) return;
  if (typeof v === "string") cadenas.push(v);
  else if (Array.isArray(v)) v.forEach((x) => recoger(x, p + 1));
  else if (v && typeof v === "object") Object.values(v).forEach((x) => recoger(x, p + 1));
};
recoger(datos.tool_input ?? datos.toolInput ?? datos.input ?? datos.arguments ?? datos);

const texto = cadenas.join("\n");
const golpe = PROTEGIDAS.find(([patron]) => patron.test(texto));

if (!golpe) responder("allow", "guard: sin rutas protegidas en la operación");

// Herramienta de fichero, o herramienta desconocida: cualquier mención basta.
if (!esShell) responder("deny", RAZON_DENY(golpe[1]));

// Shell: hace falta además un indicio de escritura, para no bloquear lecturas.
if (ESCRIBE.some((p) => p.test(texto))) {
  responder("deny", RAZON_DENY(golpe[1]));
}

responder("allow", "guard: ruta protegida mencionada, pero el comando parece de solo lectura");
