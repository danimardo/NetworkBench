# NetworkBench — referencia de tokens

Fuente única de verdad: `src/lib/design-system/tokens.css`. Esta tabla es
solo una vista de consulta rápida — si hay alguna discrepancia, el CSS
manda. Todo color, gradiente o sombra coloreada de la interfaz **tiene que
salir de aquí**; ningún componente escribe un `#`, un `rgb(`/`rgba(` ni un
`linear-/radial-gradient(` a pelo (`scripts/verify-tokens.mjs` lo comprueba
en CI).

## Identidad: dos temas, una app

- **Oscuro — "Graphite Violet Dark"** (por defecto): grafito + azul noche +
  violeta/lavanda/malva como luz ambiental.
- **Claro — "Pearl Lavender Light"**: blanco perla + gris lavanda + el mismo
  violeta como acento, en un tono algo más oscuro/saturado para conservar
  contraste sobre fondo claro.
- Proporción orientativa en ambos: **65 % grafito/perla neutro · 20 %
  gris-violeta · 10 % lavanda/malva ambiental · 5 % violeta intenso.** El
  violeta es marca/acento, nunca el color dominante — si una pantalla nueva
  "se ve muy morada", es una señal de que se ha roto esta proporción.
- Los dos temas deben reconocerse como **la misma aplicación**: mismo
  violeta, mismo rosa/malva, mismos radios, mismas cards, mismo fondo
  abstracto. Solo cambian luminosidad, opacidad y contraste.

## Colores de fondo y superficie

| Token | Oscuro | Claro | Uso |
|---|---:|---:|---|
| `--color-bg-base` | `#14172A` | `#F4F3F8` | Fondo más profundo (detrás de todo) |
| `--color-bg-main` | `#191C30` | `#F8F7FB` | Fondo principal de la ventana |
| `--color-bg-elevated` | `#202237` | `#FFFFFF` | Superficies elevadas puntuales (raro fuera de cards/hero, que usan sus propios gradientes) |

## Texto

| Token | Oscuro | Claro | Uso | Contraste sobre `--color-bg-main` |
|---|---:|---:|---|---:|
| `--color-text-primary` | `#F6F2FB` | `#252331` | Nombres, títulos, valores principales | ~15.2:1 / ~14.4:1 |
| `--color-text-secondary` | `#C5BED4` | `#5F596C` | Metadatos de segundo nivel, subtítulos | ~9.4:1 / ~6.3:1 |
| `--color-text-tertiary` | `#A49DB8` | `#756E82` | Etiquetas discretas, nav inactiva | ~6.5:1 / ~4.6:1 |
| `--color-text-muted` | `#8F8AA2` | `#6B6478` | Información poco relevante ("3 en la red") | ~5.1:1 / ~5.3:1 |
| `--color-text-disabled` | `#696579` | `#A9A3B2` | Texto de controles deshabilitados | — (no es texto de lectura) |

Todos superan el mínimo AA (4.5:1 para texto normal, 3:1 para ≥24px/negrita)
sobre su fondo principal. **No introducir un tono de texto más oscuro que
`--color-text-muted`** salvo que sea explícitamente decorativo.

**Revisión H1 (apdo. 21 pto. 7 del correo):** las cifras de esta tabla y de
la de semánticos, más abajo, están recalculadas con la fórmula de
contraste WCAG (no a ojo) tras encontrar que el valor anterior de
`--color-text-muted` en claro (`#817A8D`) dejaba 3.86:1 — la propia tabla
lo daba por bueno con una cifra que no era correcta. Se corrigió ese valor
y, aplicando el mismo cálculo al resto de la paleta, aparecieron dos casos
más por debajo de AA que no estaban en la lista de 3 del correo:
`--color-danger` y `--color-info` en claro. Los cuatro están corregidos
(ver tabla de semánticos).

## Acento de marca (violeta) — tokens de ROL

