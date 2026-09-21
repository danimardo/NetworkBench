#!/usr/bin/env node
/**
 * Hook PreToolUse de Codex: deniega escrituras en rutas protegidas de NetworkBench.
 *
 * ESTADO: INFERIDO. El formato de hooks.json y el esquema de salida
 * (hookSpecificOutput.permissionDecision) proceden de cadenas del binario de Codex
 * 0.155.1, NO de documentación ejecutada. Ver .codex/README.md.
 *
 * Por eso este hook FALLA EN ABIERTO: si no entiende lo que recibe, permite y avisa.
 * Un control secundario que rompe la sesión por no entender su propia entrada es peor
 * que no tenerlo. La garantía que SÍ falla en cerrado es el hook pre-commit de Git
 * (scripts/git-hooks/pre-commit), que no depende de ningún agente.
 *
 * Contrato asumido:
 *   entrada  : JSON por stdin, con el nombre de la herramienta y sus argumentos
 *   salida   : JSON por stdout con hookSpecificOutput.permissionDecision
 *              ("allow" | "deny" | "ask") y permissionDecisionReason
 */
import { readFileSync } from "node:fs";

const PROTEGIDAS = [
  [/(^|[\/])\.specify[\/]/i, "gestionado por el CLI de SpecKit (manifiestos SHA-256)"],
  [/(^|[\/])\.agents[\/]skills[\/]speckit-/i, "skill instalada por SpecKit"],
  [/(^|[\/])Design[\/]/i, "entrega del disenador"],
  [/(^|[\/])Historias\.md$/i, "requisitos de producto"],
  [/(^|[\/])Especificacion\.md$/i, "documento histórico"],
  [/(^|[\/])AUDITORIA_DISENO_V3\.md$/i, "documento histórico"],
];

/** Herramientas que modifican ficheros. Se comparan en minúsculas y por inclusión. */
const ESCRITURA = ["write", "edit", "patch", "apply", "create", "delete", "move", "shell", "bash"];

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

let entrada = "";
try {
  entrada = readFileSync(0, "utf8");
} catch {
  responder("allow", "guard: no se pudo leer stdin; se permite y se avisa");
}

let datos;
try {
  datos = JSON.parse(entrada);
} catch {
  responder("allow", "guard: entrada no es JSON; se permite y se avisa");
}

// Nombre de la herramienta, bajo cualquiera de las claves plausibles.
const herramienta = String(
  datos.tool_name ?? datos.toolName ?? datos.tool ?? datos.name ?? ""
).toLowerCase();

if (herramienta && !ESCRITURA.some((t) => herramienta.includes(t))) {
  responder("allow", "guard: herramienta de solo lectura");
}

// Todas las cadenas del payload, para no depender de dónde venga la ruta.
const cadenas = [];
const recoger = (v, profundidad = 0) => {
  if (profundidad > 6) return;
  if (typeof v === "string") cadenas.push(v);
  else if (Array.isArray(v)) v.forEach((x) => recoger(x, profundidad + 1));
  else if (v && typeof v === "object") Object.values(v).forEach((x) => recoger(x, profundidad + 1));
};
recoger(datos.tool_input ?? datos.toolInput ?? datos.input ?? datos.arguments ?? datos);

for (const cadena of cadenas) {
  for (const [patron, motivo] of PROTEGIDAS) {
    if (patron.test(cadena)) {
      responder(
        "deny",
        `Ruta protegida de NetworkBench — ${motivo}. Modificarla requiere autorización ` +
          `específica del propietario: informa, propón y espera. ` +
          `Ver .agents/rules/universal/fuentes-de-verdad.md`
      );
    }
  }
}

responder("allow", "guard: sin rutas protegidas en la operación");
