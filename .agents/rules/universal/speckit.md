# SpecKit

**Ámbito:** universal. **Estado:** VERIFICADO (`.specify/init-options.json`,
`.specify/integration.json`, manifiestos SHA-256).

## Configuración instalada

| Clave | Valor |
|---|---|
| Versión | `1.0.9.dev0` |
| Integración | `codex` (única instalada) |
| Scripts | PowerShell (`script: "ps"`) |
| Numeración de features | `sequential` |
| Instalado | 2026-09-21T03:48Z |

Las skills canónicas están en `.agents/skills/speckit-*/SKILL.md` (10). Codex las carga de
forma nativa desde ahí; Claude Code las alcanza mediante los stubs de `.claude/skills/`.

## Ciclo

```
speckit-constitution → speckit-specify → speckit-clarify → speckit-plan
                     → speckit-tasks → speckit-implement
```

Auxiliares: `speckit-analyze` (consistencia entre artefactos), `speckit-checklist`,
`speckit-converge` (trabajo pendiente frente al código real), `speckit-taskstoissues`.

**No existe `specs/`.** Ninguna feature se ha planificado todavía.

## Reglas

- **No modificar `.specify/memory/constitution.md`** sin autorización específica del
  propietario. Si crees que debe cambiar: motivo, diff, impacto, y esperar. La skill
  `speckit-constitution` no es esa autorización.
- **No editar nada bajo `.specify/`** ni las 10 skills `speckit-*`. Están cubiertas por
  manifiestos SHA-256 (`codex.manifest.json`, `speckit.manifest.json`); editarlas rompe la
  integridad que el CLI comprueba.
- **No borrar ni sobrescribir specs históricas.** Si hay que migrarlas: informar, proponer
  estrategia, esperar. `scripts/agent/check-spec-history.mjs` lo comprueba.
- Cada `spec.md`, `plan.md` y `tasks.md` identifica **qué versión de la constitución** usó.
  Hoy sería 0.4.0 no ratificada: decláralo.
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
