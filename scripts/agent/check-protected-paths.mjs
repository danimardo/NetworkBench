#!/usr/bin/env node
/**
 * Garantía: no se llevan al índice cambios en rutas protegidas.
 *
 * A diferencia del resto de comprobaciones, esta NO depende de que un agente coopere:
 * la invoca el hook pre-commit de Git (scripts/git-hooks/pre-commit), que se ejecuta
 * pase lo que pase con la configuración de permisos de la herramienta de turno.
 *
 * Rutas protegidas y por qué:
 *   .specify/**                     gestionado por el CLI de SpecKit, con manifiestos SHA-256
 *   .agents/skills/speckit-...        instaladas por SpecKit, dentro de esos manifiestos
 *   Design/**                       entrega del disenador
 *   Especificacion.md               documento histórico
 *   AUDITORIA_DISENO_V3.md          documento histórico
 *
 * Cambiarlas no está prohibido para siempre: está prohibido hacerlo en silencio.
 * Con autorización del propietario:  git commit --no-verify
 *
 * Uso:  node scripts/agent/check-protected-paths.mjs [--staged]
 *   sin --staged  INFORMA de los cambios del working tree (avisos, no falla).
 *                 Un working tree con rutas protegidas tocadas puede ser trabajo en
 *                 curso autorizado; lo que no puede es entrar en un commit sin que
 *                 nadie lo vea.
 *   con --staged  COMPRUEBA el índice y FALLA (modo hook pre-commit). Aquí sí es
 *                 vinculante: es el punto en que el cambio se vuelve permanente.
 */
import { execFileSync } from "node:child_process";
import { ROOT, ok, fail, warn, report } from "./_lib.mjs";

const PROTEGIDAS = [
  { patron: /^\.specify\//, motivo: "gestionado por el CLI de SpecKit (manifiestos SHA-256)" },
  { patron: /^\.agents\/skills\/speckit-/, motivo: "skill instalada por SpecKit" },
  { patron: /^Design\//, motivo: "entrega del disenador" },
  { patron: /^Especificacion\.md$/, motivo: "documento histórico" },
  { patron: /^AUDITORIA_DISENO_V3\.md$/, motivo: "documento histórico" },
];

/** La constitución no se bloquea: se señala, porque su cambio debe declararse. */
const A_DECLARAR = /^\.specify\/memory\/constitution\.md$/;

const git = (...args) => {
  try {
    return execFileSync("git", args, {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
  } catch {
    return null;
  }
};

export function check({ staged = false } = {}) {
  const r = [];
  const salida = staged
    ? git("diff", "--cached", "--name-only", "--diff-filter=ACMRD")
    : git("diff", "--name-only", "--diff-filter=ACMRD");

  if (salida === null) return [warn("No es un repositorio Git utilizable: comprobación omitida")];

  const ficheros = salida.split(/\r?\n/).filter(Boolean);
  const ambito = staged ? "en el índice" : "en el working tree";

  if (ficheros.length === 0) return [ok(`Sin cambios ${ambito}`)];

  // Primer commit: todo entra por definición, no hay nada que proteger todavía.
  if (staged && git("rev-parse", "HEAD") === null) {
    return [warn("Primer commit del repositorio: comprobación de rutas protegidas omitida")];
  }

  let tocadas = 0;
  for (const f of ficheros) {
    if (A_DECLARAR.test(f)) {
      r.push(warn(`${f} modificado: requiere autorización específica. Decláralo en el mensaje de commit`));
      continue;
    }
    const p = PROTEGIDAS.find((x) => x.patron.test(f));
    if (p) {
      tocadas++;
      // Vinculante solo en el índice: ver la cabecera.
      r.push((staged ? fail : warn)(`${f} está protegido — ${p.motivo}`));
    }
  }

  if (tocadas > 0) {
    r.push(
      (staged ? fail : warn)(
        staged
          ? `${tocadas} ruta(s) protegida(s) en el índice. Con autorización del propietario y declarándolo: git commit --no-verify`
          : `${tocadas} ruta(s) protegida(s) modificada(s) en el working tree. No bloquea aquí; el hook pre-commit sí lo hará`
      )
    );
  } else {
    r.push(ok(`${ficheros.length} fichero(s) ${ambito}, ninguno protegido`));
  }
  return r;
}

if (import.meta.filename === process.argv[1]) {
  const staged = process.argv.includes("--staged");
  process.exit(report("Rutas protegidas", check({ staged })));
}
