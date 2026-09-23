#!/usr/bin/env node
/**
 * Garantía: el sistema de instrucciones no apunta a ficheros inexistentes.
 *
 * Qué se considera una referencia comprobable: una ruta, es decir algo que contiene
 * "/" y apunta a un fichero concreto. Un nombre de fichero suelto (`package.json`)
 * NO lo es: en este repositorio suele aparecer precisamente para decir que no existe.
 *
 * Ámbito: AGENTS.md, CLAUDE.md y .agents/** excepto .agents/skills/speckit-*, que
 * las genera el CLI de SpecKit y están llenas de marcadores de posición
 * (FEATURE_DIR/, .specify/extensions.yml) que se resuelven en tiempo de ejecución.
 *
 * Además inventaría —sin fallar— las referencias a Especificacion.md que viven en
 * Design/ y en la constitución. Esos ficheros NO se editan a propósito: la decisión
 * del propietario del 2026-09-21 es que Historias.md sustituye a Especificacion.md y
 * que la equivalencia se registra en .agents/rules/universal/fuentes-de-verdad.md, no
 * se propaga con ediciones. El inventario existe para que nadie las confunda con
 * enlaces rotos por descuido.
 *
 * Uso: node scripts/agent/check-references.mjs
 */
import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { ROOT, ok, fail, warn, report, walk, rel } from "./_lib.mjs";

const EXT = /\.(md|mjs|js|ts|json|ya?ml|css|html|svelte|toml|ps1|jsonl)$/i;
/** Marcadores de posición de SpecKit y del propio sistema: no son rutas reales. */
const PLACEHOLDER = /(FEATURE_DIR|SPECIFY_|<|\$|\{|\*)/;

/** Ficheros cuyas referencias deben resolver. */
function ficherosControlados() {
  const l = [];
  for (const f of ["AGENTS.md", "CLAUDE.md"]) {
    const p = join(ROOT, f);
    if (existsSync(p)) l.push(p);
  }
  l.push(
    ...walk(join(ROOT, ".agents"), (p) => /\.(md|ya?ml)$/i.test(p)).filter(
      (p) => !rel(p).startsWith(".agents/skills/speckit-"),
    ),
  );
  return l;
}

/** Rutas citadas entre comillas invertidas o tras una importación @ de Claude Code. */
function extraerRutas(texto) {
  const rutas = new Set();
  for (const m of texto.matchAll(/`([^`\n]+)`/g)) rutas.add(m[1].trim());
  for (const m of texto.matchAll(/^@(\S+)$/gm)) rutas.add(m[1].trim());
  return [...rutas].filter(
    (r) =>
      r.includes("/") &&
      EXT.test(r) &&
      !PLACEHOLDER.test(r) &&
      !/\s/.test(r) &&
      // Rutas fuera del repositorio (configuración global del usuario): no comprobables.
      !r.startsWith("~") &&
      !/^[A-Za-z]:/.test(r),
  );
}

export function check() {
  const r = [];
  let comprobadas = 0;

  for (const fichero of ficherosControlados()) {
    const texto = readFileSync(fichero, "utf8");
    for (const ruta of extraerRutas(texto)) {
      comprobadas++;
      // Resuelve desde la raíz del repositorio o desde el propio fichero.
      const candidatos = [join(ROOT, ruta), join(fichero, "..", ruta)];
      if (!candidatos.some(existsSync)) {
        r.push(fail(`${rel(fichero)} referencia una ruta inexistente: ${ruta}`));
      }
    }
  }

  // Inventario informativo sobre ficheros que no se editan.
  const noEditables = [
    ...walk(join(ROOT, "Design"), (p) => /\.(md|mjs|html)$/i.test(p)),
    join(ROOT, ".specify", "memory", "constitution.md"),
  ].filter(existsSync);

  let total = 0;
  const detalle = [];
  for (const f of noEditables) {
    const n = (readFileSync(f, "utf8").match(/Especificacion\.md/g) || []).length;
    if (n > 0) {
      total += n;
      detalle.push(`${rel(f)} (${n})`);
    }
  }
  if (total > 0) {
    r.push(
      warn(
        `${total} referencias a Especificacion.md en ${detalle.length} ficheros no editables — léanse como Historias.md`,
      ),
    );
    for (const d of detalle.sort()) r.push(warn(`    ${d}`));
  }

  if (!r.some((x) => x.estado === "fallo")) {
    r.push(ok(`${comprobadas} rutas resueltas en AGENTS.md, CLAUDE.md y .agents/`));
  }
  return r;
}

if (import.meta.filename === process.argv[1]) {
  process.exit(report("Referencias", check()));
}
