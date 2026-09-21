# Seguridad y privilegio mínimo

**Ámbito:** universal. **Estado:** mezcla; cada punto lleva el suyo.

## Secretos

- No leer secretos sin necesidad, no mostrarlos, no copiarlos, no registrarlos en logs ni
  incluirlos en prompts, informes o artefactos. Para comprobar que una variable sensible
  existe, comprueba su presencia sin imprimir su valor.
- No crear credenciales ni exponer variables privadas al cliente (constitución, principio XII).
- **Riesgo confirmado, fuera de este repositorio (VERIFICADO):** el fichero global
  `~/.config/opencode/opencode.jsonc` contiene un token de Coolify y una clave de API de
  Context7 en texto plano. No los reproduzcas. Recomendación pendiente: rotarlos.
  OpenCode no es una herramienta soportada por este repositorio.

## Privilegio mínimo

- **Nunca ampliar permisos, capabilities, alcance del sandbox o política de aprobación
  para desbloquear tu propio trabajo.** Si algo está bloqueado, dilo y para.
- Aplica a: herramientas del agente, MCP, acceso a ficheros, red, y —cuando exista código—
  a las capabilities de Tauri, la lista blanca del firewall y los comandos IPC.
- Un revisor no necesita edición, instalación, push, PR, configuración global ni secretos.

## Barreras efectivas de este repositorio

Estado a 2026-09-21, tras probarlas todas en sesiones reales de las dos herramientas.

| Capa | Alcance | Estado |
|---|---|---|
| `scripts/git-hooks/pre-commit` | **cualquier herramienta, y tú a mano** | **VERIFICADO** |
| Hook `PreToolUse` → `scripts/agent/guard-protected-paths.mjs` | Claude Code | **VERIFICADO** |
| `.claude/settings.json` (`deny` / `ask`) | Claude Code | VERIFICADO tras corregir los globs |
| El mismo hook en Codex | Codex | **NO FUNCIONAL: no llega a dispararse** |
| `.codex/config.toml` (`approval_policy`) | Codex | **NO ES BARRERA: decide el modelo** |

El guard es **un solo fichero** que invocan las dos herramientas: no hay copias que puedan
divergir. Instalación del hook de Git, una vez por clon:

```
git config core.hooksPath scripts/git-hooks
```

### Dentro de una sesión de Codex no hay contención hoy

VERIFICADO el 2026-09-21: Codex leyó estas mismas reglas, invocó la skill de cierre y
editó igualmente una ruta protegida. Las tres vías están cerradas por ahora:

- El hook no se dispara. La característica `hooks` está `stable` y activada, así que es un
  problema de ubicación del fichero, sin resolver. Ver `.codex/hooks/README.md`.
- `approval_policy` no admite ningún valor que **obligue** a preguntar: solo `on-request`,
  donde decide el modelo, y `never`.
- El sandbox exige `codex sandbox setup --elevated`, que no se ha ejecutado. Fijarlo sin
  ese paso deja Codex sin poder ejecutar comandos: ya ocurrió el mismo día.

**Consecuencia práctica: con Codex, el commit es el primer punto donde algo se puede
impedir de verdad.** Trátalo en consecuencia y no confíes en que una regla escrita baste.

### Configuración global, fuera del alcance de este repositorio

`~/.codex/config.toml` fija `approval_policy = "never"` y
`sandbox_mode = "danger-full-access"` para todos los proyectos, y marca
`f:\apps\netbench` como `trust_level = "trusted"`. La causa raíz se corrige ahí, no aquí.

### Los globs de permisos no se dan por buenos sin probarlos

El 2026-09-21 las reglas `deny` de Claude Code estaban escritas con un prefijo de
directorio actual y **no bloquearon nada**: esa forma no está reconocida y el glob falló
en silencio. Las válidas son `Edit(ruta)`, `Edit(**/ruta)`, `Edit(//absoluta)` y
`Edit(~/ruta)`; `Edit(...)` ya cubre Write y NotebookEdit. Un glob mal escrito no avisa:
por eso existe el hook además del `deny`.

## Operaciones que requieren autorización específica

Instalar paquetes · actualizar dependencias · modificar lockfiles · instalar plugins o MCP ·
modificar configuración global del usuario · modificar CI · cualquier cambio remoto ·
cualquier operación de red que no sea consultar documentación.

## Al auditar seguridad

No inventar vulnerabilidades ni calificar una configuración de insegura sin evidencia.
Distinguir siempre: riesgo confirmado · riesgo potencial · configuración no verificable ·
ausencia de evidencia.
