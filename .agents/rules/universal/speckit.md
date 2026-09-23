# SpecKit

**Ámbito:** universal. **Estado:** VERIFICADO el 2026-09-22 (`specify integration list`,
`.specify/init-options.json`, `.specify/integration.json`, manifiestos SHA-256).

## Configuración instalada

| Clave | Valor |
|---|---|
| Versión | `1.0.9.dev0` |
| Integración | `agy` (Antigravity), única instalada y por defecto |
| Scripts | PowerShell (`script: "ps"`) |
| Numeración de features | `sequential` |
| Instalado | 2026-09-21T18:44Z |

Las skills canónicas están en `.agents/skills/speckit-*/SKILL.md` (10). Codex las carga de
forma nativa desde ahí; Claude Code las alcanza mediante los stubs de `.claude/skills/`.

## La integración no es la herramienta que usas

Distinción que causó una confusión real: la integración de SpecKit solo decide **qué
ficheros escribe y rehashea el CLI**, no con qué agente trabajas. Codex CLI y Claude Code
siguen siendo las herramientas soportadas (`AGENTS.md`), y ambas funcionan hoy con la
integración `agy` instalada.

Hasta el 2026-09-21T18:44Z la integración era `codex`. Alguien la sustituyó por `agy`, lo
que reescribió el prefijo de invocación `$speckit-x` → `/speckit-x` en 130 líneas de las 10
skills, las 3 plantillas y los 3 scripts PowerShell, borró `codex.manifest.json` y creó
`agy.manifest.json`. **Ese cambio es cosmético**: el prefijo solo importa en la sección de
hooks de las skills, y este repositorio no declara extensiones de SpecKit, así que esa
sección está inerte. Las dos herramientas leen cualquiera de las dos variantes.

## No instales la integración `claude`

VERIFICADO el 2026-09-22 en una copia desechable, no en este árbol:
`specify integration install claude --force` **sobrescribe los 10 stubs de
`.claude/skills/`** con copias completas del canónico (13 KB frente a los ~700 B del stub).

`node scripts/agent/check-adapters.mjs` rechaza el resultado con 20 fallos: «el stub no
remite a su canónico» y «parece copiar el canónico en vez de apuntarlo». Es incompatible
con el diseño de adaptadores finos de `.agents/README.md`: duplicaría la fuente de verdad.

`codex` y `agy` son peor todavía: **gestionan exactamente los mismos 10 ficheros**
`.agents/skills/speckit-*/SKILL.md`, con contenido distinto. No pueden coexistir; por eso
`agy` figura en el catálogo con `Multi-install Safe: no` y cualquier instalación paralela
exige `--force`.

Si algún día hay que cambiar de integración: `specify integration use <key>` cambia el
predeterminado sin desinstalar; `specify integration switch <key>` **desinstala la
anterior**. El segundo es el que provocó la sustitución.

## Ciclo

```
speckit-constitution → speckit-specify → speckit-clarify → speckit-plan
                     → speckit-tasks → speckit-implement
```

Auxiliares: `speckit-analyze` (consistencia entre artefactos), `speckit-checklist`,
`speckit-converge` (trabajo pendiente frente al código real), `speckit-taskstoissues`.

**Existe `specs/001-network-benchmark-v1/`** (sin versionar) con `spec.md`, `plan.md`,
`tasks.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/` y `checklists/`.
Es la única feature; `$speckit-analyze` se ejecutó el 2026-09-21.

## Reglas

- **`.specify/memory/constitution.md` se edita con permiso del propietario**, pedido en
  el momento con motivo, diff e impacto (decisión del 2026-09-21). Claude Code lo pregunta
  solo (`ask`); en Codex, pregunta tú. La skill `speckit-constitution` no es ese permiso.
  Cada enmienda sube la versión y se declara en el commit.
- **No editar nada más bajo `.specify/`** ni las 10 skills `speckit-*`. Están cubiertas por
  manifiestos SHA-256 (`codex.manifest.json`, `speckit.manifest.json`); editarlas rompe la
  integridad que el CLI comprueba.
- **No borrar ni sobrescribir specs históricas.** Si hay que migrarlas: informar, proponer
  estrategia, esperar. `scripts/agent/check-spec-history.mjs` lo comprueba.
- Cada `spec.md`, `plan.md` y `tasks.md` identifica **qué versión de la constitución** usó.
  Hoy sería 0.7.0 no ratificada: decláralo.
- El Constitution Check de cada plan cubre arquitectura, versiones, seguridad, datos,
  accesibilidad, rendimiento, pruebas y distribución. Un incumplimiento exige cambiar el
  plan o tramitar una enmienda, nunca una excepción tácita.

## Scripts

Están en `.specify/scripts/powershell/`: `check-prerequisites.ps1`, `common.ps1`,
`create-new-feature.ps1`, `resolve-template.ps1`, `setup-plan.ps1`, `setup-tasks.ps1`.

Se invocan con Windows PowerShell 5.1. **`pwsh` no está instalado en esta máquina**
(VERIFICADO): no asumas PowerShell 7 ni uses `&&`, `||`, ternarios ni `-AsHashtable`.

## Estado local no compartido

`.specify/.gitignore` excluye `feature.json` (puntero a la feature actual) y
`extensions/*/local-config.yml`. Es estado por máquina: no lo versiones ni lo cites como
configuración del proyecto.
