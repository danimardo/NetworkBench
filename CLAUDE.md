# NetworkBench — adaptador para Claude Code

Este fichero **no es la fuente de verdad**: es el adaptador que carga el sistema canónico
en Claude Code. Para cambiar una regla, edita el canónico en `.agents/`, nunca este fichero.
Contexto del diseño: `.agents/README.md`.

## Sistema canónico

Las importaciones de abajo inlinean el contenido: lo que se lista aquí llega siempre.

@AGENTS.md

@.agents/rules/universal/idioma.md
@.agents/rules/universal/git.md
@.agents/rules/universal/seguridad.md
@.agents/rules/universal/fuentes-de-verdad.md
@.agents/rules/universal/speckit.md
@.agents/rules/proyecto/estado.md
@.agents/rules/proyecto/versiones.md
@.agents/rules/proyecto/diseno.md

## Específico de Claude Code

- **Skills.** Las de `.claude/skills/` son *stubs*: su cuerpo solo te dice qué canónico
  leer. El contenido real está en `.agents/skills/<nombre>/SKILL.md`. No edites un stub
  para cambiar un procedimiento.
- **Skills sin stub.** `cierre-de-tarea` y `verificar-sistema-agentes` viven solo en
  `.agents/skills/`. No aparecerán en el listado de `/`: léelas por ruta cuando las necesites.
  Usa `cierre-de-tarea` **antes de dar por terminada cualquier tarea**.
- **Workflows.** `.agents/workflows/inspeccionar-repositorio.md` y
  `.agents/workflows/verificar-adaptadores.md`. No hay mecanismo nativo de workflows:
  se leen y se siguen.
- **Permisos.** `.claude/settings.json` define una allowlist de solo lectura y deniega las
  operaciones de escritura de Git. Si algo te hace falta, **pídelo; no amplíes el permiso.**
- **Sesión nueva.** Los cambios en `settings.json`, en las skills o en cualquier fichero
  importado aquí no surten efecto hasta reiniciar la sesión.

## Antes de terminar

Si tocaste `AGENTS.md`, `.agents/`, `CLAUDE.md` o `.claude/`:

```
node scripts/agent/verify.mjs
```

Cita su salida real. No hay CI ni hooks: nadie lo ejecuta por ti.