No hay una escala numerada (`accent-300/400/…`): cada paso de la escala
antigua no se traducía 1:1 entre temas, así que en su lugar hay tres roles.
Úsalos siempre por su papel, nunca por "el tono que quede bien":

| Token | Oscuro | Claro | Cuándo |
|---|---:|---:|---|
| `--color-accent` | `#BF7AFD` | `#8756C7` | Estado de reposo: texto de enlace/acción, icono de marca, borde de foco |
| `--color-accent-hover` | `#CD94FF` | `#7545B8` | Hover de elementos de acento (nunca de botón primario, que tiene su propio gradiente) |
| `--color-accent-active` | `#A85BEF` | `#6936A9` | Pressed/active de elementos de acento |
| `--color-lavender` | `#B7A4E5` | `#A68BD8` | Reservado para highlights puntuales (aún sin consumidor en esta primera tanda de componentes) |
| `--color-mauve` | `#C995B6` | `#D4A8C0` | Reservado para luz ambiental puntual (ídem) |

## Bordes

| Token | Oscuro | Claro | Uso |
|---|---:|---:|---|
| `--border-subtle` | `rgba(203,192,236,.10)` | `rgba(80,62,110,.06)` | Borde exterior de ventana |
| `--border-default` | `rgba(203,192,236,.16)` | `rgba(80,62,110,.10)` | Borde normal de card/botón secundario/chip |
| `--border-hover` | `rgba(203,192,236,.28)` | `rgba(80,62,110,.20)` | Borde en hover |
| `--border-accent-strong` | `rgba(191,122,253,.90)` | `rgba(135,86,199,.65)` | Borde de card seleccionada/de confianza |

## Semánticos — independientes del violeta de marca

**Nunca** un estado semántico se vuelve violeta. "Disponible" tiene que
reconocerse de un vistazo como estado, no como acción — por eso viven en su
propia paleta, separada del acento de marca.

| Token | Oscuro | Claro | Uso |
|---|---:|---:|---|
| Token | Oscuro | Claro | Uso | Contraste sobre `--color-bg-main` (claro) |
|---|---:|---:|---|---:|
| `--color-success` / `-soft` | `#78E6A7` | `#1B7549` | Disponible, prueba completada sin avisos | 5.34:1 (antes `#238E5B`, 3.87:1) |
| `--color-warning` / `-soft` | `#F6C85F` | `#8A5E00` | Ocupado, rendimiento algo inferior al esperable | 5.35:1 (antes `#A87400`, 3.81:1) |
| `--color-danger` / `-soft` | `#FF7C8D` | `#C23A50` | Versión incompatible, error, botón cerrar en hover | 4.90:1 (antes `#D84D62`, 3.82:1) |
| `--color-info` / `-soft` | `#8EA7FF` | `#4661B8` | Informativo neutro (reservado; aún sin consumidor) | 5.38:1 (antes `#506FCC`, 4.39:1) |

Los tonos claros son más oscuros/saturados que los oscuros a propósito: un
verde o ámbar pastel pierde casi todo el contraste sobre fondo blanco. Los
cuatro valores de la columna "Claro" se revisaron y oscurecieron en esta
ronda (H1, apdo. 21 pto. 7 del correo) — mismo matiz, contraste
recalculado con la fórmula WCAG. `--color-danger` sobre botón (blanco
sobre `--color-danger`, p. ej. el botón de cerrar en hover) también se
verificó aparte: 5.23:1, sin problema.

## Fondo ambiental (`AppBackground.svelte`)

| Token | Uso |
|---|---|
| `--gradient-app-bg` | Los 4 `radial-gradient` + 1 `linear-gradient` de base. Varias fuentes de luz difusas, nunca un degradado lineal simple. |
| `--gradient-blob-1` / `--blob-1-opacity` | Forma inferior izquierda (mauve/rosa) |
| `--gradient-blob-2` / `--blob-2-opacity` / `--shadow-blob-2` | Forma inferior derecha (violeta intenso, la más grande) |

