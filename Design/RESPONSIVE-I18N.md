# Responsive + i18n para H1 (auditoría v3, A08 + corrección tras revisión de código)

Cierra el punto 18 del correo (§16.9 responsive, §4 i18n), que la versión
anterior de este entregable dejaba fuera de alcance de H1 por completo. El
propio correo pedía explícitamente una **matriz de reglas y ejemplos, no
una captura por cada combinación** — eso es lo que hay aquí: reglas +ejemplos
de texto es/en +una tabla de qué probar, en vez de decenas de PNG. Las
capturas que sí se han tomado (ver más abajo) fueron para verificar cada
regla al implementarla, no son el entregable en sí.

Todo lo de este documento está implementado y verificado en
`maqueta-navegable.html` (reglas CSS + JS reales, no solo descritas aquí) y
comprobado con Playwright en `/tmp/verify-a08.mjs` y
`/tmp/verify-a08b-responsive.mjs` (ver "Verificación" al final) — cero
errores de consola en ninguno de los anchos/casos siguientes.

**Corrección posterior (revisión de código del desarrollador, con
`Especificacion.md` ya completo):** la tabla del §1 de más abajo usaba
umbrales "de aspecto razonable" (1048px/1450px de viewport, ninguno
llamado por su nombre de tramo) porque §16.9 no estaba disponible completo
en la ronda de A08. Ya lo está — §16.9 nombra los tres tramos exactos:
**compacta <1000px, estándar 1000–1400px, amplia >1400px** de ventana, con
comportamientos concretos por tramo. La tabla de §1 queda reescrita contra
esa cita literal, no contra una interpretación — con el nombre de tramo,
el porqué del número de viewport exacto, y (para amplia) las dos cosas que
antes no estaban reflejadas: el indicador de las dos direcciones a la vez
en Ejecución, y por qué el panel lateral de Detalles técnicos sigue sin
construirse.

## 1. Reglas de tamaño de ventana

`.nb-window` usa un modelo de ancho fijo-y-luego-encogible (`width: 1160px`
con `max-width: calc(100vw - 48px)` por debajo de ~1208px de viewport), así
que estas reglas usan **ancho de viewport** como aproximación del ancho de
ventana — es una app de escritorio de ventana única, no hay layout
multi-panel que dependa del tamaño real de la ventana en vez del viewport,
así que la aproximación es válida para esta ronda. Queda anotado por si en
H2 aparece algo (p. ej. un panel lateral redimensionable) donde deje de
serlo.

| Tramo (§16.9) | Rango de ventana | Rango de viewport en esta maqueta | Qué cambia | Regla CSS |
|---|---|---|---|---|
| **Compacta** | < 1000px | < 1048px (viewport − 48 = ancho de ventana en el tramo encogible, así que el cruce con 1000 es 1048 exacto, no una aproximación) | Las cuatro cosas que pide §16.9: sidebar **solo iconos** (40×40px, sin etiqueta de texto, con tooltip `data-tt`); tarjetas de equipo de Inicio en **una columna** (`flex-basis: 100%`); gráfica de Ejecución a **200px de alto** (130px es el valor por defecto); direcciones de Ejecución **apiladas** (`.nb-exec-link` en columna). | `@media (max-width: 1048px) { .nb-sidebar{width:52px} .nb-navitem{width:40px;height:40px;gap:0} .nb-navitem span{display:none} .nb-main{padding:20px 20px} .nb-device-row .nb-card{flex-basis:100%} .nb-exec-link{flex-direction:column;align-items:stretch} .nb-exec-node{width:100%} }` + `.nb-chart{height:200px}` (regla aparte, ver comentario junto a `.nb-chart` en el CSS — tiene que ir después de la regla base para ganar por cascada). |
| **Estándar** | 1000–1400px | ~1048–1450px | Sin comportamientos propios — es el estado por defecto, el que queda cuando no aplica ni compacta ni amplia. Sidebar completa, tarjetas en fila (si caben), gráfica a 130px, direcciones no apiladas. | Sin media query. |
| **Amplia** | > 1400px | > 1450px (viewport − 48 > 1400 ⟹ viewport > 1448; se usa 1450 por el mismo margen redondo que ya tenía la regla anterior) | Las dos cosas que pide §16.9: **A→B y B→A lado a lado** — indicador compacto con las dos direcciones de Ejecución visibles a la vez (`#exec-both-directions`), además de la flecha única que sigue mostrando la dirección activa (la prueba sondea una dirección detrás de otra, nunca las dos a la vez, §12); **Detalles técnicos como panel lateral** — **no implementado**, ver nota de alcance más abajo. La ventana en sí sigue creciendo con el viewport en vez de fijarse en un número (antes se quedaba en 1400px exactos para siempre — nunca llegaba a superar 1400, así que "amplia" en sentido estricto no existía). | `@media (min-width: 1450px) { .nb-window{width:calc(100vw - 48px);max-width:1800px} #exec-both-directions{display:flex} }` |

