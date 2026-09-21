# Configuración de Codex para NetworkBench

## Por qué existe este directorio

La configuración global de esta máquina (`~/.codex/config.toml`) fija
`approval_policy = "never"` y `sandbox_mode = "danger-full-access"` para **todos** los
proyectos. Sin este directorio, en Codex las instrucciones escritas serían la única
barrera: no habría ni aprobación ni sandbox.

Codex 0.155.1 aplica la configuración del proyecto **por delante** de la del usuario, así
que el repositorio puede endurecer lo que la máquina afloja. Es exactamente lo contrario
de ampliar permisos para desbloquear al agente.

## Contenido

| Fichero | Qué hace |
|---|---|
| `config.toml` | `approval_policy = on-request` y `sandbox_mode = workspace-write` |
| `hooks.toml` | Hook `PreToolUse` que deniega escrituras en rutas protegidas |
| `hooks/check-protected-paths.mjs` | El comprobador que invoca el hook |

## Confianza de los hooks

Codex pide confirmación la primera vez que encuentra hooks en un repositorio
(`HookTrustStatus` en el binario). Es deliberado: código versionado no debe ejecutarse
sin que alguien lo acepte. Acéptalos una vez y quedan activos.

`--dangerously-bypass-hook-trust` existe; no lo uses.

## Estado de verificación

**DOCUMENTADO, no ejecutado.** La precedencia de la configuración de proyecto y el formato
de los hooks proceden de las cadenas del binario instalado, no de una sesión real de Codex.

Cómo comprobarlo: abre Codex en este repositorio y pídele que escriba en `Historias.md`.
Debe pedir aprobación y el hook debe denegarlo. Si no ocurre, **la única barrera real es
el hook `pre-commit` de Git** (`scripts/git-hooks/`): dilo y actualiza
`.agents/meta/agent-capabilities.yaml`.

## Capa independiente

Pase lo que pase aquí, `scripts/git-hooks/pre-commit` sigue actuando: es independiente de
la herramienta y del agente. Instálalo con `git config core.hooksPath scripts/git-hooks`.
