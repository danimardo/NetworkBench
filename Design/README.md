# NetworkBench — sistema de diseño (v4, cierre de auditoría H1)

Entregable para el desarrollador de frontend: tokens, componentes Svelte 5 y
las reglas de implementación necesarias para construir el resto de la
interfaz sin tener que decidir nada de diseño por su cuenta. Si algo no está
aquí, en `DESIGN_TOKENS.md`, `FLUJOS.md` o `PANTALLAS-H1.md`, es que de
verdad falta por decidir — no que se haya dado por sobreentendido.

Esta versión responde a `AUDITORIA_DISENO_V3.md` (la revisión que hizo el
programador sobre la entrega anterior) punto por punto: cierra los huecos
A01–A09 marcados como urgentes para poder dar H1 por cerrado, aplica las
correcciones transversales C01–C05, deja un compromiso de alcance para
B01–B12 (H2/H3) y recoge en un anexo las decisiones D01–D04 que la propia
auditoría señala como no correspondientes al diseñador (secuencia
PAIRING/PRECHECK, degradado de la barra de título, `Historias.md`,
alcance por hitos) — resueltas por quien corresponda, no inventadas aquí.
Ver la sección **"Cierre de la auditoría v3"** más abajo para el detalle
completo, entrada por entrada.

> **`Historias.md`**: sigue sin haber llegado. La propia auditoría lo marca
> como decisión de producto (D03), no de diseño — no se ha rellenado por
> intuición. Todo lo de aquí sigue construido contra `Especificacion.md`
> exclusivamente.

> **Corrección posterior a A01–A09 (revisión de código, `Especificacion.md`
> completo):** el desarrollador hizo una segunda pasada, esta vez a nivel
> de código sobre lo ya construido, y confirmó que A01–A09 están bien
> resueltos ("buen trabajo"). Encontró un problema distinto: varios sitios
> de este entregable decían que una sección de la spec "no estaba
> disponible en esta ronda" y rellenaban el hueco con una interpretación
> propia — pero `Especificacion.md` ya estaba completo, y las cifras usadas
> no coincidían con las reales. Cinco puntos concretos, los cuatro primeros
> verificables y los cinco corregidos:
>
> 1. Umbral de veredicto de Resultado (§13.2): 85/60/60, no 85/50/50 —
>    corregido, ver "Resultado — regla de veredicto" más abajo.
> 2. Puntos de corte responsive (§16.9): compacta <1000px / estándar
>    1000–1400px / amplia >1400px, no <1048px/≥1450px sin nombrar el
>    tramo — corregido, incluyendo lo que faltaba de amplia (dos
>    direcciones a la vez, panel lateral de Detalles técnicos), ver
>    `RESPONSIVE-I18N.md`.
> 3. `NB-CONN-004` (AVISOS.md): tenía título/descripción sin rellenar,
>    §21.2 ya los define — corregido.
> 4. `NB-FW-002` (AVISOS.md): el código ya era correcto, pero sus acciones
>    y título eran un genérico "Cerrar/Reintentar" en vez de los de §14.4
>    — corregido.
> 5. Cifra principal de Resultado Nivel 1 (§16.5): pregunta abierta, no un
>    error — resuelta por confirmación directa del cliente: la cifra
>    grande es la velocidad, el % pasa a texto de apoyo.
>
> Ninguna de las cinco es una interpretación nueva: las cuatro primeras
> son citas literales de `Especificacion.md` reemplazando una
> interpretación provisional, y la quinta es una decisión de producto
> confirmada, no supuesta.

Dirección visual aprobada: **"Graphite Violet"**, en dos temas —

- **Oscuro** ("Graphite Violet Dark"): grafito + azul noche +
  violeta/lavanda/malva ambiental.
- **Claro** ("Pearl Lavender Light"): blanco perla + gris lavanda + el mismo
  violeta como acento.

Ninguno de los dos es "el modo por defecto" — el modo de tema por defecto
de la aplicación es **Sistema** (sigue `prefers-color-scheme`; ver
"Cómo funciona el tema" más abajo). "Oscuro" solo era el primero de los dos
en describirse aquí; decirlo así en versiones anteriores de este README
daba a entender lo contrario, señalado en la auditoría (C05) — corregido.

## Índice de lo que hay que leer, en orden

1. **`maqueta-navegable.html`** — ábrelo en el navegador (doble clic, sin
   instalar nada). Es la prioridad número uno del correo: reproduce el
   recorrido completo, en los dos sentidos, con sus caminos alternativos.
2. **`FLUJOS.md`** — el mapa de estados del recorrido completo (máquina de
   estados de §10.1, recorrido feliz y caminos alternativos), con qué ve
   cada uno de los dos equipos en cada momento y a qué pantalla de la
   maqueta corresponde. Léelo junto a la maqueta, no en su lugar.
3. **Este README** — arquitectura, instalación, cómo funciona el tema, API
   de cada componente, índice de pantallas/estados (hecho vs. pendiente,
   mapeado a `Especificacion.md`), y los 7 puntos corregidos de la revisión.
4. **`PANTALLAS-H1.md`** — los estados de Inicio y conexión manual que no
   tienen maqueta propia porque son variaciones de contenido sobre una
   pantalla que sí está maquetada (buscando, ninguno encontrado,
   descubrimiento desactivado, IPv6/hostname…).
5. **`DESIGN_TOKENS.md`** — referencia completa de cada token (valor claro +
   oscuro + para qué sirve), incluidos los tres materiales de cristal con
   nombre (§16.10) y los tokens de diálogo/formulario/gráfica nuevos de esta
   ronda.
6. **`RESPONSIVE-I18N.md`** — reglas de tamaño de ventana (800×600,
   1100×760, sidebar solo-iconos, >1400px), previsión de i18n es/en y de
   escala de texto 115/130% para las pantallas H1 (auditoría v3, A08). En
   forma de matriz de reglas + ejemplos, no una captura por combinación.
7. **`AVISOS.md`** — catálogo de avisos de H1: códigos, textos es/en y las
   correcciones de la auditoría v3 (A09) — versión incompatible con el
   equipo a actualizar, "Ver detalles" en código-no-coincide, código de
   diagnóstico correcto para fallo de puertos, sin `ntttcp.exe` en ninguna
   pantalla visible.
8. El código en sí — todos los componentes están comentados en los puntos
   donde hay una decisión de diseño no obvia.

## `maqueta-navegable.html` — para enseñar y aprobar el diseño

Sigue sin ser parte del código a integrar — es una maqueta HTML/CSS/JS
autocontenida (un fichero, sin dependencias). Lo que cubre en esta ronda:

- **Recorrido completo, feliz, en un solo equipo que representa los dos
  roles** (iniciador y receptor) — Inicio → selector de equipo o conexión
  manual → emparejamiento (solo la primera vez) → esperando aceptación →
  preparación (checklist) → ejecución (dos direcciones, con gráfica) →
  resultado, con auto-avance para poder enseñarlo sin tener que ir clicando
  cada paso. Dónde este equipo único no puede mostrar literalmente "lo que
  ve el otro", hay una nota explícita (`nb-demo-note`, borde discontinuo) —
  nunca se finge una segunda ventana.
- **Caminos alternativos** (rechazo, código que no coincide, sin respuesta,
  equipo ocupado, error de conexión, versión incompatible, fallo de
  firewall, fallo del motor, pérdida de conexión, cancelación) mediante
  botones "Simular: …" con el mismo borde discontinuo — nunca existirían en
  la app real, están para poder enseñar cada camino sin depender de un
  segundo equipo físico ni de provocar el fallo de verdad. Ver `FLUJOS.md`
  para la tabla completa de a qué corresponde cada uno.
