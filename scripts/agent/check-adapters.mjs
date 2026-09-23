#!/usr/bin/env node
/**
 * Garantía: los adaptadores no divergen del sistema canónico.
 *
 * Comprueba, para las skills speckit-*:
 *   1. Toda skill canónica en .agents/skills/ tiene su stub en .claude/skills/.
 *   2. Todo stub corresponde a una skill canónica existente (sin huérfanos).
 *   3. El frontmatter name y description del stub coincide LITERALMENTE con el canónico.
 *      Es la única duplicación inevitable del sistema: el formato de Claude Code exige
 *      frontmatter propio. Ver .agents/README.md.
 *   4. El stub apunta explícitamente a su canónico y no copia su cuerpo.
 *
 * Al fallar: regenera el stub desde el canónico. NUNCA edites un stub para cambiar
 * un procedimiento: cambia el canónico.
 *
 * Uso: node scripts/agent/check-adapters.mjs
 */
import { readFileSync, existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { ROOT, ok, fail, report, frontmatterField } from "./_lib.mjs";

const CANONICO = join(ROOT, ".agents", "skills");
const ADAPTADO = join(ROOT, ".claude", "skills");
const PREFIJO = "speckit-";
/** Un stub es un puntero, no una copia. Por encima de esto, algo se copió. */
const MAX_LINEAS_STUB = 40;

const listar = (dir) => {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true })
    .filter((d) => d.isDirectory() && d.name.startsWith(PREFIJO))
    .map((d) => d.name)
    .sort();
};

export function check() {
  const r = [];
  const canon = listar(CANONICO);
  const stubs = listar(ADAPTADO);

  if (canon.length === 0) {
    return [fail(".agents/skills/ no contiene ninguna skill speckit-*: falta el sistema canónico")];
  }

  for (const n of canon) {
    if (!stubs.includes(n))
      r.push(fail(`Falta el stub .claude/skills/${n}/SKILL.md para la skill canónica ${n}`));
  }
  for (const n of stubs) {
    if (!canon.includes(n))
      r.push(fail(`Stub huérfano .claude/skills/${n}/ sin skill canónica correspondiente`));
  }

  for (const n of canon.filter((x) => stubs.includes(x))) {
    const rutaCanon = join(CANONICO, n, "SKILL.md");
    const rutaStub = join(ADAPTADO, n, "SKILL.md");
    if (!existsSync(rutaCanon)) {
      r.push(fail(`.agents/skills/${n}/ no contiene SKILL.md`));
      continue;
    }
    if (!existsSync(rutaStub)) {
      r.push(fail(`.claude/skills/${n}/ no contiene SKILL.md`));
      continue;
    }

    const c = readFileSync(rutaCanon, "utf8");
    const s = readFileSync(rutaStub, "utf8");

    for (const campo of ["name", "description"]) {
      const vc = frontmatterField(c, campo);
      const vs = frontmatterField(s, campo);
      if (vc === null) {
        r.push(fail(`${n}: el canónico no declara "${campo}" en su frontmatter`));
        continue;
      }
      if (vs === null) {
        r.push(fail(`${n}: el stub no declara "${campo}" en su frontmatter`));
        continue;
      }
      if (vc !== vs) r.push(fail(`${n}: "${campo}" diverge entre canónico y stub`));
    }

    if (!s.includes(`.agents/skills/${n}/SKILL.md`)) {
      r.push(fail(`${n}: el stub no remite a su canónico .agents/skills/${n}/SKILL.md`));
    }
    const lineas = s.split(/\r?\n/).length;
    if (lineas > MAX_LINEAS_STUB) {
      r.push(
        fail(
          `${n}: el stub tiene ${lineas} líneas (máximo ${MAX_LINEAS_STUB}); parece copiar el canónico en vez de apuntarlo`,
        ),
      );
    }
  }

  if (!r.some((x) => x.estado === "fallo")) {
    r.push(ok(`${canon.length} skills speckit-* canónicas con stub coherente en .claude/skills/`));
  }
  return r;
}

if (import.meta.filename === process.argv[1]) {
  process.exit(report("Coherencia de adaptadores", check()));
}
