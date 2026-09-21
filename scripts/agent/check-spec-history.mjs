#!/usr/bin/env node
/**
 * Garantía: no se reescribe ni se borra material normativo histórico.
 *
 * La regla "no borrar specs históricas" es una instrucción; esto es la comprobación.
 * Compara el contenido actual con el de HEAD para los documentos protegidos:
 *
 *   - Especificacion.md y AUDITORIA_DISENO_V3.md: históricos. Viven en HEAD y están
 *     borrados del working tree A PROPÓSITO (decisión del propietario, 2026-09-21).
 *     Si reaparecen con contenido distinto al de HEAD, alguien los reescribió.
 *   - specs/**: artefactos de SpecKit. No pueden desaparecer respecto a HEAD.
 *   - .specify/memory/constitution.md: requiere autorización específica para cambiar.
 *     Su modificación NO se bloquea aquí; se señala para que se declare.
 *
 * Uso: node scripts/agent/check-spec-history.mjs
 */
import { existsSync, readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join } from "node:path";
import { ROOT, ok, fail, warn, report } from "./_lib.mjs";

const HISTORICOS = ["Especificacion.md", "AUDITORIA_DISENO_V3.md"];
const CONSTITUCION = ".specify/memory/constitution.md";

const git = (...args) => {
  try {
    return { ok: true, salida: execFileSync("git", args, { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }) };
  } catch (e) {
    return { ok: false, error: e };
  }
};

/** Última versión conocida de un fichero, aunque ya no exista en HEAD. */
function versionHistorica(f) {
  const enHead = git("show", `HEAD:${f}`);
  if (enHead.ok) return enHead;
  // Último commit que tocó la ruta (el que la borró o renombró).
  const sha = git("rev-list", "-1", "HEAD", "--", f);
  if (!sha.ok || !sha.salida.trim()) return { ok: false };
  return git("show", `${sha.salida.trim()}~1:${f}`);
}

export function check() {
  const r = [];

  const head = git("rev-parse", "HEAD");
  if (!head.ok) return [warn("No es un repositorio Git utilizable: comprobación omitida")];

  // 1. Documentos históricos: si existen en el working tree, deben ser idénticos a su
  //    última versión conocida. Se busca en TODO el historial, no solo en HEAD: tras el
  //    renombrado Especificacion.md -> Historias.md el fichero ya no está en HEAD, y
  //    mirar solo HEAD haría desaparecer la protección en silencio.
  for (const f of HISTORICOS) {
    const enHead = versionHistorica(f);
    if (!enHead.ok) { r.push(warn(`${f} no aparece en el historial: nada que proteger`)); continue; }
    const ruta = join(ROOT, f);
    if (!existsSync(ruta)) {
      r.push(ok(`${f} conservado en el historial y ausente del working tree (intencionado)`));
      continue;
    }
    const actual = readFileSync(ruta, "utf8");
    if (actual !== enHead.salida) {
      r.push(fail(`${f} difiere de su última versión en el historial. Es un documento histórico: restaura e informa`));
    } else {
      r.push(ok(`${f} presente e idéntico a su versión histórica`));
    }
  }

  // 2. specs/**: ningún fichero versionado puede haber desaparecido.
  const enHeadSpecs = git("ls-tree", "-r", "--name-only", "HEAD", "specs/");
  const listaSpecs = enHeadSpecs.ok
    ? enHeadSpecs.salida.split(/\r?\n/).filter(Boolean)
    : [];
  if (listaSpecs.length === 0) {
    r.push(ok("specs/ no existe todavía en HEAD: ninguna spec que proteger"));
  } else {
    const borradas = listaSpecs.filter((f) => !existsSync(join(ROOT, f)));
    if (borradas.length > 0) {
      for (const f of borradas) r.push(fail(`Spec borrada respecto a HEAD: ${f}`));
    } else {
      r.push(ok(`${listaSpecs.length} ficheros bajo specs/ presentes`));
    }
  }

  // 3. Constitución: se señala cualquier divergencia con HEAD, no se bloquea.
  const constEnHead = git("show", `HEAD:${CONSTITUCION}`);
  const constRuta = join(ROOT, CONSTITUCION);
  if (!existsSync(constRuta)) {
    r.push(fail(`${CONSTITUCION} no existe: falta el marco normativo`));
  } else if (!constEnHead.ok) {
    r.push(warn(`${CONSTITUCION} existe pero no está versionado en HEAD: sin trazabilidad en Git`));
  } else if (readFileSync(constRuta, "utf8") !== constEnHead.salida) {
    r.push(warn(`${CONSTITUCION} difiere de HEAD: su modificación requiere autorización específica. Decláralo`));
  } else {
    r.push(ok(`${CONSTITUCION} idéntica a HEAD`));
  }

  return r;
}

if (import.meta.filename === process.argv[1]) {
  process.exit(report("Protección de documentos históricos", check()));
}
