# NetworkBench — instrucciones para agentes

Punto de entrada del sistema de instrucciones. Portable: lo carga cualquier herramienta
que soporte `AGENTS.md`. El detalle por ámbito vive en `.agents/`; aquí solo está lo
permanente, transversal y necesario en casi cualquier tarea.

## Idioma

Comunícate en español, con ortografía y acentuación correctas. Informes, propuestas,
planes, reglas, skills, workflows y documentación: en español. Código, identificadores,
nombres de API, rutas y términos técnicos: según la convención real del repositorio.

## Qué es este proyecto

Aplicación de escritorio para Windows que mide el rendimiento de red entre dos equipos
usando Microsoft NTTTCP como motor. Frontend Svelte/TypeScript sobre Tauri 2 + Rust,
renderizado en WebView2. Todo local: sin nube, sin cuentas, sin servidor.

## Estado real (2026-09-21)

**La aplicación no existe todavía.** El repositorio contiene documentación normativa y un
sistema de diseño de referencia. No hay `package.json`, `Cargo.toml`, `src/`, `src-tauri/`,
`tests/` ni CI.

Consecuencia directa: **no hay comandos de build, test o lint del proyecto.** No los
inventes, no los ejecutes y no declares aprobado lo que no se ha ejecutado. El único
comando verificado hoy es `node Design/scripts/verify-tokens.mjs`.

## Fuentes de verdad y jerarquía

Cuando dos documentos se contradigan, prevalece el de arriba:

1. `.specify/memory/constitution.md` — marco normativo. **Versión 0.4.0, NO ratificada**,
   con decisiones Q1-Q4 abiertas y puertas técnicas G1-G6 sin cerrar. Una propuesta suya
   sin respuesta no es una aprobación.
2. `Historias.md` — requisitos de producto (§§1-28 + estrategia de pruebas).
3. `Design/` — presentación: tokens, componentes, flujos, pantallas y maqueta.
4. `Design/maqueta-navegable.html` — ilustrativa, nunca contrato.

`Especificacion.md` fue **sustituido por `Historias.md`**. Sigue en el historial de Git
(commit `0bb03cb`) como documento histórico. Toda referencia a `Especificacion.md` en
`Design/` y en la constitución debe leerse como referencia a `Historias.md`.
Detalle en `.agents/rules/universal/fuentes-de-verdad.md`.

## Reglas universales innegociables

Estas no se delegan a un fichero aparte: se cumplen siempre.

- **Git.** Sin autorización explícita y específica: no hacer commit, push, amend, rebase,
  merge, tags, ramas, cambios de rama, PR ni fusiones. Nunca revertir ni limpiar trabajo
  existente que no hayas creado tú en esta sesión.
- **Constitución.** No modificar `.specify/memory/constitution.md` sin autorización
  específica. Si crees que debe cambiar: explica el motivo, muestra el diff, explica el
  impacto y espera.
- **Specs históricas.** No borrar ni sobrescribir specs ni documentos históricos. Si hace
  falta migrarlos, propón la estrategia y espera.
- **Secretos.** No leer secretos sin necesidad, no mostrarlos, no copiarlos, no registrarlos
  y no exponer variables privadas al cliente.
- **Mínimo privilegio.** Nunca ampliar permisos, capabilities ni alcance del sandbox para
  desbloquear tu propio trabajo. Si algo está bloqueado, dilo.
- **Evidencia.** No inventar rutas, comandos, versiones ni capacidades. Si algo solo puede
  inferirse, identifícalo como inferencia.
- **Alcance.** No corregir problemas no relacionados ni ampliar el alcance de la tarea.

## Estados de verificación

Usa estos términos en vez de expresiones ambiguas como «parece funcionar»:

`VERIFICADO` (evidencia directa del repositorio o de un comando ejecutado) ·
`DOCUMENTADO` (respaldado por documentación oficial de la versión detectada, no ejecutado) ·
`INFERIDO` (evidencia indirecta razonable, insuficiente) ·
`NO VERIFICABLE` (el entorno no lo permite) ·
`NO PRESENTE` (comprobado que no existe).

## Metodología

El ciclo de trabajo es SpecKit 1.0.9.dev0, integración `codex`, scripts PowerShell:

```
specify → clarify → plan → tasks → implement
```

Las skills canónicas están en `.agents/skills/speckit-*/`. Todavía no existe `specs/`:
ninguna feature se ha planificado. Detalle en `.agents/rules/universal/speckit.md`.

## Definición de terminado

Una tarea no está terminada mientras:

- quede trabajo del alcance sin hacer y sin declarar explícitamente como no hecho;
- se presente como verificado algo que no se ha ejecutado;
- se presente prosa como garantía determinista;
- queden contradicciones materiales sin señalar;
- no se haya dicho qué no pudo comprobarse y por qué.

Procedimiento completo en `.agents/skills/cierre-de-tarea/SKILL.md`.

## Verificación del propio sistema

```
node scripts/agent/verify.mjs
```

Ejecuta las garantías deterministas: tamaño de este fichero, coherencia de los adaptadores,
referencias colgantes, protección de documentos históricos, rutas protegidas y tokens de
diseño. No hay CI: **nadie lo ejecuta por ti.**

La única barrera que no depende de ningún agente es el hook de Git. Instálalo una vez
por clon y no lo esquives:

```
git config core.hooksPath scripts/git-hooks
```

Rechaza cualquier commit que toque `.specify/`, `.agents/skills/speckit-*`, `Design/`,
`Historias.md` o los documentos históricos. `--no-verify` existe, requiere autorización
del propietario y debe declararse en el mensaje del commit.

## Herramientas de agentes soportadas

El desarrollo de NetworkBench se lleva principalmente con **Codex CLI y Claude Code**.
Es un hecho del entorno de trabajo, no un requisito del producto: por eso vive aquí y no
en `Historias.md`.

| Herramienta | Versión validada | Punto de entrada | Skills |
|---|---:|---|---|
| Codex CLI | 0.155.1 | este fichero, de forma nativa | `.agents/skills/` nativo |
| Claude Code | 2.1.278 | `CLAUDE.md`, que importa este fichero | `.claude/skills/` (stubs) |

OpenCode está instalado en la máquina pero **no es una herramienta soportada** por este
repositorio. Capacidades, límites y fecha de validación de cada una:
`.agents/meta/agent-capabilities.yaml`. No asumas que una capacidad sigue igual tras
actualizar: procedimiento en `.agents/skills/verificar-sistema-agentes/SKILL.md`.

## Mapa del sistema canónico

| Ruta | Contenido |
|---|---|
| `.agents/rules/universal/` | Reglas de ámbito general: idioma, git, seguridad, fuentes de verdad, SpecKit |
| `.agents/rules/proyecto/` | Reglas de este proyecto: estado, versiones, diseño |
| `.agents/skills/` | Procedimientos especializados, incluidas las 10 skills de SpecKit |
| `.agents/workflows/` | Secuencias de proceso reutilizables |
| `.agents/meta/` | Stack detectado, capacidades de herramientas y estado de validación |
| `scripts/agent/` | Garantías deterministas |

Lee la regla del ámbito que vayas a tocar **antes** de tocarlo. `.agents/README.md`
explica cómo está organizado y cómo añadir algo sin duplicar fuentes de verdad.