**Nota de alcance — Detalles técnicos como panel lateral, no implementado
a propósito:** el contenido de Detalles técnicos es H2 (§16.6 — el botón
sigue deshabilitado en Resultado, con el aviso "Hito H2", ver tabla B del
README, fila B04). Construir un panel lateral vacío, sin ese contenido
dentro, sería una cáscara visual sin nada real que mostrar — no una
pantalla más fiel a la spec, solo una más grande y vacía. Queda como
dependencia explícita para quien construya H2 (fila B05 del README,
actualizada), no como algo evitado por falta de tiempo.

**Nota de alcance — 800×600 y 1100×760:** siguen sin necesitar una regla
CSS propia distinta de las de la tabla — 800×600 cae dentro del rango
compacta de aquí arriba (sus cuatro comportamientos ya cubren ese ancho) y
1100×760 es el caso estándar por defecto. Un sistema de breakpoints más
fino (tablet, ventana muy panorámica, etc.) no está pedido y no se ha
diseñado — quedaría para H2 si hiciera falta.

## 2. Previsión de i18n (es/en) para las pantallas H1

Todo el texto de `maqueta-navegable.html` está en español, hardcodeado —
como ya señalaba este README (`TitleBar.svelte`: "textos en español —
sustituir por las claves i18n reales, §4"). No existe todavía un sistema de
claves i18n real ni un segundo idioma implementado; lo que sigue es
**previsión de diseño** para que el inglés (o cualquier idioma más largo
que el español) no rompa el layout cuando llegue ese sistema — no una
traducción para producción.

Tabla de las cadenas más representativas del recorrido H1, con el inglés
propuesto solo a efectos de medir longitud y señalar riesgo — la copia
final en cada idioma es una decisión de producto/redacción, no de este
sistema de diseño:

| Cadena (es) | Propuesta (en) | Dónde | Riesgo de layout |
|---|---|---|---|
| Inicio / Historial / Ajustes | Home / History / Settings | `Sidebar.svelte`, ítems de nav | Bajo en texto completo; en el modo solo-iconos (<1048px) no hay texto visible, así que ahí el riesgo es nulo por diseño. |
| Prueba en curso | Test in progress | Tooltip de nav bloqueada (`data-tt`) | Bajo — el tooltip mide su propio contenido (`content: attr(data-tt)`), no tiene ancho fijo. |
| Cancelar prueba | Cancel test | Botón secundario, ejecución/preparación | Bajo — `Button.svelte`/`.nb-btn-secondary` no tienen ancho fijo, crecen con el contenido. |
| Coincide y aceptar | Confirm match | Botón primario, diálogo de aceptación | Bajo, mismo motivo. Ejemplo elegido a propósito: "Coincide y aceptar" es más largo que "Aceptar" (variante peer conocido) — si el botón tuviera ancho fijo esto ya habría fallado en español; al no tenerlo, el inglés tampoco es un problema nuevo. |
| Dirección 1 de 2 | Direction 1 of 2 | Eyebrow sobre la fase de ejecución (`nb-eyebrow`) | Bajo — texto corto en los dos idiomas. |
| Probando envío desde DESKTOP-DANIEL | Testing upload from DESKTOP-DANIEL | Título de fase, ejecución | **Medio** — interpola el nombre del equipo, que el usuario elige libremente (§7). Un nombre de equipo largo ya es un riesgo independiente del idioma; no se ha probado con nombres largos en esta ronda (fuera del alcance de A08, que es de idioma/tamaño de ventana, no de longitud de contenido de usuario) — señalado aquí para que quede constancia, no resuelto. |
| Tiempo restante aproximado: 24 s | Estimated time remaining: 24s | Bajo la barra de progreso de ejecución | Bajo — una sola línea, sin ancho fijo. |
| Calentando la conexión con SERVER-01… | Warming up the connection to SERVER-01… | Fase de calentamiento, ejecución | Igual que "Probando envío desde…": interpola nombre de equipo, mismo riesgo medio señalado una vez. |
| Reconectando… / SERVER-01 necesita una acción | Reconnecting… / SERVER-01 needs action | Nota de estado degradado, ejecución | Bajo — `.nb-exec-state-note` es un bloque de ancho completo, no una etiqueta estrecha. |
| Comprobar de nuevo | Check again | Botón de acción dentro de la nota de estado | Bajo, mismo motivo que el resto de botones. |
| Retransmisiones | Retransmissions | Etiqueta de dato, grid de Resultado (`.nb-result-stat`) | **Medio** — "Retransmissions" (15 caracteres) es más largo que "Retransmisiones" (15, casi igual en este caso concreto) pero el patrón general de este grid (etiquetas cortas en español, p. ej. "Pico") puede tener homólogos en inglés más largos ("Peak throughput" vs. "Pico") — la celda no tiene ancho fijo, pero no se ha probado el grid completo con las cuatro etiquetas en su versión inglesa más larga. Señalado, no verificado en esta ronda. |
| 89% de 10 Gbit/s | 89% of 10 Gbit/s | Cifra principal de Resultado (`.nb-result-headline`) | Bajo — tipografía grande de una sola línea, sin contenedor de ancho fijo alrededor. |
| Prueba cancelada / Resultado incompleto | Test cancelled / Incomplete result | Título del panel de Resultado | Bajo. |
| El código no coincide | The code doesn't match | Título de aviso (`#aviso-title`) | Bajo — mismo patrón que el resto de títulos de diálogo. |

**Patrón general (por qué el riesgo es bajo en casi todo):** ningún
componente de texto de esta ronda usa un ancho fijo en píxeles para
contener texto — botones, etiquetas, títulos y notas crecen con su
contenido (regla ya vigente desde antes de A08, confirmada aquí a
propósito por si el inglés la hubiera roto en algún sitio; no la ha
roto). Las dos únicas excepciones señaladas arriba (nombre de equipo
interpolado, etiquetas del grid de Resultado) son riesgos de **contenido
dinámico o específico de una copia futura**, no de diseño de layout — se
dejan anotadas para quien construya el catálogo i18n real (H2/H3, fuera
de §4 tal y como está definido en la propia spec), no resueltas aquí
porque resolverlas requeriría decidir textos en inglés reales, que no es
una decisión de este sistema de diseño.

## 3. Correcciones encontradas al verificar responsive (A08)

Dos fallos reales, encontrados al comprobar la regla de 800×600 de la
tabla del §1, no al leer la spec — quedan documentados aquí porque son
parte de por qué hace falta la matriz de pruebas del §4, no solo las
reglas CSS:

- **Leyenda de la gráfica de Resultado se recortaba a 800px de ancho.**
  `.nb-chart-legend` no tenía `flex-wrap`, así que la entrada más larga
  ("SERVER-01 → DESKTOP-DANIEL (patrón discontinuo, §16.7/§19.4 — no solo
  color)") se salía del contenedor en vez de bajar de línea. Corregido:
  `.nb-chart-legend { display: flex; flex-wrap: wrap; row-gap: 4px;
  column-gap: 18px; ... }`.
- **El scroll no se reiniciaba al cambiar de pane.** Todos los panes viven
  dentro de un único contenedor con scroll (`.nb-main { overflow: auto }`),
  mostrados/ocultados con `display`, así que el `scrollTop` se conservaba
  del pane anterior. A tamaños donde hace falta scroll a menudo (800×600),
  esto podía aterrizar al usuario a media pantalla de un pane que se acaba
  de abrir por primera vez — confirmado con el "Ver desde:" de Ejecución
  quedando fuera de vista tras desplazar Inicio y navegar. Corregido en
  `navigateTo()`: `main.scrollTop = 0` en cada cambio de pane.

## 4. Matriz de pruebas combinada

Esto es lo que pide el correo en vez de una captura por combinación: qué
comprobar, cruzando ancho de ventana × idioma × escala de texto. "✓" =
cubierto y verificado en esta ronda (comportamiento implementado y
comprobado con Playwright, aunque el idioma/escala en sí sea solo
previsión de diseño, no i18n real todavía); "—" = fuera de alcance de H1,
con motivo.

| | 800×600 (compacta) | 1100×760 (estándar) | ≥1450px (amplia) | Solo-iconos (<1048px, se solapa con 800×600) |
|---|---|---|---|---|
| **Español (actual)** | ✓ apilado + scroll reiniciado | ✓ caso base | ✓ ventana crece con el viewport, >1400px real; indicador de dos direcciones visible | ✓ sidebar 52px, tooltips |
| **Inglés (previsión)** | — sin sistema i18n real; riesgo de layout evaluado en §2 (bajo, salvo las dos excepciones señaladas) | — mismo motivo | — mismo motivo | — mismo motivo |
| **Texto 115%** | — funcionalidad de escalado es H2 (`AUDITORIA_DISENO_V3.md` lo sitúa así); ver nota de diseño de más abajo | — | — | — |
| **Texto 130%** | — mismo motivo | — | — | — |

Las cuatro celdas "✓" son las que existen de verdad en esta ronda y están
cubiertas por `/tmp/verify-a08.mjs` (regresión programática) más las
capturas de comprobación visual tomadas durante A08. Las celdas "—" de
idioma/escala no son huecos silenciosos: están resueltas al nivel que le
corresponde a H1 (previsión de diseño, no implementación), con el motivo
explícito en cada fila en vez de dejarlas vacías sin más.

## 5. Previsión de diseño para escala de texto 115%/130%

La funcionalidad de escalado de texto es H2 (no se implementa aquí), pero
el correo pide que el **diseño** ya la tenga en cuenta ahora para no tener
que rehacer nada al llegar H2. Esto es diseño, no implementación:

- **El gap real está en los tokens, no en los componentes.**
  `DESIGN_TOKENS.md` define todos los `--font-size-*` en `px` fijos
  (`--font-size-xs: 12px` … `--font-size-2xl: 28px`). Un escalado real de
  115%/130% necesita que estos tokens respondan a la preferencia de
  tamaño de texto del sistema/la app — lo natural es migrarlos a `rem`
  (con el `font-size` raíz como único punto de control) en vez de
  multiplicar cada valor a mano. Esto es un cambio en `tokens.css`, no en
  cada componente individual — ningún componente de esta ronda lee un
  tamaño de fuente fuera de estos tokens (regla 1 del README, "cero
  literales"), así que el cambio queda centralizado cuando llegue H2.
- **Ningún contenedor de texto de esta ronda tiene alto fijo.** Confirmado
  al revisar los mismos sitios de la tabla del §2: botones, tarjetas,
  diálogos y notas de estado crecen con su contenido (`padding` +
  `min-height` como mucho, nunca `height` fijo). Un texto un 30% más alto
  empuja el layout en vez de recortarse — es el comportamiento correcto
  para 130%, y ya está vigente sin cambios adicionales.
- **Excepción real a vigilar en H2:** los iconos de tamaño fijo en `px`
  (`Icon.svelte`, prop `size`) no escalan con el texto porque no están
  pensados para hacerlo — es intencional (un icono de 19px sigue siendo
  legible a cualquier escala de texto), pero el espaciado alrededor de un
  icono junto a texto escalado sí puede necesitar revisión visual en H2
  (p. ej. `.nb-navitem` con icono + etiqueta, cuando la etiqueta crece un
  30% y el icono no). Señalado para la ronda de H2, no resuelto aquí.
- **La leyenda de la gráfica (§3) y el patrón de "sin ancho fijo" del §2
  ya cubren la parte de layout de texto largo/grande** — 130% de texto es,
  para el propósito de romper o no romper el layout, un caso similar a
  "texto en un idioma más largo", y las mismas reglas que lo protegen
  (crecer con el contenido, `flex-wrap`, sin alturas fijas) protegen
  también la escala de texto. No hace falta una regla nueva de layout para
  esto — hace falta la migración de tokens a `rem` para que el escalado
  *llegue* a los componentes, que es el trabajo real de H2.

## Verificación

- `node scripts/verify-tokens.mjs` — sin regresión, cero colores
  literales.
- `/tmp/verify-a01.mjs` … `/tmp/verify-a07.mjs` — el recorrido completo y
  todos los caminos alternativos de rondas anteriores, sin errores de
  consola tras los cambios de A08 (el reinicio de scroll y el breakpoint
  de icono-solo no tocan ninguna de esas rutas).
- `/tmp/verify-a08.mjs` — dedicado a la ronda A08: ancho de sidebar/estado
  de etiquetas por debajo de 1048px, ancho de `.nb-window` por encima de
  1450px (comentario actualizado tras la corrección: ahora se comprueba
  que supere 1400px, no que se quede fijo en él), estado base a 1100×760,
  `flex-wrap` y no-desbordamiento de la leyenda de la gráfica a 800×600, y
  reinicio de `scrollTop` entre panes. Los cinco casos pasan, cero errores
  de consola/página.
- `/tmp/verify-a08b-responsive.mjs` (nuevo, corrección tras revisión de
  código con `Especificacion.md` completo) — dedicado a los tres tramos
  exactos de §16.9: en compacta (900px), sidebar solo-iconos, gráfica a
  200px de alto, `.nb-exec-link` en columna y tarjetas de Inicio a
  `flex-basis: 100%`; en estándar (1100×760), sidebar completa y ventana
  entre 1000-1400px; en amplia (1700px), ventana por encima de 1400px de
  verdad (no fija en 1400), `#exec-both-directions` visible, direcciones
  no apiladas, y el indicador de las dos direcciones sincronizado con
  `data-dir` tanto al aterrizar en Ejecución como tras el flujo real de
  "Simular: reconexión automática" (que sí llama a `setExecDirection('ba')`
  desde código de producto, no solo desde un atajo de test); y el cruce
  exacto de compacta en 1048px/1049px de viewport. Los doce casos pasan,
  cero errores de consola/página.