Es CSS, nunca una imagen — así se adapta a cualquier tamaño de ventana,
resolución o escalado de Windows sin rehacer ningún recurso gráfico (100 %,
125 %, 150 %, 200 %, §16.9).

## Cristal (glass) — tres niveles de blur

El **blur en px es igual en los dos temas**; lo que cambia es la opacidad de
fondo (dentro de cada `--gradient-card-*`/`--surface-*`/`--pill-bg`, ya
resuelta en el token — no hay que tocar el blur al cambiar de tema).

| Token | Valor | Dónde |
|---|---:|---|
| `--blur-pill` | `10px` | `StatusPill` variante pill |
| `--blur-card` | `14px` | `Card` default, `Button` secondary |
| `--blur-hero` | `18px` | `Card` hero |
| `--blur-sidebar` | `24px` | `Sidebar` |
| `--blur-modal` | `26px` | Diálogos/paneles (aún sin componente — reservar este token al construirlos) |

Opacidad de fondo orientativa (ya aplicada dentro de los tokens de
gradiente/superficie; tabla solo para entender el porqué de los valores):

| Elemento | Oscuro | Claro |
|---|---:|---:|
| Card normal | 70–85 % | 78–92 % |
| Sidebar | 75–85 % | 82–92 % |
| Panel principal (hero) | 75–90 % | 82–95 % |
| Glow violeta | 15–30 % | 8–16 % |

En claro se reduce la transparencia respecto a oscuro porque el exceso de
transparencia sobre blanco tiende a bajar el contraste — nunca "blanco sólido
puro", pero tampoco tan traslúcido como en oscuro.

## Los tres materiales de §16.10, con nombre

La Especificación exige tres materiales con nombre propio (`glass-chrome`,
`glass-card`, `glass-overlay`), cada uno con blur/saturación/alpha/borde/
reflejo como tokens, y un **respaldo opaco obligatorio** para cuando
`backdrop-filter` no está disponible (`@supports not (backdrop-filter)`).
Esto ya estaba implementado en los componentes de facto (cada uno con su
`backdrop-filter: blur(...)`) pero sin el nombre de material explícito ni el
fallback — se añade aquí. **Nota:** el respaldo opaco figura como criterio
de aceptación de [H3] en la Especificación (§26), pero se resuelve ya en
tokens para no tener que rediseñarlo entonces.

| Material | Dónde | Blur | Saturación | Alpha de fondo | Borde | Token de fondo (con soporte) | Token de fondo (fallback opaco) |
|---|---|---:|---:|---:|---|---|---|
| `glass-chrome` | `TitleBar`, `Sidebar`, barras de herramientas futuras | `24px` (`--blur-sidebar`) | `--material-saturate` | ver tabla de opacidad arriba | `--sidebar-border` / `--titlebar-border` | `--sidebar-bg`, `--gradient-titlebar` | `--surface-solid-chrome` |
| `glass-card` | `Card` default/hero, `Button` secondary, chips | `14–18px` (`--blur-card`/`--blur-hero`) | `--material-saturate` | ver tabla de opacidad arriba | `--border-default` / `--border-card-hero` | `--gradient-card-*` | `--surface-solid-card` |
| `glass-overlay` | `Dialog`, `Toast`, paneles laterales (Detalles técnicos) | `26px` (`--blur-modal`) | `--material-saturate` | ~94–98 % (casi opaco: es para leer, no para ver el fondo a través) | `--border-dialog` | `--gradient-dialog` | `--surface-solid-overlay` |

