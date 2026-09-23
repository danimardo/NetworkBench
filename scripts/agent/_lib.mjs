/**
 * Utilidades compartidas por las comprobaciones de scripts/agent/.
 * Sin dependencias externas: Node ESM puro, portable en Windows.
 */
import { readdirSync, statSync } from "node:fs";
import { join, dirname, sep } from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

export const ok = (mensaje) => ({ estado: "ok", mensaje });
export const fail = (mensaje) => ({ estado: "fallo", mensaje });
export const warn = (mensaje) => ({ estado: "aviso", mensaje });

const ICONO = { ok: "  ok  ", fallo: " FALLO", aviso: " aviso" };

/** Imprime los resultados de una comprobación. Devuelve el código de salida. */
export function report(titulo, resultados) {
  console.log(`\n── ${titulo}`);
  for (const r of resultados) console.log(`${ICONO[r.estado]}  ${r.mensaje}`);
  return resultados.some((r) => r.estado === "fallo") ? 1 : 0;
}

/** Recorre un directorio y devuelve las rutas absolutas de los ficheros que pasen el filtro. */
export function walk(dir, filtro = () => true, acc = []) {
  let entradas;
  try {
    entradas = readdirSync(dir);
  } catch {
    return acc;
  }
  for (const e of entradas) {
    const p = join(dir, e);
    let st;
    try {
      st = statSync(p);
    } catch {
      continue;
    }
    if (st.isDirectory()) {
      if (e === ".git" || e === "node_modules" || e === ".logs") continue;
      walk(p, filtro, acc);
    } else if (filtro(p)) {
      acc.push(p);
    }
  }
  return acc;
}

/** Ruta relativa a la raíz, con separadores POSIX para que la salida sea estable. */
export const rel = (p) =>
  p
    .slice(ROOT.length + 1)
    .split(sep)
    .join("/");

/** Extrae un campo simple del frontmatter YAML de la cabecera de un fichero. */
export function frontmatterField(contenido, campo) {
  const m = /^---\r?\n([\s\S]*?)\r?\n---/.exec(contenido);
  if (!m) return null;
  const linea = m[1].split(/\r?\n/).find((l) => l.startsWith(`${campo}:`));
  return linea === undefined ? null : linea.slice(campo.length + 1).trim();
}
