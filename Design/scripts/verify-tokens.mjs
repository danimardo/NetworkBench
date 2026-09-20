#!/usr/bin/env node
/**
 * Verificación de CI: ningún componente puede tener un color, gradiente o
 * sombra coloreada como literal — SOLO var(--token) de tokens.css (§16.10:
 * "ningún valor literal en componentes, verificable por script en CI").
 *
 * Uso:
 *   node scripts/verify-tokens.mjs
 *
 * Añadir al workflow ci.yml junto al resto de verificaciones propias
 * (Especificacion.md §3.5, paso 2), antes de Prettier/ESLint/svelte-check.
 *
 * Qué comprueba:
 *   - Sin `#RRGGBB` / `#RGB` fuera de tokens.css.
 *   - Sin `rgb(`/`rgba(` fuera de tokens.css (el propio `color-mix()` y
 *     `currentColor` de StatusPill están permitidos porque no fijan un
 *     color, lo derivan del token que reciban).
 *   - Recorre todo `src/lib` (componentes + rutas cuando existan).
 *
 * Qué NO comprueba (a propósito, para no generar falsos positivos):
 *   - No mira `icons.ts` más allá de esta lista de exclusión — los trazos
 *     SVG no llevan color, usan `currentColor`.
 *   - No entra en `node_modules` ni `.svelte-kit`.
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, extname } from "node:path";

const ROOT = join(import.meta.dirname, "..");
const TARGET_DIRS = [join(ROOT, "src")];
const EXCLUDE_FILES = new Set(["tokens.css"]);
const CHECK_EXTENSIONS = new Set([".svelte", ".ts", ".css"]);

const HEX_COLOR = /#[0-9A-Fa-f]{3}(?:[0-9A-Fa-f]{3}){0,2}\b/g;
const RGB_FN = /\brgba?\(/g;

/** @type {{file: string, line: number, match: string}[]} */
const violations = [];

function walk(dir) {
  for (const entry of readdirSync(dir)) {
    if (entry === "node_modules" || entry === ".svelte-kit" || entry === "dist") continue;
    const full = join(dir, entry);
    const info = statSync(full);
    if (info.isDirectory()) {
      walk(full);
      continue;
    }
    if (EXCLUDE_FILES.has(entry)) continue;
    if (!CHECK_EXTENSIONS.has(extname(entry))) continue;
    checkFile(full);
  }
}

function checkFile(file) {
  const text = readFileSync(file, "utf8");
  const lines = text.split("\n");
  lines.forEach((line, i) => {
    for (const re of [HEX_COLOR, RGB_FN]) {
      re.lastIndex = 0;
      let m;
      while ((m = re.exec(line))) {
        violations.push({ file, line: i + 1, match: m[0] });
      }
    }
  });
}

for (const dir of TARGET_DIRS) walk(dir);

if (violations.length > 0) {
  console.error(`✗ ${violations.length} color literal(es) fuera de tokens.css:\n`);
  for (const v of violations) {
    console.error(`  ${v.file.replace(ROOT + "/", "")}:${v.line}  →  ${v.match}`);
  }
  console.error(
    "\nMueve estos valores a design-system/tokens.css (con su pareja claro/oscuro) y referencia el token con var(--…)."
  );
  process.exit(1);
}

console.log("✓ Sin colores literales fuera de tokens.css.");