- Inicio con las cinco señales de tarjeta ya separadas (disponibilidad,
  compatibilidad, confianza, favorito, selección — pueden combinarse: hay un
  ejemplo a propósito con las tres a la vez) e iconos de adaptador reales
  (Ethernet/Wi-Fi), no siempre Wi-Fi.
- Selector de equipo y conexión manual (con validación de IP).
- Resultado básico con selector de demo para ver las cuatro variantes
  (correcto / con advertencias / problemático / cancelada) sin repetir la
  prueba.
- Selector de tema real de tres vías (Sistema/Claro/Oscuro) en Ajustes, con
  detección de `prefers-color-scheme` en vivo si está en "Sistema".
- Historial y el resto de Ajustes: siguen como aviso de "todavía no está
  diseñado" — hito H3, fuera de esta ronda.

El desarrollador de frontend no necesita este fichero para nada — usa los
componentes Svelte y los tokens de más abajo. Es solo para la fase de
aprobación visual y para enseñar el recorrido completo a quien tenga que
darle el visto bueno.

## Índice de pantallas y estados — hecho vs. pendiente

Mapeado a las secciones del correo de revisión y a `Especificacion.md`.
"Maquetado" = tiene su propia composición en `maqueta-navegable.html`.
"Documentado" = está en `PANTALLAS-H1.md` o `FLUJOS.md` sin maqueta propia,
tal y como permite el correo. "Componente" = existe como pieza Svelte
reutilizable en `src/lib/components/`, independientemente de si ya hay una
pantalla real construida con ella.

| # correo | Tema | Especificacion.md | Estado |
|---|---|---|---|
| 1 | Índice de pantallas/estados y mapeo a la spec | — | Esta tabla. |
| 2 | Mapa de flujo completo, dos equipos, caminos alternativos | §10.1, §9, §10.2-10.8 | **Hecho** — `FLUJOS.md` + maquetado en `maqueta-navegable.html`. |
| 3 | Inicio/descubrimiento/equipos, ~10 variantes, señales separadas | §7 | Maquetado el estado con contenido (4 tarjetas, las 3 señales combinadas, incompatible). Variantes de contenido (buscando, ninguno encontrado, descubrimiento off, red pública, muchos favoritos, aparece/desaparece) — **documentadas** en `PANTALLAS-H1.md`, no maquetadas (misma estructura, cambia el contenido). |
| 4 | Selector de equipo + conexión manual, validación | §7.2 | Maquetado (validación de IPv4). Hostname/IPv6/puerto/opciones avanzadas — **documentado**, no maquetado (`PANTALLAS-H1.md`). |
| 5 | Emparejamiento/confianza/aceptación, iniciador y receptor, toast nativo vs. interno | §9, §10.4 | **Hecho tras la auditoría v3 (A01)** — diálogo combinado real en los dos lados (iniciador y receptor), para peer desconocido, conocido e identidad cambiada. `Toast.svelte` es el aviso **con la app en primer plano**, no el de app minimizada — es la notificación nativa del sistema (fuera de este paquete) la que cubre ese caso; corregida la fila que lo decía al revés (C05). |
| 6 | Preparación + ejecución + resultado básico | §10.5, §10.6-10.7 | **Maquetado, ampliado tras la auditoría v3 (A05)** — checklist con dos grupos explícitos (§10.2 local, ya resuelto; §10.5 conjunta, con dos fallos simulables); ejecución con el elemento principal de §16.4 (dos tarjetas de equipo + flecha de dirección, componente `ExecutionLink.svelte`), calentamiento/enfriamiento, reconexión y acción requerida recuperables (siempre con "Cancelar prueba" accesible), gráfica y ticker, cambio de dirección. |
| 7 | Resultado completo, animación, variante UDP | §16.5 | Resultado básico maquetado con **5** variantes de demo tras la auditoría v3 (A06) — ver nota de §13 más abajo. Animación de "momento premio" y variante UDP — **pendiente**, H2/H3. |
| 8 | Sistema de gráficas (color+patrón, accesibilidad, impresión) | §16.7, §19.4 | Tokens creados (`--color-chart-*`) y aplicados en la gráfica de Ejecución, con distinción color **y** patrón de trazo (discontinuo en B→A). Adaptación a impresión — **pendiente**, H2. |
| 9 | Detalles técnicos | §16.6 | **Pendiente**, botón ya presente y deshabilitado en Resultado con aviso "Hito H2". Es el único sitio previsto para nombres de proceso técnico (`ntttcp.exe`) — hasta que exista, ninguna pantalla lo muestra (auditoría v3, A09, ver `AVISOS.md`). |
| 10 | Opciones avanzadas | §17 | **Pendiente** (H3 en la propia spec), referenciado como texto en el selector. |
| 11 | Historial/evolución | §19 | **Pendiente** (H3 en la propia spec) — placeholder ya navegable. |
| 12 | Ajustes completo | §23 | Tema (Sistema/Claro/Oscuro) **hecho** y es el real. Resto — **pendiente** (H3 en la propia spec), placeholder ya navegable. |
| 13 | Diálogos/acciones delicadas, inventario | §5.2, §10.8.5 | `Dialog.svelte` como componente base **hecho**; diálogo de cierre de ventana y confirmación de cancelar-con-resultados-parciales — **pendiente** de pantalla propia (la maqueta simplifica "Cancelar prueba" a un solo paso, anotado en el propio texto del aviso). Catálogo de avisos de error/diagnóstico (§21.2) — corregido y con tabla es/en tras la auditoría v3 (A09), ver `AVISOS.md`. Foco atrapado (Tab/Shift+Tab) + fondo `inert` mientras está abierto — auditoría v3, C04, ver la sección de `Dialog.svelte` más abajo. |
| 14 | Exportación PDF | §20 | **Pendiente** (H3 en la propia spec), botón ya presente y deshabilitado con aviso "Hito H2". |
| 15 | Librería de componentes ampliada, todos los estados | — | Ver tabla de componentes más abajo. |
| 16 | Tokens completos, tres materiales con nombre, sin literales | §16.10 | **Hecho** — ver `DESIGN_TOKENS.md` § "Los tres materiales de §16.10, con nombre". `verify-tokens.mjs` en verde. Respaldo opaco (`@supports not (backdrop-filter)`) completo en los cinco componentes que lo usan (Dialog/Toast/Tooltip ya lo tenían; Card/Sidebar/Button/StatusPill corregidos tras la auditoría v3, C01), en Svelte y en la maqueta. Medidas de diálogo, duraciones y `font-size`/`saturate` sueltos, tokenizados y sincronizados entre Svelte y maqueta (auditoría v3, C03 — ver `DESIGN_TOKENS.md` §§ Tipografía/Motion/Diálogos). |
| 17 | Accesibilidad/contraste, 3 pares señalados | §16.2 | **Corregido** — ver punto 7 del changelog más abajo. Ampliado tras la auditoría v3 (C04): foco atrapado + fondo `inert` en `Dialog.svelte`, `Tooltip.svelte` con `aria-describedby`/Escape-sin-perder-el-foco, distinción no-color de "de confianza" en `DeviceCard.svelte`, nombre accesible + animación por `transform` en `ProgressBar.svelte` — ver cada componente en la tabla de más abajo. |
| 18 | Tamaño de ventana/responsive + i18n | §16.9, §4 | **Hecho tras la auditoría v3 (A08), puntos de corte corregidos tras revisión de código** — ver `RESPONSIVE-I18N.md`: los tres tramos exactos de §16.9 (compacta <1000px, estándar 1000–1400px, amplia >1400px), con sus comportamientos — sidebar solo-iconos/tarjetas en columna/gráfica 200px/direcciones apiladas en compacta; ventana que ya crece de verdad por encima de 1400px (antes se quedaba fija exactamente en 1400) y las dos direcciones de Ejecución visibles a la vez en amplia (el panel lateral de Detalles técnicos, la otra pieza de amplia, sigue sin poder construirse por depender de contenido que es H2 — ver fila B05); previsión de diseño es/en y de escala de texto 115/130% (la funcionalidad de escalado sigue siendo H2). |
| 19 | Especificación de animaciones | §16.11 | Micro-interacciones existentes (hover/press) documentadas en el código; coreografías de "momento premio" — **pendiente**, ligado al punto 7. |
| 20 | Iconos/app-icon/tray-icon | §16.12 | Librería de iconos de interfaz ampliada (18 iconos nuevos en `icons.ts`); app-icon/tray-icon — **pendiente**, son activos de producto, no de este sistema de diseño. |
| 21 | 7 correcciones a la entrega anterior | — | **Hecho** — changelog completo más abajo. |
| 22 | Orden de entrega H1→H2→H3 | §26 | Confirmado; esta entrega es H1. |

