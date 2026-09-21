# Fuentes de verdad

**Ámbito:** universal. **Estado:** VERIFICADO. **Decisión del propietario:** 2026-09-21.

## Jerarquía

Cuando dos documentos se contradigan, prevalece el de arriba:

| # | Documento | Naturaleza | Estado |
|---|---|---|---|
| 1 | `.specify/memory/constitution.md` | Marco normativo | v0.6.0 **NO ratificada** |
| 2 | `Historias.md` | Requisitos de producto | Normativo, sin versionar en Git. **Editable** desde el 2026-09-21 por decisión del propietario; no está en el guard ni en el hook de Git |
| 3 | `Design/*.md` | Presentación: tokens, flujos, pantallas, avisos | Normativo donde no contradiga producto, seguridad o accesibilidad |
| 4 | `Design/maqueta-navegable.html` | Ilustración para enseñar y aprobar | **Nunca contrato** |

La constitución lo dice así: «Historias.md define producto dentro de este marco; Design
define presentación donde no contradiga producto, seguridad o accesibilidad».

## La constitución no está ratificada

Versión 0.6.0, `Ratified: pendiente — TODO(RATIFICATION_DATE)`. Enmiendas del 2026-09-21:
0.5.0 (accesibilidad básica desde H1, completa en H2) y 0.6.0 (matriz Windows 10 22H2 y
Windows 11 x64). Tiene abiertas:

- **Q2** (umbrales de cobertura) y **Q4** (tema inicial: Oscuro, respondido en `spec.md`,
  no enmendado). Q1 y la parte de accesibilidad de Q3 están enmendadas; el resto de Q3
  (offline, persistencia H1, 32 simultáneos, updater) está adoptado en la feature 001.
- **G1-G6**: puertas técnicas que deben resolverse con evidencia antes de implementar lo afectado.
- Una tabla de **17 discrepancias** `Historias.md` ↔ constitución «resueltas por esta
  propuesta» que aún **no se han trasladado** a los artefactos afectados.

Consecuencia: una propuesta de la constitución sin respuesta del propietario **no es una
aprobación**. Al planificar cualquier feature, identifica qué reglas aplicables siguen
siendo propuestas y decláralo. No codifiques las dos interpretaciones de una discrepancia
como oráculos simultáneos, ni uses un test para decidir el producto en silencio.

## `Especificacion.md` fue sustituido por `Historias.md`

**Decisión del propietario, 2026-09-21.**

`Historias.md` (1981 líneas, sin versionar) es la evolución directa de `Especificacion.md`
(1281 líneas, en el commit `0bb03cb`): mismo título, y sus primeras 1330 líneas difieren en
78 líneas de diff. Añade Tailwind CSS 4, la referencia obligatoria a `Design/` y una
sección de estrategia integral de pruebas.

Reglas derivadas:

- **Lee `Historias.md`.** Es la fuente de requisitos.
- `Especificacion.md` es **histórico**: vive en `HEAD`, está borrado del working tree
  a propósito y **no se restaura**.
- Existen **28 referencias a `Especificacion.md`** repartidas en 8 ficheros
  (`.specify/memory/constitution.md`, `Design/README.md`, `Design/FLUJOS.md`,
  `Design/PANTALLAS-H1.md`, `Design/AVISOS.md`, `Design/RESPONSIVE-I18N.md`,
  `Design/maqueta-navegable.html`, `Design/scripts/verify-tokens.mjs`).
  **Léelas como referencias a `Historias.md`.** La numeración de secciones §1-§28 se
  conserva entre ambos documentos.
- **No edites `Design/**` ni la constitución para reapuntarlas.** `Design/` es la entrega
  del diseñador y la auditoría v3 dejó constancia de que no se modificó; la constitución
  requiere autorización específica. La equivalencia se registra aquí, y
  `scripts/agent/check-references.mjs` las inventaría para que nadie las tome por rotas.

## Hueco abierto: puerta G6

La constitución pide «recuperar o reemplazar referencias al antiguo `Historias.md` §§29-54».
El `Historias.md` actual llega a §28 más la sección de estrategia de pruebas. **Esas
secciones §§29-54 siguen sin localizar.** No cierres requisitos de contenido desconocido:
si una tarea depende de ellas, dilo y para.

## Lo que no es fuente de verdad

- `.logs/app.jsonl` — **no pertenece a este proyecto**. Son logs del servidor MCP
  `mcp-coolify` escritos en el directorio de trabajo. Ignóralo.
- `Design/src/lib/` — código de **referencia**, sin build ni aplicación. `theme.svelte.ts`
  usa `localStorage` como sustituto documentado de `settings.json`: es un marcador de
  posición, no un contrato.
- El listado de herramientas instaladas en la máquina. No equivale a las versiones del
  proyecto: ver `.agents/rules/proyecto/versiones.md`.
