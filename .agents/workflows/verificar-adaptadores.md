# Workflow — verificar adaptadores

**Cuándo:** tras tocar `AGENTS.md`, `.agents/`, `CLAUDE.md` o `.claude/`; tras actualizar
Claude Code o Codex; o antes de cerrar cualquier tarea que haya modificado el sistema de
instrucciones.

**Objetivo:** que ningún adaptador se convierta en una fuente de verdad paralela.

## Qué es canónico y qué es adaptador

| Canónico (se edita) | Adaptador (se regenera) |
|---|---|
| `AGENTS.md` | `CLAUDE.md` |
| `.agents/rules/**` | `.claude/skills/speckit-*/SKILL.md` |
| `.agents/skills/**` | `.claude/settings.json` |
| `.agents/workflows/**` | |

**Nunca edites un adaptador para cambiar una regla.** Cambia el canónico.

## 1. Comprobación automática

```
node scripts/agent/verify.mjs
```

| Script | Comprueba | Si falla |
|---|---|---|
| `check-agents-size.mjs` | `AGENTS.md` < 200 líneas | Mueve contenido a `.agents/rules/`, no recortes reglas |
| `check-adapters.mjs` | Cada skill canónica tiene stub y viceversa; `name` y `description` coinciden | Regenera el stub desde el canónico |
| `check-references.mjs` | Ninguna referencia a fichero inexistente en `AGENTS.md`, `CLAUDE.md` y `.agents/**` | Corrige la ruta; **no** crees el fichero para tapar el aviso |
| `check-spec-history.mjs` | Documentos históricos de `HEAD` intactos; `specs/**` no borradas | Restaura lo que hayas tocado e informa |
| `verify-tokens.mjs` | Sin colores literales fuera de `tokens.css` | Usa `var(--token)` |

## 2. Regenerar un stub de skill

Un stub de `.claude/skills/<n>/SKILL.md` contiene **solo**: frontmatter con `name` y
`description` copiados literalmente del canónico, y una instrucción de leer y seguir
`.agents/skills/<n>/SKILL.md`. **Nunca** copies el cuerpo: eso crea divergencia.

Si añades una skill canónica nueva, crea su stub en el mismo cambio.

## 3. Comprobación manual de carga

Los scripts verifican ficheros, no comportamiento. Para comprobar que la herramienta
carga de verdad lo que debe, sigue `.agents/skills/verificar-sistema-agentes/SKILL.md`,
secciones 2 y 3. Recuerda que Claude Code **requiere sesión nueva** tras cambiar
`settings.json`, las skills o los ficheros importados.

## 4. Registrar

Si has validado contra una versión nueva de una herramienta, actualiza
`.agents/meta/agent-capabilities.yaml` (versión, fecha, capacidades usadas y limitaciones)
y `.agents/meta/validation-state.yaml` (qué se ejecutó y qué no).

## 5. Declarar honestamente

Los scripts son garantías deterministas: su salida se cita tal cual. La carga en Codex se
verifica observando una sesión real; hasta entonces es `INFERIDO`, no `VERIFICADO`. No
presentes una instrucción escrita en prosa como si fuera una garantía.
