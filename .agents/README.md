# Sistema canónico de instrucciones

Este directorio es la **fuente de verdad** del sistema de instrucciones para agentes.
`AGENTS.md` en la raíz es el punto de entrada; todo lo demás cuelga de aquí.

## Principio de diseño

**Los adaptadores referencian, no copian.** No existe ningún fichero generado que duplique
el contenido de otro. Así la divergencia no se vigila: es imposible por construcción,
salvo en un punto declarado más abajo.

## Organización

| Ruta | Responde a | Regla de oro |
|---|---|---|
| `rules/` | ¿Qué debe cumplirse? | Breve, verificable, no redundante, específica de su ámbito |
| `skills/` | ¿Cómo se hace bien una tarea especializada? | No duplica instrucciones permanentes |
| `workflows/` | ¿Qué secuencia sigue un proceso? | No duplica una skill; si se solapan, una sola fuente |
| `meta/` | ¿Qué se detectó y validó, cuándo y contra qué versión? | Caché, nunca sustituto de una verificación |

No hay `roles/`. No existe consumidor verificado para roles neutrales en las dos
herramientas soportadas, y `§26` del encargo prohíbe crear roles documentales sin consumidor.

## Jerarquía de reglas

```
universal/  →  proyecto/  →  (tecnología, cuando exista código)
```

Una regla específica no contradice una general sin documentar la excepción de forma
explícita, en el propio fichero de la regla específica.

**No existen reglas de Tauri, Rust, Vite, Tailwind ni testing.** Sus versiones están
fijadas en la constitución pero ninguna está instalada ni configurada. Se crearán cuando
la puerta G5 se cierre y exista el primer `package.json` / `Cargo.toml` real. Escribirlas
antes sería documentar un proyecto que no existe.

## Cómo consume cada herramienta

| Herramienta | Versión validada | Punto de entrada | Skills | Mecanismo |
|---|---:|---|---|---|
| Codex CLI | 0.155.1 | `AGENTS.md` (nativo) | `.agents/skills/` (nativo) | Carga directa: sin adaptador |
| Claude Code | 2.1.278 | `CLAUDE.md` → `@AGENTS.md` | `.claude/skills/*` (stubs) | Importación `@path` inlineada |

**Asimetría declarada.** La importación `@path` de Claude Code es determinista: inlinea el
fichero. En Codex, la referencia desde `AGENTS.md` a un fichero de `rules/` es prosa y
depende de que el modelo decida leerla. Por eso las reglas innegociables viven **dentro**
de `AGENTS.md` y `rules/` guarda solo las de ámbito acotado.

## Duplicación inevitable

El `name` y la `description` de cada skill deben repetirse en el frontmatter del stub de
`.claude/skills/`, porque el formato de Claude Code lo exige. Son dos líneas por skill y
`scripts/agent/check-adapters.mjs` las compara contra el canónico.

No se usan enlaces simbólicos: no están autorizados y son frágiles en Windows.

## Cómo añadir algo

1. Decide la capa: ¿regla, skill o workflow? Si dudas entre skill y workflow, es una skill.
2. Comprueba que no exista ya. **No dupliques una fuente de verdad**: enlaza a ella.
3. Escríbelo en el ámbito más específico que aplique, no en `universal/`.
4. Si es una restricción importante, pregúntate si puede reforzarse con un script en
   `scripts/agent/`. La prosa es el último recurso, no el primero.
5. Si afecta a una skill, recuerda regenerar su stub en `.claude/skills/` y ejecutar
   `node scripts/agent/verify.mjs`.

## Lo que NO se toca desde aquí

- `.specify/**` — gestionado por el CLI de SpecKit, con manifiestos SHA-256. Editarlo
  rompe `codex.manifest.json` y `speckit.manifest.json`.
- `.agents/skills/speckit-*/` — instaladas por SpecKit, incluidas en esos manifiestos.
- `Design/**` — entrega del diseñador.
- `Historias.md` — requisitos de producto.
