#!/usr/bin/env node
/**
 * Verificación del sistema de instrucciones para agentes de NetworkBench.
 *
 * Orquesta las garantías deterministas. NO verifica el producto: hoy no existe
 * aplicación, ni build, ni tests. Ver .agents/rules/proyecto/estado.md.
 *
 * ATENCIÓN: no hay CI ni hooks en este repositorio. Nada ejecuta esto por ti.
 * Invócalo a mano tras tocar AGENTS.md, .agents/, CLAUDE.md o .claude/.
 *
 * Uso:  node scripts/agent/verify.mjs
 * Salida: 0 si todo pasa; 1 si alguna comprobación falla. Los avisos no fallan.
 */
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { ROOT, report, warn } from "./_lib.mjs";

import { check as checkSize } from "./check-agents-size.mjs";
import { check as checkAdapters } from "./check-adapters.mjs";
import { check as checkReferences } from "./check-references.mjs";
import { check as checkSpecHistory } from "./check-spec-history.mjs";
import { check as checkProtected } from "./check-protected-paths.mjs";

const COMPROBACIONES = [
  ["Tamaño de AGENTS.md", checkSize],
  ["Coherencia de adaptadores", checkAdapters],
  ["Referencias", checkReferences],
  ["Protección de documentos históricos", checkSpecHistory],
  ["Rutas protegidas (working tree)", () => checkProtected({ staged: false })],
];

/** El verificador de tokens de Design/ es un proceso aparte, ya existente. */
function checkTokens() {
  const script = join(ROOT, "Design", "scripts", "verify-tokens.mjs");
  if (!existsSync(script)) {
    return [warn("Design/scripts/verify-tokens.mjs no existe: comprobación omitida")];
  }
  try {
    const salida = execFileSync(process.execPath, [script], {
      cwd: join(ROOT, "Design"),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    return [{ estado: "ok", mensaje: salida.trim().split(/\r?\n/).pop() || "sin salida" }];
  } catch (e) {
    const detalle = `${e.stdout || ""}${e.stderr || ""}`
      .trim()
      .split(/\r?\n/)
      .slice(-3)
      .join(" | ");
    return [{ estado: "fallo", mensaje: `verify-tokens.mjs falló: ${detalle}` }];
  }
}

/** Evita que el guard vuelva a confundir contenido documental con rutas de destino. */
function checkGuardProtectedPaths() {
  const test = join(ROOT, "scripts", "agent", "guard-protected-paths.test.mjs");
  if (!existsSync(test)) {
    return [warn("scripts/agent/guard-protected-paths.test.mjs no existe: comprobación omitida")];
  }
  try {
    const salida = execFileSync(process.execPath, ["--test", test], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    const resumen = salida.match(/ℹ pass \d+/)?.[0] ?? "pruebas del guard correctas";
    return [{ estado: "ok", mensaje: resumen }];
  } catch (e) {
    const detalle = `${e.stdout || ""}${e.stderr || ""}`
      .trim()
      .split(/\r?\n/)
      .slice(-5)
      .join(" | ");
    return [{ estado: "fallo", mensaje: `pruebas del guard fallaron: ${detalle}` }];
  }
}

let codigo = 0;
const resumen = [];

for (const [titulo, fn] of [
  ...COMPROBACIONES,
  ["Guard de rutas protegidas", checkGuardProtectedPaths],
  ["Tokens de diseño", checkTokens],
]) {
  let resultados;
  try {
    resultados = fn();
  } catch (e) {
    resultados = [{ estado: "fallo", mensaje: `la comprobación lanzó un error: ${e.message}` }];
  }
  codigo |= report(titulo, resultados);
  const fallos = resultados.filter((r) => r.estado === "fallo").length;
  const avisos = resultados.filter((r) => r.estado === "aviso").length;
  resumen.push({ titulo, fallos, avisos });
}

console.log("\n── Resumen");
for (const s of resumen) {
  const etiqueta = s.fallos > 0 ? `${s.fallos} fallo(s)` : "correcto";
  console.log(`  ${etiqueta.padEnd(12)} ${s.titulo}${s.avisos ? ` (${s.avisos} aviso(s))` : ""}`);
}
console.log(
  codigo === 0
    ? "\nSistema de instrucciones coherente. Esto NO verifica el producto: para eso, pnpm verify."
    : "\nHay fallos. Corrige el sistema canónico, no los adaptadores.",
);

process.exit(codigo);