## Resultado — regla de veredicto usada en los ejemplos (§13, auditoría v3 A06)

Tres correcciones a la pantalla de Resultado, todas del mismo problema de
fondo: los ejemplos numéricos no estaban vinculados a ninguna regla, así
que sonaban razonables sin serlo.

**"Pérdida de paquetes" era una métrica de UDP.** La prueba estándar es
TCP, y TCP no pierde en silencio: retransmite. Lo que hay que mostrar es
la retransmisión (segmentos retransmitidos, con su % sobre el total), no
una pérdida que TCP no expone tal cual. Corregido en el tercer dato del
grid (`Retransmisiones`).

**Los ejemplos de "correcto" no eran correctos si se comparan con el
adaptador real.** SERVER-01 y DESKTOP-DANIEL tienen los dos un adaptador
de 10 Gbit/s (ver pane Inicio y `ExecutionLink.svelte`) — así que "941
Mbit/s" como resultado "bueno" es en realidad el 9,4 % de esa capacidad,
que por cualquier regla razonable sería un resultado pobre, no bueno.
Regla real, **§13.1–§13.2 de `Especificacion.md`** (confirmada tras la
revisión del desarrollador — en la ronda anterior este documento no estaba
disponible completo y se usó una regla propuesta a falta de él; los
umbrales de esa versión provisional, 85/50/50, no coincidían con los
reales y quedan corregidos aquí): `util = velocidad / ref` por dirección,
con `ref` = capacidad del adaptador **más lento** de los dos extremos
(nunca un número absoluto fijo) — y sobre el **mínimo** de ambas
direcciones para el veredicto global:

| `util` | Nivel | Título (§13.2) |
|---|---|---|
| ≥ 0,85 | OK | «Rendimiento acorde a la capacidad del enlace» |
| 0,60 – 0,85 | WARN | «El rendimiento es algo inferior a lo esperable» |
| < 0,60 | PROBLEM | «El rendimiento es claramente inferior a lo esperable» |

Los ejemplos de las cuatro variantes con veredicto (`RESULT_VARIANTS` en
`maqueta-navegable.html`) ya usaban el mínimo de ambas direcciones sobre
la capacidad del enlace más lento — con estos umbrales corregidos siguen
cayendo en el mismo nivel que antes (89 %→OK, 62 %→WARN, 15 %→PROBLEM), así
que no ha hecho falta tocar las cifras de los ejemplos, solo el propio
texto de la regla, que antes citaba 50 % como frontera en vez de 60 %.

**Cifra principal de Nivel 1 (§16.5): velocidad, no %.** Pregunta abierta
en la ronda anterior, ya confirmada por el cliente: la cifra grande es la
**velocidad** (mín. de ambas direcciones, o la única si solo se completó
una), coherente con la pantalla de Ejecución, que ya usa la velocidad como
cifra grande («9,36 Gbit/s», §16.4). El % de la capacidad del enlace pasa
a texto de apoyo debajo — sigue siendo el dato que decide el veredicto de
la tabla de arriba, solo que ya no es la cifra visualmente protagonista.

**"Cancelada" e "incompleta" son dos cosas distintas.** Antes solo
existía "cancelada". Ahora hay una quinta variante de demo,
`incomplete`: la conexión se pierde sin recuperación a mitad de la
prueba (`NB-CONN-005`, ver FLUJOS.md "Pérdida de canal, sin
recuperación") con una dirección ya completa que se conserva (§19.3,
H3). Misma forma visual de "sin veredicto" en la cifra principal que
"cancelada", pero tono de fallo (no de advertencia) y un texto que deja
claro que fue un fallo, no una decisión del usuario — son causas
distintas y no deberían leerse igual.

## Changelog — los 7 puntos del apartado 21 del correo

