# Git

**Ámbito:** universal. **Estado:** VERIFICADO (`git status`, `git remote -v`).

## Contexto del repositorio

- Raíz: `F:/Apps/NetBench`. Rama por defecto: `main`. Un solo worktree, sin submódulos.
- Remoto: `origin` → `https://github.com/danimardo/NetworkBench.git`.
- Un único commit: `0bb03cb`.
- **`Historias.md`, `.specify/` y `.agents/` están sin versionar.** Una pérdida del working
  tree se llevaría la constitución y los requisitos. Es el riesgo abierto más grave del
  repositorio y solo el propietario puede cerrarlo autorizando el commit.
- `Especificacion.md` y `AUDITORIA_DISENO_V3.md` están borrados del working tree y
  presentes en `HEAD`. Es intencionado: ver `fuentes-de-verdad.md`.

## Prohibido sin autorización explícita y específica

No basta con que una tarea «necesite» el paso. Requieren permiso aparte, cada vez:

commit · amend · push · pull · fetch con escritura de refs · rebase · merge · cherry-pick ·
reset · revert · crear rama · cambiar de rama · crear o mover tags · stash · crear PR ·
aprobar PR · fusionar PR · `git clean` · `git restore` · `git checkout --` de ficheros.

Aprobar una propuesta de implementación **no** autoriza ninguna de estas operaciones.

## Prohibido siempre

- Revertir, limpiar o descartar cambios existentes que no hayas creado tú en esta sesión.
  El working tree sucio es estado de trabajo del propietario, no ruido.
- Restaurar `Especificacion.md` o `AUDITORIA_DISENO_V3.md` sin que se pida.
- Reescribir el historial.
- `--no-verify` o saltarse hooks.

## Permitido

Operaciones de solo lectura: `status`, `log`, `diff`, `show`, `ls-tree`, `rev-parse`,
`branch --show-current`, `worktree list`, `remote -v`.

## Atribución

Cuando el propietario autorice un commit, el mensaje va en español y termina con la línea
de coautoría que indique la herramienta en uso.
