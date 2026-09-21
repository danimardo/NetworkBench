---
name: "verificar-sistema-agentes"
description: "Comprobar que cada herramienta de agentes carga las instrucciones, reglas y skills de NetworkBench, y detectar si un cambio de versión alteró sus capacidades. Usar tras actualizar Claude Code o Codex, o al sospechar que no se está cargando el contexto."
compatibility: "NetworkBench. Herramientas soportadas: Claude Code y Codex CLI."
metadata:
  author: "sistema de agentes NetworkBench"
  canonical: ".agents/skills/verificar-sistema-agentes/SKILL.md"
---

# Verificar el sistema de agentes

Dos cosas distintas: que **los ficheros sean coherentes** (lo comprueban los scripts) y que
**la herramienta los cargue de verdad** (lo comprueba una persona, observando la sesión).

## 1. Garantías deterministas

```
node scripts/agent/verify.mjs
```

Agrupa cinco comprobaciones: tamaño de `AGENTS.md` (< 200 líneas), coherencia entre
`.agents/skills/speckit-*` y sus stubs en `.claude/skills/`, referencias colgantes,
protección de documentos históricos, y tokens de diseño.

Cada una es ejecutable por separado desde `scripts/agent/`. **Nada las ejecuta sola:** no
hay CI ni hooks en este repositorio.

## 2. Comprobar la carga en Claude Code (2.1.278, VERIFICADO)

| Qué | Cómo |
|---|---|
| Workspace | El directorio de trabajo debe ser `F:\Apps\NetBench` |
| Instrucciones | `/memory` lista los ficheros de memoria cargados. Debe aparecer `CLAUDE.md` del proyecto |
| Importaciones | `CLAUDE.md` usa `@ruta`, que **inlinea** el fichero. Si `AGENTS.md` y las reglas no aparecen en el contexto, la ruta está mal escrita |
| Skills | `/` lista las skills disponibles. Deben aparecer las diez `speckit-*` más `cierre-de-tarea` y `verificar-sistema-agentes` |
| Invocar una skill | `/speckit-specify` o, en lenguaje natural, pedir la tarea que describe |
| Permisos | `/permissions` muestra los efectivos. Los de `.claude/settings.json` son de solo lectura |
| Bloqueos | Pedir una operación prohibida (p. ej. un commit) debe producir una negativa o una petición de permiso, **nunca** la ejecución |
| Reinicio | Cambios en `settings.json`, en las skills o en los ficheros importados: **requieren sesión nueva** |

## 3. Comprobar la carga en Codex CLI (0.155.1)

| Qué | Cómo |
|---|---|
| Instrucciones | Codex lee `AGENTS.md` de la raíz de forma nativa. `AGENTS.override.md` tiene prioridad si existe; hoy no existe y no debe crearse sin motivo |
| Skills | Codex lee las skills del repositorio desde `.agents/skills/`. **Estado: INFERIDO con evidencia fuerte** — cadenas del binario instalado (`"failed to stat repo skills root"` junto a `.agents` + `skills`, y la ruta de fuente `ext\skills\src\host_prompt.rs`). **No se ha comprobado ejecutando Codex** |
| Cómo verificarlo de verdad | Abrir una sesión de Codex en el repositorio y pedirle que liste sus skills disponibles, o invocar `speckit-specify`. Si no aparecen, el canónico necesita un adaptador propio y hay que informar |
| Reglas | Codex **no tiene mecanismo propio de reglas**. Las de `.agents/rules/` le llegan como referencia en prosa desde `AGENTS.md`: es una garantía dependiente del modelo, no determinista |
| Permisos | Se configuran en `~/.codex/config.toml`, **fuera de este repositorio**. Hoy: `approval_policy = "never"`, `sandbox_mode = "danger-full-access"`, proyecto `trusted`. Ninguna regla de aquí lo contrarresta |

## 4. Tras actualizar una herramienta

**No regeneres nada automáticamente.** Una versión nueva no demuestra que una capacidad
haya cambiado, ni que siga existiendo.

1. Registra la versión nueva: `claude --version`, `codex --version`.
2. Compárala con la validada en `.agents/meta/agent-capabilities.yaml`.
3. Si coinciden, reutiliza las capacidades ya verificadas: no repitas la auditoría.
4. Si difieren, revisa **solo la herramienta afectada**: mecanismo de carga, rutas de
   instrucciones, rutas de skills, formato de frontmatter, permisos y hooks.
5. Clasifica el cambio: `SIN IMPACTO` · `REVISIÓN RECOMENDADA` · `CAMBIO NECESARIO` ·
   `INCOMPATIBILIDAD`, con la evidencia.
6. Si requiere modificar algo, **presenta los cambios y pide aprobación**.
7. Actualiza `.agents/meta/agent-capabilities.yaml` y `validation-state.yaml` con la
   versión, la fecha y qué se comprobó.

No copies configuración de una versión distinta sin comprobarla. No asumas que una
capacidad de la CLI existe igual en IDE, escritorio o ejecución remota.

## 5. Si algo no se carga

Antes de tocar nada, descarta lo barato: directorio de trabajo equivocado, sesión sin
reiniciar, ruta de importación mal escrita, frontmatter inválido en un `SKILL.md`.

Después informa con evidencia. **No amplíes permisos ni dupliques contenido para
esquivar el problema**: eso convierte un fallo visible en una divergencia silenciosa.