1. **Titlebar: fondo/espaciado/colores/ancho de botón vs. §16.13.** El
   ancho de los botones de ventana era 46px, se corrige a **48px**
   (`TitleBar.svelte` y la maqueta). La altura de 32px de la barra ya era
   correcta — el error estaba solo en el ancho de los botones. Se elimina
   también la línea de separación inferior que pedía §16.13 ("no debe
   percibirse como una barra distinta"). **El degradado de fondo de la
   barra sigue sin ser el plano opaco `--solid` que pide literalmente
   §16.13** — es una decisión de diseño ya aprobada visualmente (mockup de
   propuestas) que contradice la letra de ese punto de la spec. Queda
   señalada aquí expresamente, como pide el propio correo ("si alguna
   decisión de diseño supone cambiar un requisito de la especificación,
   indícanoslo para aprobar el cambio") — **pendiente de que la aprobéis
   explícitamente o pidáis volver al plano opaco**. Comentario también en
   `TitleBar.svelte`.
2. **Tarjeta de equipo: disponibilidad/compatibilidad/confianza/favorito
   mezclados.** `DeviceCard.svelte` reescrito con cinco props
   independientes (`availability`, `compatible`, `trust`, `favorite`,
   `selected`) que pueden combinarse libremente — el propio correo lo pedía
   con el ejemplo "un equipo puede ser favorito, de confianza y estar
   ocupado al mismo tiempo", y es exactamente el ejemplo que reproducen
   `HomeScreenExample.svelte` y la maqueta (NAS-BACKUP). El icono de
   confianza (escudo) y el de favorito (estrella) están deliberadamente
   separados para no confundir los dos conceptos.
3. **Inconsistencias maqueta ↔ Svelte.** Se revisaron y unificaron los
   valores señalados (tamaño de fuente del nombre de equipo, ahora
   `var(--font-size-2xl)` en ambos sitios en vez de un `21px` suelto en la
   maqueta) y se dejó `maqueta-navegable.html` usando exactamente los
   mismos valores de token que `tokens.css`.
4. **Tema inicial "Oscuro" en vez de "Sistema".** El código
   (`theme.svelte.ts`, función `readStoredMode()`) **ya hacía lo correcto**
   — por defecto `"system"`; el fallo estaba solo en la documentación (este
   README) y en la maqueta, que arrancaban asumiendo oscuro. Corregido en
   ambos sitios; la maqueta ahora detecta `prefers-color-scheme` de verdad
   cuando está en modo Sistema.
5. **Tooltips nativos → patrón diseñado.** Nuevo componente
   `Tooltip.svelte` (glass-overlay, retardo de 250ms, respeta
   `prefers-reduced-motion`) sustituyendo todo uso de `title=` — en
   `Sidebar.svelte` (pista de "Prueba en curso") y en la maqueta
   (`data-tt`, equivalente CSS-only del mismo patrón visual, usado en los
   botones de ventana y en los indicadores de adaptador/confianza de las
   tarjetas). **C04 (auditoría v3):** ganó `aria-describedby` (snippet
   parametrizado con el id de la burbuja) y Escape-oculta-sin-perder-el-foco
   — ver la sección propia de `Tooltip.svelte` más abajo para el detalle y
   la limitación conocida del equivalente `[data-tt]` de la maqueta.
6. **Fallbacks opacos de material incompletos.** Los tres materiales de
   cristal con nombre (`glass-chrome`/`glass-card`/`glass-overlay`, §16.10)
   tienen ahora su fallback `@supports not (backdrop-filter: …)` completo,
   con un token de superficie sólida propio para cada uno
   (`--surface-solid-chrome/-card/-overlay`) — ver `DESIGN_TOKENS.md`.
7. **Contraste: 3 pares por debajo de 4.5:1 en tema claro.** Los tres
   colores señalados se oscurecieron (mismo matiz, más oscuros) hasta
   cumplir AA sobre `--color-bg-main` (`#F8F7FB`), verificado por cálculo
   directo de la fórmula de contraste WCAG, no a ojo:
   - `--color-text-muted`: `#817A8D` (3.86:1) → **`#6B6478`** (5.29:1).
   - `--color-success` (claro): `#238E5B` (3.87:1) → **`#1B7549`** (5.34:1).
   - `--color-warning` (claro): `#A87400` (3.81:1) → **`#8A5E00`** (5.35:1).

   Los tokens `*-soft` (fondos translúcidos de las píldoras de estado) se
   actualizaron a juego con el nuevo valor base. Aplicado en `tokens.css`
   y en `maqueta-navegable.html` (que mantiene sus propios valores para
   no depender de `<link>` externos, ver comentario al inicio del
   fichero) — mismo valor en los dos sitios.

   **Hallazgo adicional, no estaba en la lista de 3 del correo:** al
   aplicar el mismo cálculo de contraste al resto de la paleta aparecieron
   dos pares más por debajo de AA en tema claro — `--color-danger`
   (`#D84D62`, 3.82:1) y `--color-info` (`#506FCC`, 4.39:1). Corregidos con
   el mismo criterio: `#C23A50` (4.90:1) y `#4661B8` (5.38:1). Ver
   `DESIGN_TOKENS.md` para la tabla completa recalculada.

## Arquitectura — encaja con `Especificacion.md` §25

```
src/lib/design-system/
  tokens.css        → paleta/tipografía/espaciado/radios/blur/motion/
                       diálogos/campos/materiales/gráficas (ampliado H1)
  theme.svelte.ts    → aplica data-theme="light"|"dark" en <html>
src/lib/components/
  AppBackground.svelte
  TitleBar.svelte
  Sidebar.svelte
  Button.svelte
  Card.svelte
  StatusPill.svelte
  DeviceCard.svelte        ← reescrito H1: 5 señales independientes
  ThemeToggle.svelte
  Tooltip.svelte            ← nuevo H1
  Dialog.svelte             ← nuevo H1
  Toast.svelte               ← nuevo H1
  TextField.svelte           ← nuevo H1
  VerificationCode.svelte    ← nuevo H1
  ChecklistItem.svelte       ← nuevo H1
  ProgressBar.svelte         ← nuevo H1
  Icon.svelte + icons.ts    ← +18 iconos H1
  HomeScreenExample.svelte   ← referencia visual, NO es una ruta real
scripts/
  verify-tokens.mjs  → comprobación de CI: cero colores literales
maqueta-navegable.html   → recorrido completo navegable, para aprobación
FLUJOS.md                → mapa de estados del recorrido, con alternativas
PANTALLAS-H1.md          → estados documentados sin maqueta propia
DESIGN_TOKENS.md         → referencia de cada token
RESPONSIVE-I18N.md       → reglas responsive + previsión i18n/escala de texto (A08)
AVISOS.md                → catálogo de avisos H1, códigos y textos es/en (A09)
```

Copia `design-system/`, `components/` y `scripts/verify-tokens.mjs` dentro
del proyecto Tauri + Svelte + TS + Vite ya iniciado, en las mismas rutas.

## Instalación

1. Importa los tokens **una sola vez**, en el layout raíz:

   ```svelte
   <!-- src/routes/+layout.svelte -->
   <script>
     import '$lib/design-system/tokens.css';
     import '$lib/design-system/theme.svelte'; // activa el control de tema al arrancar
   </script>
   <slot />
   ```

2. `TitleBar.svelte` importa `@tauri-apps/api/window` de forma dinámica y
   con `try/catch`, así que no rompe `vite dev` en el navegador; en el
   binario real necesita `"decorations": false` en la ventana de
   `tauri.conf.json` (§16.13).

3. Añade `node scripts/verify-tokens.mjs` al workflow `ci.yml`, junto al
   resto de verificaciones propias que ya describe la Especificación §3.5
   paso 2 (antes de Prettier/ESLint/`svelte-check`). Falla si alguien
   introduce un color a pelo en un componente.

4. `HomeScreenExample.svelte` **no es una ruta real** — es solo la prueba de
   que los átomos combinados reproducen el mockup y de que el tema cambia
   sin tocar componentes. La pantalla de Inicio de verdad debe vivir en
   `routes/` y conectarse a `lib/stores/` (descubrimiento mDNS, sesión…) y
   `lib/api/` (comandos Tauri), no traer datos de muestra hardcodeados.

## Cómo funciona el tema — para quien construya pantallas nuevas

1. `theme.svelte.ts` expone un objeto `theme` con `theme.mode` (`"system" |
   "light" | "dark"`, lo que el usuario eligió) y `theme.resolved`
   (`"light" | "dark"`, lo que está aplicado ahora mismo — ya resuelto
   contra `prefers-color-scheme` si `mode` es `"system"`). Por defecto,
   `mode` es **`"system"`** (ver changelog, punto 4).
2. Internamente escribe `data-theme="light"` o `"dark"` en `<html>`.
   `tokens.css` define el tema oscuro en `:root` y lo sobrescribe en
   `:root[data-theme="light"]`.
3. **Ningún componente necesita saber en qué tema está.** Si un componente
   nuevo solo usa `var(--token)`, cambia de tema solo. Si sientes la
   tentación de escribir `{#if theme.resolved === 'light'}` dentro de un
   componente para decidir un color, es señal de que falta un token en
   `tokens.css`, no de que haga falta esa rama condicional.
4. Para cambiar el tema desde código: `theme.setMode('light' | 'dark' |
   'system')`. La UI de Ajustes ya existe: `ThemeToggle.svelte` (tres
   opciones — Sistema / Claro / Oscuro, exactamente como pide §23).
5. Persistencia: por ahora `theme.svelte.ts` usa `localStorage` como
   placeholder. Hay un `TODO` marcado en el fichero indicando dónde
   sustituirlo por la lectura/escritura real de `settings.json` (§19.1)
   cuando exista el comando Tauri correspondiente.

## Componentes — API de referencia

### `AppBackground.svelte`

Fondo ambiental de la ventana (gradientes + dos formas difusas, nunca una
imagen). Se coloca como primer hijo del contenedor con `position: relative`
que hace de "lienzo" de la pantalla.

| Prop | Tipo | Defecto | Para qué |
|---|---|---|---|
| `state` | `"idle" \| "success" \| "warning" \| "danger"` | `"idle"` | Enganche para cuando exista el motor de interpretación (§13) — hoy solo aplica un filtro sutil, no cambia la estructura. |

### `TitleBar.svelte`

Barra de título propia, ya conectada a la API de ventana de Tauri.

| Prop | Tipo | Defecto |
|---|---|---|
| `appName` | `string` | `"NetworkBench"` |
| `labels` | `{ minimize, maximize, restore, close: string }` | textos en español — **sustituir por las claves i18n reales (§4)** |

No emite eventos: minimizar/maximizar/cerrar actúan directamente sobre la
ventana de Tauri. El cierre siempre pasa por `CloseRequested`, así que el
diálogo de §5.2 sigue funcionando sin cambiar nada aquí.

### `Sidebar.svelte`

| Prop | Tipo | Defecto |
|---|---|---|
| `items` | `{id, label, icon: IconName}[]` | Inicio / Historial / Ajustes |
| `activeId` | `string` | — (requerido) |
| `onNavigate` | `(id: string) => void` | — (requerido) |
| `disabled` | `boolean` | `false` — actívalo desde que se entra en `CONNECTING` (auditoría v3, A07 — antes desde `PREPARING`, ver `FLUJOS.md` nota 3) hasta `COMPLETED`/`FAILED`/`CANCELLED` (§16.1) |
| `disabledHint` | `string` | `"Prueba en curso"` — vía `Tooltip.svelte` cuando `disabled` |

**C04 (auditoría v3, accesibilidad):** el ítem deshabilitado usaba el
atributo `disabled` nativo **además** de `aria-disabled` — pero `disabled`
saca el botón del orden de tabulación, así que su propio `Tooltip` (que se
muestra al recibir el foco) nunca era alcanzable por teclado. Pasa a usar
solo `aria-disabled="true"` (sigue foco-alcanzable, sin `onclick` en esa
rama así que activarlo no hace nada) — atenuado visual movido de `:disabled`
a `[aria-disabled="true"]` en el CSS.

**A07 — "Inicio" nunca queda inerte:** con `disabled=true`, todos los
demás ítems se deshabilitan, pero "Inicio" sigue siendo un botón normal
que llama a `onNavigate("inicio")` igual que siempre — el componente no
sabe nada de sesiones ni de confirmaciones. Es quien lo usa el que, al
recibir `onNavigate("inicio")` con una sesión activa, tiene que mostrar
una confirmación en vez de navegar directo (`maqueta-navegable.html` lo
resuelve con el overlay `confirm-leave` y el estado `sessionActive` —
mismo patrón a seguir en la app real). La alternativa de dejarlo inerte
como el resto se descartó: no debe existir un estado en el que la
persona no tenga NINGÚN sitio al que ir para salir de la prueba.

Material `glass-chrome` (`backdrop-filter: blur(var(--blur-sidebar))`) — ya
tiene su respaldo opaco (C01, auditoría v3), `--surface-solid-chrome`.

### `Button.svelte`

| Prop | Tipo | Defecto |
|---|---|---|
| `variant` | `"primary" \| "secondary" \| "ghost"` | `"secondary"` |
| `disabled` | `boolean` | `false` |
| `type` | `"button" \| "submit"` | `"button"` |
| `onclick` | `(e: MouseEvent) => void` | — |
| `icon` | `Snippet` (opcional) | — icono a la izquierda del texto |
| `children` | `Snippet` (requerido) | texto del botón |

Un único botón `primary` por pantalla/sección. Todo lo demás es `secondary`
(cristal) o `ghost` (texto de acento, sin fondo). `secondary` es el único
con `backdrop-filter` — ya tiene su respaldo opaco (C01, auditoría v3), ver
`DESIGN_TOKENS.md` § "Los tres materiales…".

### `Card.svelte`

| Prop | Tipo | Defecto |
|---|---|---|
| `variant` | `"default" \| "hero" \| "selected"` | `"default"` |
| `onclick` | `(e: MouseEvent) => void` (opcional) | si se pasa, la card es interactiva |
| `disabled` | `boolean` | `false` — atenúa la card tanto si es interactiva como si no |
| `children` | `Snippet` (requerido) | — |

**Cambio H1:** cuando `onclick` está presente, la card ya **no** se
renderiza como `<button>` sino como `<div role="button" tabindex="0">` con
gestión manual de Enter/Espacio. Motivo: `DeviceCard` necesita meter un
`<button>` real dentro (el toggle de favorito), y `<button>` dentro de
`<button>` es HTML inválido — el navegador cierra el de fuera en cuanto
encuentra el de dentro, descuadrando el layout entero (bug real, visto y
corregido en esta ronda, tanto aquí como en `maqueta-navegable.html`). Esto
es la única excepción a la regla 3 de más abajo ("todo lo interactivo es un
elemento nativo") y está documentada como tal en el propio código.

`default` y `hero` usan `backdrop-filter` (`selected` no); las dos tienen ya
su respaldo opaco (C01, auditoría v3) — mismo token para ambas, ver
`DESIGN_TOKENS.md`.

### `StatusPill.svelte`

| Prop | Tipo | Defecto |
|---|---|---|
| `tone` | `"success" \| "warning" \| "danger" \| "info" \| "neutral"` | `"neutral"` |
| `variant` | `"pill" \| "inline"` | `"pill"` |
| `children` | `Snippet` (requerido) | texto del estado |

`variant="pill"` es cristal (`backdrop-filter`); `"inline"` no. El pill no
encaja en ninguno de los tres materiales con nombre de §16.10 (token propio,
`--pill-bg`), así que tiene su propio respaldo opaco dedicado,
`--surface-solid-pill` (C01, auditoría v3 — ver `DESIGN_TOKENS.md`).

### `DeviceCard.svelte` — reescrito en esta ronda (changelog punto 2)

| Prop | Tipo | Defecto | Para qué |
|---|---|---|---|
| `name` | `string` | — | |
| `ip` | `string` | — | |
| `alias` | `string` (opcional) | — | |
| `adapterType` | `"ethernet" \| "wifi" \| "other"` | — (obligatoria) | icono de adaptador correcto — **nunca asumir Wi-Fi**; por eso ya no tiene valor por defecto (auditoría v3, C05) — si de verdad no se conoce, pasar `"other"` a propósito |
| `linkSpeedMbps` | `number` (opcional) | — | |
| `availability` | `"available" \| "busy" \| "unreachable"` | `"available"` | eje independiente de compatibilidad/confianza |
| `compatible` | `boolean` | `true` | si es `false`, prevalece sobre `availability` en el texto de estado |
| `trust` | `"unknown" \| "known" \| "trusted"` | `"unknown"` | icono de escudo, nunca la estrella |
| `favorite` | `boolean` | `false` | icono de estrella, botón propio con `onToggleFavorite` |
| `selected` | `boolean` | `false` | |
| `onclick` / `onToggleFavorite` | funciones (opcionales) | — | |

Los cinco ejes son independientes y pueden combinarse — un equipo puede ser
favorito, de confianza y estar ocupado a la vez (`HomeScreenExample.svelte`
y la maqueta lo demuestran con NAS-BACKUP). `isSelectable = compatible &&
availability === "available"`.

**C04 (auditoría v3, accesibilidad):** "de confianza" distinguía de
"conocido" **solo por color** en el escudo (regla 6 de más abajo — "icono/
punto + texto, nunca solo color"). Añadido un punto no-color propio
(`.nb-device-trust-badge`) visible únicamente en `trust="trusted"`, y el
propio escudo pasa a ser foco-alcanzable (`tabindex="0"`, `role="img"`,
`aria-label`) — necesario además para que su `Tooltip` (que ahora se
muestra al recibir el foco, no por `:focus-within` puro, ver más abajo)
tenga algo que enfocar. La separación favorito/tarjeta
(`e.stopPropagation()` en el botón de estrella) ya estaba resuelta antes de
esta ronda — verificada de nuevo, sin cambios. Sincronizado con el mismo
punto en `maqueta-navegable.html` (`::after` en vez de un elemento nuevo,
porque ahí el marcado es estático).

### `ThemeToggle.svelte`

Sin props — lee y escribe directamente en el `theme` compartido de
`theme.svelte.ts`. Sistema / Claro / Oscuro (§23).

### `Tooltip.svelte` — nuevo H1

| Prop | Tipo | Defecto |
|---|---|---|
| `text` | `string` | — (requerido) |
| `placement` | `"top" \| "bottom" \| "left" \| "right"` | `"top"` |
| `children` | `Snippet<[string]>` (requerido) | recibe el `id` de la burbuja, para `aria-describedby` |

Sustituye cualquier uso de `title=` nativo (changelog punto 5). Retardo de
250ms antes de aparecer, respeta `prefers-reduced-motion`.

**C04 (auditoría v3, accesibilidad — "aria-describedby y cierre con Escape
en Tooltip, alcanzable por teclado"):** dos cambios.
`children` pasó de `Snippet` a `Snippet<[string]>`: recibe el `id` único de
la propia burbuja (`$props.id()`) para que quien instancia el `Tooltip` lo
ponga como `aria-describedby` en **su** elemento — este componente no
controla qué renderiza `children()`, así que no puede añadir el atributo
por su cuenta (ver el uso en `Sidebar.svelte`/`DeviceCard.svelte`). Y la
visibilidad dejó de ser pura CSS (`:hover`/`:focus-within`) y pasó a un
`$state` explícito: el patrón WAI-ARIA de tooltip pide que Escape **oculte**
la burbuja sin mover el foco fuera del control envuelto — con CSS puro no
hay forma de expresar "oculto por Escape pero el foco sigue aquí", porque
`:focus-within` seguiría siendo cierto. La maqueta usa un mecanismo distinto
y más simple (`[data-tt]`, CSS puro vía `content: attr(data-tt)`) que no
puede llevar `aria-describedby` ni un Escape-sin-perder-el-foco sin
rehacerse por completo — limitación conocida, documentada aquí en vez de
dejarse sin mencionar. Sí se corrigió ahí el alcance por teclado: los
`[data-tt]` sobre `<span>` no interactivos (no un `<button>`) no tenían
`tabindex`, así que nunca podían recibir foco — se les añadió `tabindex="0"`
donde corresponde (p. ej. el escudo de confianza).

### `Dialog.svelte` — nuevo H1

| Prop | Tipo | Defecto |
|---|---|---|
| `title` | `string` | — (requerido) |
| `description` | `string` (opcional) | — |
| `onClose` | `() => void` (opcional) | — |
| `dismissible` | `boolean` | `true` — si es `false`, ni Escape ni clic fuera cierran |
| `size` | `"sm" \| "md" \| "lg"` | `"sm"` |
| `children` | `Snippet` (requerido) | contenido |
| `actions` | `Snippet` (opcional) | fila de botones |

Base para emparejamiento, aceptación y avisos (§9, §10.4) — en la maqueta,
`.nb-overlay`/`.nb-dialog` son el equivalente estático de este componente.

**C04 (auditoría v3, accesibilidad — "foco de Dialog"):** antes solo se
ponía el foco inicial al abrir, pero nada impedía que Tab lo sacara hacia
el fondo (`aria-modal="true"` es una pista para lectores de pantalla, no
una barrera real de teclado). Ahora Tab/Shift+Tab quedan atrapados dentro
del diálogo, y el resto de la ventana se marca `inert` mientras está
abierto (tampoco alcanzable por un lector de pantalla en modo de
exploración) — se deshace al cerrar, sin tocar nada que ya estuviera
`inert` por otro motivo. Se asume que el componente cuelga cerca de la raíz
de la pantalla; ver el comentario completo en el propio archivo sobre ese
supuesto. Equivalente en la maqueta: un único observador genérico
(`MutationObserver` sobre la clase `is-active` de cualquier `.nb-overlay`,
ver `<script>`) hace lo mismo sin tener que tocar cada uno de los puntos
donde se abre un overlay — los `.nb-overlay` son hermanos de `.nb-body`
(ambos hijos de `.nb-window`), así que `inert` se marca ahí, no en
`.nb-sidebar`/`.nb-main` directamente (lo heredan por comportamiento).

### `Toast.svelte` — nuevo H1

| Prop | Tipo | Defecto |
|---|---|---|
| `tone` | `"info" \| "success" \| "warning" \| "danger"` | `"info"` |
| `icon` | `IconName` (opcional) | según `tone` |
| `onDismiss` | `() => void` (opcional) | — |
| `children` | `Snippet` (requerido) | — |

**Distinto del toast nativo de Windows** (§10.4): este componente es para
cuando la app está en primer plano; el aviso de solicitud entrante con la
app minimizada usa la notificación nativa del sistema, fuera de este
paquete. No confundir los dos — es uno de los puntos que señalaba el correo
(apdo. 5).

### `TextField.svelte` — nuevo H1

| Prop | Tipo | Defecto |
|---|---|---|
| `label` | `string` | — (requerido) |
| `value` | `string` (`$bindable`) | `""` |
| `placeholder` / `hint` / `error` | `string` (opcionales) | — |
| `disabled` / `readonly` | `boolean` | `false` |
| `type` | `"text" \| "search"` | `"text"` |
| `id` / `oninput` | — | — |

El error se muestra con icono **y** texto, nunca solo el borde en rojo
(regla 6 de más abajo). Usado en la pantalla de conexión manual.

### `VerificationCode.svelte` — nuevo H1

| Prop | Tipo |
|---|---|
| `code` | `string` |

Solo visualización (no es un input) del código de 6 dígitos de
emparejamiento (§9.1.5) — formatea en dos grupos de 3, número tabular,
texto seleccionable.

### `ChecklistItem.svelte` — nuevo H1

| Prop | Tipo | Defecto |
|---|---|---|
| `label` | `string` | — (requerido) |
| `state` | `"pending" \| "running" \| "done" \| "failed" \| "action"` | — (requerido) |
| `detail` | `string` (opcional) | motivo del fallo o de la acción necesaria |
| `actionSlot` | `Snippet` (opcional) | botón de acción disponible |

Pantalla de Preparación (§10.5). Cinco estados, no tres — hacen falta para
no dejar "en curso" o "necesita una acción" (p. ej. UAC rechazado) a medias.

### `ProgressBar.svelte` — nuevo H1

| Prop | Tipo | Defecto |
|---|---|---|
| `value` | `number` (0-100) | `0` |
| `indeterminate` | `boolean` | `false` |
| `tone` | `"accent" \| "success" \| "warning" \| "danger"` | `"accent"` |
| `label` | `string` | — (obligatoria) |

Pantalla de Ejecución ("barra de progreso de la dirección actual con
«Tiempo restante aproximado»", §16.4) y tramos sin duración estimable
(`indeterminate`, p. ej. "Analizando resultados…").

**C04 (auditoría v3, accesibilidad — "nombre accesible y animación por
transform"):** dos cambios. `label` es nueva y **obligatoria** (mismo
criterio que `adapterType` en `DeviceCard`, C05): da el nombre accesible del
propio `role="progressbar"` — antes no tenía ninguno, un lector de pantalla
solo anunciaba el número ("27 %") sin decir de qué; quien instancia el
componente tiene que decidirlo (p. ej. la fase real: "Calentando…"). Y el
relleno determinado pasa de animar `width` (dispara layout en cada frame) a
`transform: scaleX()` con `transform-origin: left` — solo compositor, igual
que el resto de animaciones de la app. Sincronizado en la maqueta:
`.nb-progress` gana `role="progressbar"`/`aria-label`/`aria-valuenow` (este
último se quita mientras es indeterminado, igual que aquí), y su relleno
pasa igualmente de `style.width` a `style.transform: scaleX()`.

**C03 (auditoría v3):** la animación de `indeterminate` usa
`--duration-progress-indeterminate` (1.3s) — la maqueta tenía este mismo
tramo desincronizado (1.1s, y bajo `prefers-reduced-motion` solo lo
ralentizaba a 2.4s en vez de pararlo). Corregida para igualar el
comportamiento de este componente, que es la fuente real: con movimiento
reducido, la animación se **para** del todo (barra fija al 100%, opacidad
0.5), no se ralentiza.

### `ExecutionLink.svelte` — nuevo H1 (A05)

| Prop | Tipo |
|---|---|
| `local` | `{ name, nicModel, adapterType, linkSpeedMbps }` |
| `remote` | mismo tipo que `local` |
| `direction` | `"outbound" \| "inbound"` |

Elemento principal de Ejecución (§16.4): las dos tarjetas de equipo (con
modelo de NIC y velocidad de enlace) unidas por una flecha de dirección.
`local` es siempre este equipo y siempre va a la izquierda; `remote`
siempre a la derecha — el componente no tiene ningún prop de
perspectiva, así que no hay forma de invertir el significado de la
flecha cambiando quién se ve como "local" (auditoría v3, A05: "sin
invertir el significado al cambiar de perspectiva local/remoto"). Solo
`direction` cambia con el tiempo, y solo mueve la flecha, nunca las
tarjetas.

### `Icon.svelte` + `icons.ts`

| Prop | Tipo | Defecto |
|---|---|---|
| `name` | `IconName` (ver `icons.ts`) | — |
| `size` | `number` (px) | `18` |
| `color` | `string` CSS | `"currentColor"` |

+18 iconos nuevos esta ronda (`laptop`, `ethernet`, `adapter-other`,
`shield`, `alert-triangle`, `x-circle`, `edit`, `trash`, `search`,
`chevron-down`, `close`, `copy`, `external-link`, `refresh`, `arrow-right`,
`clock`, `info`, `wifi-off`, `filter`). Nunca un emoji, nunca una librería
de terceros con su propio estilo.

## Reglas para construir componentes nuevos

1. **Cero colores/gradientes/sombras coloreadas literales.** Si no existe el
   token, créalo en `tokens.css` con su pareja claro/oscuro antes de usarlo.
   `scripts/verify-tokens.mjs` lo hace cumplir.
2. `cursor: default` en todo lo clicable que no sea un enlace externo; nunca
   `text-decoration: underline`; `user-select: none` salvo en texto
   copiable/editable.
3. Todo lo interactivo es un elemento nativo (`<button>`, `<a href>`,
   `<input>`+`<label>`) — con **una excepción documentada**: una card
   interactiva que contiene otro control interactivo propio (p. ej. el
   favorito de `DeviceCard`) usa `<div role="button" tabindex="0">` con
   Enter/Espacio gestionados a mano, porque `<button>` no puede contener
   otro `<button>` (ver `Card.svelte`). Fuera de ese caso concreto, sigue
   siendo siempre un elemento nativo.
4. `:focus-visible { outline: 2px solid var(--color-focus-ring); }` en todo
   lo interactivo.
5. Toda transición/transform va envuelta en `@media (prefers-reduced-motion:
   reduce)` que la anula; hover se limita a `translateY(-1px)`, pressed a
   `scale(.985)`.
6. Estados con icono/punto **+ texto**, nunca solo color.
7. Nada de `<select>` nativo, `alert`/`confirm`/`prompt` del navegador,
   `title=` nativo (usa `Tooltip.svelte`), emojis como iconos, ni fuentes
   descargadas de Internet para la UI.

## Hoja de ruta confirmada (§22 del correo)

**H1 (esta entrega):** recorrido principal completo en ambas direcciones —
Inicio, selector/conexión manual, emparejamiento/aceptación, preparación,
ejecución, resultado básico con errores y cancelación. Confirmado por el
cliente como orden de trabajo.

**H2 (siguiente):** resultado completo con animación, detalles técnicos,
catálogo de errores completo con textos finales, firewall, accesibilidad
ampliada.

**H3:** historial/evolución, opciones avanzadas, UDP, exportación
PDF/JSON/CSV, Ajustes completo, bandeja del sistema, actualizador.

**Corrección (auditoría v3, C05):** la versión anterior de esta hoja de
ruta ponía "exportación PDF" en H2. `Especificacion.md` §20 lleva la
etiqueta `[H3]` en su propio encabezado, igual que §17 (opciones
avanzadas) y §18 (UDP) — los tres son H3, no H2. Corregido aquí; ver
también la tabla B más abajo (B08).

## Cierre de la auditoría v3

`AUDITORIA_DISENO_V3.md` (la revisión del desarrollador de frontend sobre
la entrega anterior) organizaba sus puntos en cuatro bloques — A01–A09
(urgentes, bloqueantes de H1), C01–C05 (correcciones transversales),
B01–B12 (alcance H2/H3) y D01–D04 (decisiones fuera del rol de diseño).
Esta sección es el cierre punto por punto: qué se ha hecho, qué queda
comprometido para cuándo, y qué se ha dejado señalado en vez de decidido
por cuenta propia. El texto literal de `AUDITORIA_DISENO_V3.md` no está
disponible en esta ronda — donde hace falta reconstruir contenido a partir
de lo que ya está resuelto en el resto del README, se dice expresamente
aquí, no se presenta como cita. `Especificacion.md`, en cambio, ya está
disponible completo desde la revisión de código del desarrollador (ronda
posterior a A09) — los puntos de este documento que citaban esa misma
limitación para la spec (umbrales de §13.2, puntos de corte de §16.9,
códigos de §21.2/§14.4) están corregidos y confirmados contra ella, no
como una interpretación pendiente de confirmar.

### A01–A09 — bloqueantes de H1

Los nueve, **hechos** en esta entrega. Cada uno tiene su propia nota en la
tabla "Índice de pantallas y estados" de más arriba y/o en su sección de
componente — no se repiten aquí para no duplicar y desincronizar. Resumen
de una línea por si solo hace falta la lista:

| # | Tema | Dónde está el detalle |
|---|---|---|
| A01 | Diálogo real de emparejamiento/aceptación (receptor), antes solo descrito en texto | Fila 5 de la tabla de pantallas; `Dialog.svelte` |
| A02 | "Analizar conexión" con 0/1/varios candidatos, de verdad maquetado (no solo descrito) | `PANTALLAS-H1.md` § "'Analizar conexión' con 0/1/varios candidatos"; `FLUJOS.md` nota de fila 2 |
| A03 | Nombre de equipo truncado sin forma de ver el completo; acceso a info de equipo incompatible | `DeviceCard.svelte` (`title=`), fila 3 |
| A04 | Conexión manual: validación real de nombre DNS, IPv4 e IPv6 | `PANTALLAS-H1.md` § conexión manual |
| A05 | Checklist con dos grupos explícitos (local §10.2 / conjunta §10.5), `ExecutionLink.svelte` | Fila 6 de la tabla de pantallas |
| A06 | Regla de veredicto de Resultado, ejemplos recalculados contra el adaptador más lento, variante "incompleta" distinta de "cancelada" | Sección "Resultado — regla de veredicto" |
| A07 | Bloqueo de sidebar movido de `PREPARING` a `CONNECTING`; "Inicio" nunca queda inerte | `Sidebar.svelte`, `FLUJOS.md` nota 3 |
| A08 | Responsive (800×600/1100×760/≥1450px) + previsión i18n/escala de texto — puntos de corte confirmados como compacta/estándar/amplia (§16.9) y tramo amplia completado tras revisión de código posterior | `RESPONSIVE-I18N.md` |
| A09 | Catálogo de avisos corregido (nombres de versión, código NB-FW-002, "Ver detalles", `ntttcp.exe` retirado de UI), tabla es/en | `AVISOS.md` |

### C01–C05 — correcciones transversales

| # | Tema | Estado | Dónde está el detalle |
|---|---|---|---|
| C01 | Respaldo opaco (`@supports not (backdrop-filter)`) en los materiales con nombre | **Hecho** — Card/Sidebar/Button/StatusPill corregidos (Dialog/Toast/Tooltip ya lo tenían), Svelte y maqueta | Fila 16 de la tabla de pantallas; `DESIGN_TOKENS.md` |
| C02 | Contraste AA del gradiente `--gradient-button-primary` con texto blanco (tema claro), verificado extremo por extremo, no solo en el punto medio | **Hecho** | `DESIGN_TOKENS.md` § tabla de contraste del botón primario |
| C03 | Literales sueltos (`11.5px`, `saturate`, duraciones, anchos de diálogo) tokenizados y sincronizados maqueta↔Svelte | **Hecho** — de paso, 3 desincronizaciones reales encontradas y corregidas (transición de `ProgressBar`, duración indeterminada, comportamiento con movimiento reducido) | `DESIGN_TOKENS.md`; sección de `ProgressBar.svelte` |
| C04 | Accesibilidad: foco de `Dialog`, `Tooltip` por teclado, distinción no-color en `DeviceCard`, nombre accesible + animación por `transform` en `ProgressBar` | **Hecho** — ver cada componente en "Componentes — API de referencia"; equivalente aplicado también en la maqueta (overlays, escudo de confianza, barra de progreso) | Fila 13 y 17 de la tabla de pantallas |
| C05 | Corrección de fallos propios de esta documentación (tema inicial, `Toast` vs. notificación nativa, `adapterType` obligatorio, hoja de ruta H2/H3) | **Hecho** | Changelog puntos 2 y 4; fila 5 de la tabla de pantallas; hoja de ruta de arriba |

### B01–B12 — compromiso de alcance H2/H3

El correo pedía un compromiso de alcance, no solo una lista de pendientes.
Esta tabla junta lo que ya estaba marcado "Pendiente" repartido por el
README (tabla de pantallas, filas 7–12/14/19/20, y la hoja de ruta de
arriba) en una única numeración B01–B12, con su hito ya confirmado —
ninguno de los doce es una promesa nueva, son los mismos compromisos que
ya aparecían sueltos, solo puestos juntos como pide el correo.

| # | Tema | Especificacion.md | Hito | Nota |
|---|---|---|---|---|
| B01 | Resultado completo: animación de "momento premio" + coreografía | §16.5, §16.11 | **H2** | Resultado básico (5 variantes) ya maquetado; falta la animación en sí |
| B02 | Resultado: variante UDP | §16.5 | **H2** | Depende de B01 (misma pantalla) |
| B03 | Sistema de gráficas: adaptación a impresión | §19.4 | **H2** | Color+patrón ya resuelto (§16.7); falta impresión |
| B04 | Detalles técnicos (incl. `ntttcp.exe` como único sitio previsto), y su tratamiento como panel lateral en ventana amplia (§16.9, >1400px) | §16.6, §16.9 | **H2** | Botón ya presente y deshabilitado con aviso "Hito H2". El breakpoint de amplia ya existe (`RESPONSIVE-I18N.md` §1), pero el panel lateral en sí no se construye sin el contenido de Detalles técnicos que mostraría — hacerlo ahora sería una cáscara vacía, no una pantalla más fiel a la spec. |
| B05 | Catálogo de errores: diálogo de demo propio para `NB-CONN-004` en la maqueta; resto de causas de firewall (`NB-FW-001/003/004/005`) sin escenario interactivo | §14.4, §21.2 | **H2** | Catálogo ya corregido y con tabla es/en (A09, `AVISOS.md`); el texto final de `NB-CONN-004` y `NB-FW-002` (título, descripción, acciones) ya está confirmado contra §14.4/§21.2 y aplicado — lo que queda para H2 es construir el resto de diálogos interactivos, no rellenar más texto |
| B06 | Accesibilidad ampliada: auditoría con lector de pantalla real (NVDA/JAWS/VoiceOver) | §16.11 | **H2** | C04 (esta ronda) resolvió foco/teclado/no-color por código y revisión manual; falta la verificación con AT real, que no puede hacerse con Playwright |
| B07 | Escalado de texto real (115%/130%) | §16.9 | **H2** | Diseño ya previsto en `RESPONSIVE-I18N.md`; falta la funcionalidad |
| B08 | Exportación PDF/JSON/CSV | §20 | **H3** | Corregido de H2→H3 en esta ronda (C05) — la propia spec lo marca `[H3]` |
| B09 | Opciones avanzadas | §17 | **H3** | Referenciado como texto en el selector |
| B10 | Historial/evolución | §19 | **H3** | Placeholder ya navegable |
| B11 | Ajustes completo | §23 | **H3** | Tema (Sistema/Claro/Oscuro) ya es el real; resto placeholder |
| B12 | App-icon/tray-icon, bandeja del sistema, actualizador | §16.12 y hoja de ruta | **H3** | Activos de producto/infraestructura, no de este sistema de diseño — cada uno se resuelve cuando le toque, no aquí |

### D01–D04 — decisiones señaladas como fuera del rol de diseño

La propia auditoría marca estos cuatro puntos como decisiones que no le
corresponden al diseñador — se recogen aquí para que quede constancia de
que se han visto y no se han decidido por cuenta propia, cada uno con
dónde ya estaba (o queda) señalado:

- **D01 — Secuencia `PAIRING`/`PRECHECK`.** El orden exacto entre
  emparejamiento y comprobaciones locales (§9.1, §10.2) está documentado
  tal y como lo describe `Especificacion.md` en `FLUJOS.md` (filas 4-5 de
  la tabla de estados) — pero si la implementación real necesita otro
  orden (p. ej. por cómo esté hecha la capa de red en Tauri/Rust), ese
  cambio es de quien construye el flujo, no del diseño visual. No se ha
  forzado ningún orden "porque queda mejor".
- **D02 — Degradado de la barra de título.** Ya señalado en el changelog,
  punto 1: el fondo de `TitleBar.svelte` usa un degradado sutil en vez del
  plano opaco `--solid` que pide literalmente §16.13. Es una decisión de
  diseño visual ya aprobada en mockups anteriores que contradice la letra
  de la spec — **pendiente de aprobación explícita** de seguir así o
  volver al plano opaco. No se resuelve aquí unilateralmente.
- **D03 — `Historias.md`.** Sigue sin haber llegado (nota al principio de
  este README). Es contenido de producto/negocio (los "por qué" detrás de
  cada pantalla), no una decisión de sistema de diseño — nada de lo
  construido aquí lo ha rellenado por intuición ni ha inventado historias
  de usuario para justificar decisiones visuales.
- **D04 — Alcance por hitos H1→H2→H3.** El orden general (fila 22 de la
  tabla de pantallas) está confirmado contra §26 de `Especificacion.md`; la
  tabla B de arriba es el desglose punto por punto de ese orden. Qué entra
  exactamente en cada hito, más allá de lo que la propia spec ya etiqueta,
  es una decisión de planificación de producto — este documento solo
  refleja lo ya confirmado, no propone recortes ni añadidos de alcance por
  su cuenta.
