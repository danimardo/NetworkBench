# Configuración de Codex para NetworkBench

## Estado vigente — verificado el 2026-09-21

El hook funciona en sesiones nuevas de **Codex CLI 0.155.1**. La declaración correcta es
`.codex/hooks.json` y requiere revisar y confiar en el guard mediante `/hooks`.
La escritura ensayada se bloqueó y la lectura terminó con `PreToolUse Completed` y código 0.
El guard devuelve `{}` para continuar: esta versión rechazaba la respuesta `allow` sola.
Evidencia, esquema y límites en `.codex/validacion-hooks-2026-09-21.md`.

La candidata anidada y su README se eliminaron. No se cambió la configuración del
sandbox. Los comentarios antiguos de `config.toml` sobre precedencia y aprobaciones no
son una garantía: las sesiones de prueba mostraron `approval: never`.

## Antecedentes conservados

El contenido siguiente registra el diseño y los ensayos previos al diagnóstico anterior;
sus estados pendientes y afirmaciones sobre precedencia no describen el resultado actual.

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
| `config.toml` | `approval_policy = on-request`. **No fija `sandbox_mode`**: ver abajo |
| `hooks.json` | Registra el hook `PreToolUse` (ruta corregida) |
| `hooks.json` → `scripts/agent/guard-protected-paths.mjs` | Deniega escrituras en rutas protegidas. **El guard es compartido con Claude Code**: un solo fichero, dos consumidores |

## Por qué no se fija `sandbox_mode`

Se intentó `sandbox_mode = "workspace-write"` el 2026-09-21 y **dejó Codex inservible en
este repositorio**: fallaba al ejecutar cualquier comando y ni siquiera podía leer ficheros.

Causa, VERIFICADA en las cadenas del binario: el sandbox de Windows exige un paso previo
con elevación,

```
codex sandbox setup --elevated --user <usuario> --codex-home <ruta>
```

y además no admite `command/exec/write`, `terminate`, `resize`, streaming ni
`outputBytesCap` personalizado. Sin ese setup, cualquier modo distinto de
`danger-full-access` rompe la sesión.

Se retiró a propósito. Forzarlo desde el repositorio exigiría elevación y cambiaría estado
global de la máquina: eso lo decide el propietario, no un fichero versionado. Si algún día
se ejecuta ese setup, puede añadirse y **comprobarse que Codex sigue ejecutando comandos
antes de consolidarlo**.

## El hook distingue leer de escribir

Una ruta mencionada no es una ruta escrita.

| Tipo de herramienta | Criterio |
|---|---|
| fichero (`write`, `edit`, `patch`, `create`, `delete`, `move`) | cualquier mención de ruta protegida → `deny` |
| shell (`shell`, `bash`, `powershell`, `exec`, `run`) | ruta protegida **más** indicio de escritura (`>`, `tee`, `sed -i`, `rm`, `mv`, `Set-Content`, `git restore`...) → `deny` |
| resto | `allow` |

Así `cat Historias.md` se permite y `echo x >> Historias.md` no.

Probado con 12 casos el 2026-09-21: 7 denegados y 5 permitidos, todos como se esperaba.
Los payloads son simulados: **el esquema real de entrada de Codex no está verificado.**

## Confianza de los hooks

Codex pide confirmación la primera vez que encuentra hooks en un repositorio
(`HookTrustStatus` en el binario). Es deliberado: código versionado no debe ejecutarse
sin que alguien lo acepte. Acéptalos una vez y quedan activos.

`--dangerously-bypass-hook-trust` existe; no lo uses.

## Estado de verificación

**DOCUMENTADO, no ejecutado.** La precedencia de la configuración de proyecto y el formato
de los hooks proceden de las cadenas del binario instalado, no de una sesión real de Codex.

Cómo comprobarlo: abre Codex en este repositorio y pídele que escriba en `Historias.md`.
Debe pedir aprobación y el hook debe denegarlo, **sin que se rompa la ejecución de
comandos**. Si los comandos volvieran a fallar, el problema está en esta configuración,
no en el hook: retírala y dilo.

**2026-09-21, primera prueba real:** Codex no pudo modificar `Historias.md` — la barrera
cumplió— pero informó de que el entorno fallaba al ejecutar comandos y no podía leer
ficheros. Causa: el `sandbox_mode` que entonces fijaba este fichero. Retirado.

Si el hook no denegara, **la única barrera real es el `pre-commit` de Git**
(`scripts/git-hooks/`): dilo y actualiza `.agents/meta/agent-capabilities.yaml`.

## Capa independiente

Pase lo que pase aquí, `scripts/git-hooks/pre-commit` sigue actuando: es independiente de
la herramienta y del agente. Instálalo con `git config core.hooksPath scripts/git-hooks`.