**Estado del respaldo opaco (C01, auditoría v3):** Dialog/Toast/Tooltip ya
lo tenían antes de esta ronda. `Card` (default y hero), `Sidebar`, `Button`
(secondary) y `StatusPill` lo tenían pendiente — corregido aquí, en los
componentes Svelte y en sus equivalentes de `maqueta-navegable.html` (donde
el mismo tratamiento visual se repite bajo varios nombres de clase por
pantalla — `.nb-hero`/`.nb-placeholder`/`.nb-summary-card`/`.nb-exec-phase`/
`.nb-result-card` son todos `glass-card` variante hero, por ejemplo). El
tooltip CSS-only de la maqueta (`[data-tt]::after`, equivalente de
`Tooltip.svelte`) también se corrigió de paso — usaba el mismo material
`glass-overlay` que `.nb-dialog` sin tener su propio respaldo, un gap
encontrado al hacer esta misma revisión, no pedido explícitamente pero de
la misma familia de problema.

`StatusPill` no encaja en ninguno de los tres materiales de la tabla — su
`--pill-bg` es un token propio, no un `--gradient-card-*`/`--gradient-dialog`
compartido — así que tiene su propio token de respaldo, **`--surface-solid-pill`**
(calculado por mezcla alfa de `--pill-bg` sobre `--color-bg-main`, mismo
método de cálculo directo que los ajustes de contraste de la ronda anterior,
no un valor a ojo).

**Cómo aplicar el respaldo opaco en un componente nuevo** (patrón, no hace
falta un mixin: son 4 líneas siempre iguales):

```css
.mi-superficie {
  background: var(--gradient-card-default); /* o el gradiente que toque */
  backdrop-filter: blur(var(--blur-card)) saturate(var(--material-saturate));
  -webkit-backdrop-filter: blur(var(--blur-card)) saturate(var(--material-saturate));
}
@supports not (backdrop-filter: blur(1px)) {
  .mi-superficie {
    background: var(--surface-solid-card);
  }
}
```

La barra de título, por decisión de §16.13, usa **siempre** el plano opaco
(nunca el cristal) para fundirse con el resto de la ventana — con la
salvedad de la "Propuesta definitiva" aprobada, que en su lugar usa
`--gradient-titlebar` (ver el comentario en `TitleBar.svelte` y el registro
de cambios de `README.md`: es una divergencia de spec marcada para aprobar,
no un olvido).

