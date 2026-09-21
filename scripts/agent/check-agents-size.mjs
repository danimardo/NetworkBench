#!/usr/bin/env node
/**
 * Garantía: AGENTS.md debe mantenerse por debajo de 200 líneas.
 *
 * Motivo: es el punto de entrada portable. Si crece, deja de ser lo permanente y
 * transversal y se convierte en documentación duplicada que ninguna herramienta lee
 * entera. El contenido de ámbito acotado va a .agents/rules/.
 *
 * Al fallar: mueve contenido a .agents/rules/, NO recortes reglas.
 *
 * Uso: node scripts/agent/check-agents-size.mjs
 */
import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { ROOT, ok, fail, report } from "./_lib.mjs";

const LIMITE = 200;
const AVISO = 170;
const objetivo = join(ROOT, "AGENTS.md");

export function check() {
  const r = [];
  if (!existsSync(objetivo)) {
    return [fail("AGENTS.md no existe en la raíz del repositorio")];
  }
  const lineas = readFileSync(objetivo, "utf8").split(/\r?\n/).length;
  if (lineas >= LIMITE) {
    r.push(fail(`AGENTS.md tiene ${lineas} líneas; el límite es ${LIMITE}. Mueve contenido a .agents/rules/`));
  } else if (lineas >= AVISO) {
    r.push(ok(`AGENTS.md tiene ${lineas} líneas (límite ${LIMITE}) — cerca del límite`));
  } else {
    r.push(ok(`AGENTS.md tiene ${lineas} líneas (límite ${LIMITE})`));
  }
  return r;
}

if (import.meta.filename === process.argv[1]) {
  process.exit(report("Tamaño de AGENTS.md", check()));
}
