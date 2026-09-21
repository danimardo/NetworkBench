# Diseño y presentación

**Ámbito:** `Design/` y, cuando exista, todo el frontend. **Estado:** VERIFICADO.

## Qué es `Design/`

La entrega del diseñador, versión v4 («cierre de auditoría H1»). Es **referencia
obligatoria** de presentación: `Historias.md` la declara así en su cabecera y en §16.0.

| Fichero | Contenido |
|---|---|
| `README.md` | Índice de lectura, API de los componentes, reglas para componentes nuevos |
| `DESIGN_TOKENS.md` | Catálogo de tokens |
| `FLUJOS.md` | Recorridos |
| `PANTALLAS-H1.md` | Pantallas y estados del hito H1 |
| `RESPONSIVE-I18N.md` | Adaptación y bilingüismo |
| `AVISOS.md` | Avisos y mensajes |
| `maqueta-navegable.html` | Prototipo para enseñar y aprobar |
| `src/lib/components/` | 18 componentes Svelte 5 de referencia |
| `src/lib/design-system/` | `tokens.css` y `theme.svelte.ts` (runes) |
|  `Design/scripts/verify-tokens.mjs` | Verificador de tokens |

## Reglas

- **No modificar `Design/`.** Es entrega de un tercero; la auditoría v3 dejó constancia
  de que no se alteró. Si algo debe cambiar: informar, proponer, esperar.
- **Ningún valor de color literal fuera de `tokens.css`.** Solo `var(--token)`.
  `Historias.md` §16.10 lo exige «verificable por script en CI».
  Verificación: `node Design/scripts/verify-tokens.mjs` → pasa hoy.
- **La maqueta no es contrato.** Ilustra; no se traslada literalmente al producto.
  El README lo dice y la auditoría v3 lo subraya.
- El sistema de diseño **no incorpora librerías de gráficas ni de iconos**: SVG y
  componentes propios, versionados con el commit (constitución).
- Fuentes: familias de Windows con reserva local. Cualquier fuente adicional se empaqueta
  con versión, hash y licencia. **Nunca cargar una fuente desde Internet.**

## Límites conocidos del verificador (no los presentes como garantía)

`verify-tokens.mjs` comprueba ausencia de `#RRGGBB`, `#RGB`, `rgb(` y `rgba(` fuera de
`tokens.css`, recorriendo `src/`. **No** comprueba tokenización de dimensiones, tipografía
ni animación; **no** cubre todos los formatos CSS de color; **no** demuestra accesibilidad
ni corrección funcional. Su comentario de cabecera cita `Especificacion.md §3.5`: léase
`Historias.md §3.5`.

## Lo que sigue pendiente de la auditoría v3

- Sin validación visual renderizada, de interacción ni con lector de pantalla:
  no había navegador conectado. NO VERIFICABLE hasta hoy.
- Los tokens de `tokens.css` y los de la maqueta **no son copias equivalentes**.
- H2 y H3 no están diseñados. Entregar por hitos es válido; dejarlos para después no
  equivale a haberlos diseñado.
- Accesibilidad exigible **desde H1** (WCAG 2.2 AA), con ampliación propia hasta el 200 %.

## Cuando exista el frontend real

Tailwind CSS 4.3.3 es obligatorio (constitución y `Historias.md`). Svelte 5 con runes, sin
SvelteKit, sin SSR y sin hidratación. Las reglas de framework se escribirán cuando exista
`package.json`: hoy no hay proyecto que reglar.