**Texto denso sobre material opaco** (§16.10: "el cristal es para
superficies, no para lectura prolongada"): Detalles técnicos, XML, tooltips
largos y el diálogo de instrucciones deben ir sobre `--color-bg-elevated`
(ya es la superficie opaca del sistema, no hace falta un token nuevo) en vez
de sobre cualquier `--gradient-card-*`/`--gradient-dialog` translúcido.

## Diálogos, overlay y formularios

Tokens nuevos para `Dialog.svelte`, `Toast.svelte`, `TextField.svelte` y
`VerificationCode.svelte` (ver `README.md` § Componentes nuevos —
corregido en la auditoría v3, C05: este párrafo citaba un
`CodeInput.svelte` que nunca ha existido; el componente entregado siempre
se llamó `VerificationCode.svelte`).

| Token | Uso |
|---|---|
| `--overlay-scrim` | Fondo oscurecido detrás de un diálogo modal (clic fuera = cerrar, §16.14) |
| `--gradient-dialog` / `--border-dialog` / `--shadow-dialog` | Superficie del propio diálogo (material `glass-overlay`) |
| `--surface-field` / `-hover` / `-disabled` | Fondo de un campo de texto/número en reposo, hover y deshabilitado |
| `--border-field` / `-hover` / `-focus` / `-error` | Borde del campo en cada estado — `-focus` y `-error` son siempre `--color-accent`/`--color-danger`, nunca un tono nuevo |
| `--shadow-field-focus` / `-error` | Halo de foco/error alrededor del campo (además del borde: nunca solo color, §16.10) |
| `--color-field-placeholder` | Texto de ejemplo dentro de un campo vacío |

**Medidas de diálogo (C03, auditoría v3):**

| Token | Valor | Uso |
|---|---:|---|
| `--dialog-width-sm` | `380px` | La mayoría de diálogos — avisos, confirmaciones |
| `--dialog-width-md` | `460px` | El de aceptación (resumen del solicitante + código + casilla de confianza) — más contenido que un aviso simple |
| `--dialog-width-lg` | `620px` | Reservado, sin uso todavía en H1 |

Antes literales (`.nb-dialog-sm/-md/-lg { max-width: 380px/460px/620px; }`
en `Dialog.svelte`) y, en la maqueta, un cuarto valor propio (430px) que no
coincidía con ninguno de los tres — un único ancho para todos sus
diálogos, sin la distinción sm/md que el propio `Dialog.svelte` ya
documentaba ("la mayoría son sm, el de aceptación es md"). Corregido: la
maqueta usa `--dialog-width-sm` por defecto y `--dialog-width-md` solo
para `[data-overlay="peer-accept"]`, igual que el componente real.

## Gráficas — colores de serie (§16.7, §19.4)

Los colores de A→B y B→A **no** son semánticos (no significan "bien/mal") ni
usan el violeta de marca (que es acción, no dato) — tienen sus propios
tokens, distinguibles del resto de la paleta y entre sí también sin color
(trazo continuo vs discontinuo, definido en el propio SVG del componente de
gráfica, no aquí):

| Token | Uso |
|---|---|
| `--color-chart-a-b` | Serie A→B (trazo continuo) |
| `--color-chart-b-a` | Serie B→A (trazo discontinuo) |
| `--color-chart-grid` | Líneas de cuadrícula, muy tenues |
| `--color-chart-axis-label` | Texto de ejes y ticks |
| `--color-chart-warmup-zone` / `-cooldown-zone` | Sombreado de las zonas de calentamiento/enfriamiento (§16.4) |
| `--color-chart-gap` | Hueco de muestra perdida (§8.5: "las muestras perdidas se marcan como huecos, no se interpolan") — se dibuja como segmento discontinuo con este tono, nunca uniendo los puntos como si no faltara nada |

Esto cubre los tokens; el propio componente de gráfica (SVG, ejes
autoescalados, tooltip accesible por teclado) es trabajo de H2 según la
Especificación (§16.7) y no se ha construido — ver `PANTALLAS-H1.md` para el
estado provisional que sí lleva la pantalla de Ejecución en esta entrega.

## Botón primario — el único botón "con color"

| Token | Oscuro | Claro |
|---|---:|---:|
| `--gradient-button-primary` | `#D69AFF → #BF7AFD → #AD63F1` | `#8A5EC0 → #7F51B8 → #6D3CA6` |
| `--gradient-button-primary-hover` | (aclara) | `#7F51B8 → #7440B5 → #5F3391` (oscurece) |
| `--color-button-primary-text` | `#24152F` | `#FFFFFF` |

En claro el texto pasa a blanco y el gradiente se oscurece: el violeta claro
que luce perfecto sobre fondo oscuro pierde contraste sobre fondo blanco, así
que en claro se usa un violeta más saturado/oscuro en vez de intentar
mantener exactamente los mismos tres tonos.

**Auditoría v3 (C02):** el primer valor que se usó en claro
(`#9D72D3 → #8756C7 → #7440B5`, hover `#A97FDC → #9565D0 → #8354C4`) no
cumplía AA con texto blanco en varios puntos del degradado —
verificado extremo por extremo, no solo con el fondo medio:

| Color | Contraste con blanco |
|---|---:|
| `#9D72D3` (extremo claro, normal) | 3.63:1 |
| `#A97FDC` (extremo claro, hover) | 3.11:1 |
| `#9565D0` (medio, hover) | 4.16:1 |

Los tres por debajo de 4.5:1. Recompuesto para que **todos** los puntos de
ambos degradados cumplan AA (4.66:1–8.86:1 según el punto). Efecto
secundario intencional: en claro, el hover ahora **oscurece** en vez de
aclarar (en oscuro sigue aclarando, ahí no hay problema de contraste) — la
señal de hover la sigue dando el mismo `translateY(-1px)` + aumento de
sombra que ya existía, así que sigue siendo perceptible sin depender solo
del cambio de color. Mismo hallazgo y mismo criterio aplicado a
`--gradient-nav-active` (activo de sidebar, chip de tema): su tono medio
(`#9565D0`, 4.16:1) también incumplía — sustituido por
`--color-accent`/`--color-accent-active` (`#8756C7 → #6936A9`), que ya
cumplían y evitan un tono nuevo.

## Tipografía

```
--font-ui:   "Segoe UI Variable", "Segoe UI", system-ui, -apple-system, sans-serif;
--font-mono: "Cascadia Mono", "Consolas", ui-monospace, monospace;
```

Fuente **nativa de Windows**, nunca una web font descargada para la UI —
coherente con §16.14 (aspecto nativo, no web) y §24.3 (todo local, sin
depender de Internet). `--font-mono` está reservado para IPs/valores
numéricos densos si en alguna pantalla futura hiciera falta alinear
columnas (de momento `DeviceCard` usa `font-variant-numeric: tabular-nums`
con la fuente normal, que ya alinea dígitos sin cambiar de tipografía).

| Token | Valor |
|---|---:|
| `--font-size-xs` | `12px` |
| `--font-size-sm` | `13px` |
| `--font-size-md` | `14px` |
| `--font-size-lg` | `16px` |
| `--font-size-xl` | `20px` |
| `--font-size-2xl` | `28px` |

| Token (C03, auditoría v3) | Valor | Uso |
|---|---:|---|
| `--font-size-2xs` | `11.5px` | Metadatos densos: IP de equipo, hints de campo, badges de demo, pestañas de perspectiva de Ejecución. Antes era un `11.5px` literal repetido en 6 componentes Svelte y una decena de sitios de la maqueta, sin token — unificado aquí. |

| Token | Peso | Uso |
|---|---:|---|
| `--font-weight-body` | 450 | Texto corrido |
| `--font-weight-control` | 550 | Botones, controles |
| `--font-weight-subtitle` | 600 | Subtítulos |
| `--font-weight-title` | 700 | Títulos, nombre del equipo |

## Espaciado (escala de 4px)

`--space-1` a `--space-12` = `4px … 48px` (ver `tokens.css` para la lista
completa). En la interfaz aprobada los valores que más se repiten son 8, 12,
16, 24 y 32px — antes de inventar un espaciado nuevo, comprueba si uno de
estos ya sirve.

## Radios — redondeado moderno, nunca caricaturesco

| Token | Valor | Uso |
|---|---:|---|
| `--radius-xs` | `6px` | Controles diminutos |
| `--radius-sm` | `10px` | Botones pequeños, inputs, chips de icono |
| `--radius-md` | `14px` | Botones, chips, ítems de sidebar |
| `--radius-lg` | `18px` | Cards |
| `--radius-xl` | `24px` | Card principal (hero), modales |
| `--radius-pill` | `999px` | Pills de estado |

## Motion

```
--ease-standard: cubic-bezier(0.2, 0, 0, 1);
--duration-fast:  120ms;  /* hover de controles pequeños (winctrl, nav) */
--duration-base:  160ms;  /* la mayoría de transiciones */
--duration-slow:  220ms;  /* box-shadow de card (más lento que el resto) */
```

**Duraciones propias de una animación concreta (C03, auditoría v3):** las
tres de arriba son para transiciones de hover/press/foco — no fuerces en
ellas una animación con su propio ritmo (spinner, pulso, pausa antes de
mostrarse). Existían como literales sueltos en varios sitios, algunos
además desincronizados entre `maqueta-navegable.html` y el componente
Svelte real (`--duration-progress-indeterminate` era 1.1s en la maqueta y
1.3s en `ProgressBar.svelte` — la maqueta se corrigió para igualar al
componente, que es la fuente real):

```
--duration-tooltip-delay:          250ms;  /* retardo antes de mostrar Tooltip.svelte */
--duration-emphasis-pulse:         900ms;  /* pulso de "sección destacada" en Inicio */
--duration-spinner:                0.8s;   /* giro del spinner de ChecklistItem — x2 con prefers-reduced-motion, vía calc(), no un segundo token */
--duration-progress-indeterminate: 1.3s;   /* barra deslizante de ProgressBar.svelte cuando `indeterminate` */
```

**`prefers-reduced-motion` de la barra indeterminada:** la maqueta solo
ralentizaba esta animación (2.4s) bajo `prefers-reduced-motion`; el
componente real la **para** del todo (barra fija al 100%, opacidad 0.5) —
una tercera desincronización, corregida en la maqueta para igualar el
comportamiento real, no solo su duración.

Reglas:

- Hover de card/botón: `transform: translateY(-1px)` — **nunca** más de 1px.
- Pressed: `transform: scale(.985)`.
- Todo componente interactivo envuelve sus transiciones/transforms en
  `@media (prefers-reduced-motion: reduce) { transition: none; transform:
  none; }` (ver cualquier componente de `src/lib/components/` como
  ejemplo — todos lo hacen ya).
- Nada de esto es aún la "animación atrevida" que pide §16.10 para
  transiciones de pantalla o la revelación del resultado — eso es
  coreografía a nivel de pantalla/ruta, no de átomo, y queda pendiente para
  cuando se construyan esas vistas.

## Foco y accesibilidad

- `--color-focus-ring` = alias de `--color-accent` en ambos temas. Todo
  elemento interactivo lleva `:focus-visible { outline: 2px solid
  var(--color-focus-ring); outline-offset: 2px; }` (o `-2px` cuando el
  control está en el borde de un contenedor, como los `winctrl`).
- Estados (éxito/aviso/error) siempre van con icono/punto **+ texto**, nunca
  solo color (`StatusPill`, `DeviceCard`).
- Todo control interactivo es un `<button>`/`<a href>` real, nunca un `div`
  con `onclick` a mano — **con una excepción documentada**: `Card.svelte`,
  cuando recibe `onclick`, renderiza `<div role="button" tabindex="0">` con
  Enter/Espacio gestionados en el propio componente (nunca un `<button>`
  real), porque `DeviceCard` mete un `<button>` de favorito dentro y
  `<button>` no puede contener otro `<button>` — es HTML inválido y rompía
  el layout (bug real, corregido en la revisión H1). Esta descripción
  sustituye a una anterior que decía que `Card` "decide entre `<button>` o
  `<div>`" — desde esa corrección, la rama interactiva es siempre `<div
  role="button">`, nunca `<button>` (corregido en la auditoría v3, C05,
  que señaló que esta página seguía describiendo el comportamiento
  antiguo). Fuera de ese caso concreto, la regla general sigue siendo:
  siempre un elemento nativo.

## Reglas de "aspecto nativo, no web" (§16.14) ya aplicadas

Estas reglas viven en cada componente, pero se listan aquí para que
cualquier componente **nuevo** las siga sin tener que releer la
especificación entera:

- `cursor: default` en botones, tarjetas clicables, pestañas, chips — nunca
  `pointer` salvo en un enlace que abre algo externo.
- Enlaces sin subrayar, ni en reposo ni en hover.
- `user-select: none` en controles; `text` solo en campos editables y
  valores copiables (IPs, huellas, cifras de resultado).
- Sin `<select>` nativo, `alert()`/`confirm()`/`prompt()` del navegador.
- Sin emojis como iconos — siempre un trazo SVG propio (`icons.ts`).
