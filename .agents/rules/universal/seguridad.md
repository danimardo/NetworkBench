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

## Limitación conocida del entorno (VERIFICADO, no corregible desde el repositorio)

`~/.codex/config.toml` fija globalmente `approval_policy = "never"` y
`sandbox_mode = "danger-full-access"`, y marca `f:\apps\netbench` como `trust_level = "trusted"`.
Ninguna regla de este repositorio puede contrarrestarlo: se corrige en ese fichero, que
está fuera del alcance del sistema de agentes. Consecuencia práctica: **en Codex, estas
reglas son la única barrera; no hay una segunda oportunidad de aprobación.**

Claude Code sí aplica `.claude/settings.json`, cuyos permisos de proyecto son de solo lectura.

## Operaciones que requieren autorización específica

Instalar paquetes · actualizar dependencias · modificar lockfiles · instalar plugins o MCP ·
modificar configuración global del usuario · modificar CI · cualquier cambio remoto ·
cualquier operación de red que no sea consultar documentación.

## Al auditar seguridad

No inventar vulnerabilidades ni calificar una configuración de insegura sin evidencia.
Distinguir siempre: riesgo confirmado · riesgo potencial · configuración no verificable ·
ausencia de evidencia.
