# NetworkBench — estados sin maqueta propia (H1)

El correo del programador lo permite explícitamente: *"no necesitamos una
pantalla diferente para cada estado (sería un número enorme)... sí
necesitamos que quede documentado qué cambia en cada uno"*. Este fichero es
esa documentación para los estados de **Inicio/descubrimiento** (§3 del
correo, `Especificacion.md` §7) y de **conexión manual** que no tienen su
propia captura en `maqueta-navegable.html` — porque son variaciones de
contenido sobre la MISMA estructura visual que sí está maquetada (la
pantalla de Inicio, la de conexión manual), no una composición distinta.

Cómo leer esto: cada fila describe qué cambia respecto al estado por
defecto que sí ves en la maqueta (Inicio con SERVER-01 / PORTATIL-MARIA /
NAS-BACKUP / LAPTOP-INVENTARIO). Estructura, tipografía, tokens y
componentes son los mismos — cambia el contenido de la sección "Equipos
disponibles" y, en algunos casos, un aviso adicional.

---

## Inicio / descubrimiento (§7.1)

| Estado | Qué cambia | Referencia |
|---|---|---|
| **Buscando** (justo al abrir la app o al refrescar) | La sección "Equipos disponibles" muestra 2-3 `Card` en estado de carga: icono sustituido por un spinner (mismo patrón que `ChecklistItem` `running`), nombre e IP sustituidos por un bloque `skeleton` (barra atenuada, `--surface-secondary`, `border-radius: var(--radius-sm)`, sin texto). El contador "N en la red" no se muestra hasta la primera respuesta mDNS. | §7.1 |
| **Ninguno encontrado** | La fila de tarjetas se sustituye por un `nb-placeholder` (el mismo patrón que hoy usan Historial/Ajustes): icono de radar, "No se ha encontrado ningún equipo todavía", con el texto secundario que ya pide la spec: *"¿No aparece? Si está en otra red, usa Conectar manualmente"* + botón que lleva a la pantalla `manual` ya maquetada. **Auditoría v3 (A02): ya maquetado de verdad** en `maqueta-navegable.html` (`#devices-empty`), no solo descrito aquí — botón de demo "Simular: 0 equipos" en Inicio. | §7.1 |
| **Descubrimiento desactivado** (ajuste "Descubrimiento automático" en off, §23/H3) | Mismo `nb-placeholder` que "ninguno encontrado", pero con el texto cambiado a "El descubrimiento automático está desactivado" + enlace directo a **Ajustes → Red** (auditoría v3, A03: decía "→ General" — el ajuste vive en Red según §23, no en General), además de "Conectar manualmente". La cabecera pierde la píldora "Visible en la red" (pasa a "No visible en la red", tono `--color-text-muted`, sin el punto verde). | §7.1, §23 |
| **Perfil de red público, sin resultados a los 5 s** | Aparece un `nb-demo-note`-como-aviso persistente (no un diálogo modal — no bloquea) encima de la fila de equipos: icono de alerta ámbar, "Puede que Windows esté bloqueando la búsqueda en esta red pública", enlace "Más información" → futura pantalla de Firewall (§14, H2). Convive con "ninguno encontrado" si sigue sin haber resultados. | §7.1 (línea sobre perfil "público") |
| **Muchos favoritos/recientes** (sección "Otros equipos") | La fila de chips (`nb-chip-row`) ya está pensada para envolver (`flex-wrap: wrap`) — con muchos elementos simplemente ocupa más líneas, sin scroll horizontal ni límite. No hace falta un componente nuevo, solo confirmar en desarrollo que no se trunca la lista (§7.3: "sin límite" en favoritos). | §7.3 |
| **Muchos equipos DESCUBIERTOS** (no solo muchos favoritos — auditoría v3, A03) | Distinto del anterior: son tarjetas completas (`.nb-card`), no chips. **Auditoría v3:** `.nb-device-row` no tenía `flex-wrap` — con más de 3-4 tarjetas se apretaban cada vez más estrechas en la misma fila en vez de pasar a una nueva. Corregido: `flex-wrap: wrap` + `flex: 1 1 220px; min-width: 220px` por tarjeta, así que con muchas se reparten solas en filas de ~4 sin script adicional ni límite. Sin demo dedicada (afectaría a la cabecera de "3 en la red" verse rara con un número inventado grande) pero verificado inyectando tarjetas de más. | §7.1 |
| **Último usado, no disponible ahora** | **Auditoría v3 (A03):** corregido — antes el bloque `nb-hero-last` ("Última prueba: SERVER-01… Repetir con SERVER-01") se mostraba siempre, sin comprobar disponibilidad, contradiciendo la propia cita del correo del §16.2 ("si… el último peer está disponible"). Ahora se oculta entero (`updateHeroLast()`) en cuanto SERVER-01 deja de estar disponible — no se llega a mostrar para luego fallar al pulsarlo. Botón de demo "Simular: SERVER-01 ya no disponible". | §7.3, §16.2, FLUJOS.md |
| **Un equipo aparece/desaparece en vivo** (mientras la pantalla de Inicio está abierta) | Aparecer: la card entra con la misma transición sutil que ya tienen las cards al hover (`transition: background/border/transform`), nunca un salto brusco de layout — se inserta al final de "Equipos disponibles". Desaparecer (30 s sin anuncio mDNS, §7.1): se retira con un fade corto; si es la card seleccionada en ese momento y el usuario está en `selector`, no se le echa de la pantalla — la sesión ya en curso no depende de que la card siga en Inicio. | §7.1 |
| **Cero adaptadores de red** (§16.2, cita literal) | **Auditoría v3 (A03), ya maquetado de verdad:** "si no hay ningún adaptador con `operStatus = up`, se sustituye el contenido por «Este equipo no tiene conexión de red» con acción «Abrir configuración de red»". Sustituye TODO el contenido bajo la cabecera (hero + Equipos disponibles + Otros equipos, `#inicio-content`), no solo la sección de equipos — no tiene sentido ofrecer "Analizar conexión" ni "Conectar manualmente" sin ningún adaptador. La píldora de cabecera pasa a roja "Sin conexión de red" (`--color-danger`), distinta del gris "Descubrimiento desactivado" (hay red, pero no se busca). "Abrir configuración de red" lleva a Ajustes → Red. Botón de demo "Simular: sin adaptadores de red". | §16.2 |
| **Primera vez, sin historial ni favoritos** | No es un estado nuevo de "Equipos disponibles" (ese ya está cubierto por "Buscando"/"Ninguno encontrado") sino de la sección "Otros equipos": la fila de chips simplemente no se renderiza (0 elementos, no un placeholder propio — un `nb-chip-row` vacío no necesita explicarse, a diferencia de "Equipos disponibles" vacío, que si necesita el `nb-placeholder` porque es la acción principal de la pantalla). El hero tampoco muestra "Última prueba…" (no hay ninguna todavía — mismo mecanismo que "último usado no disponible", `updateHeroLast()` ya cubre este caso: sin sesión previa, sin trust=cualquiera en `DEVICES` para ese id, se oculta igual). | §7.3 |
| **Nombre truncado en la consulta / IPv6** | El nombre mostrado en una tarjeta es el que anuncia el propio equipo por mDNS — si ese nombre es muy largo, `.nb-device-name` ya trunca con `text-overflow: ellipsis` (una línea, sin saltos) y el nombre completo queda disponible por `title`/tooltip al pasar el ratón, igual que hace ya cualquier `.nb-device-name` de la maqueta. Un equipo anunciado únicamente por su literal de IPv6 (sin nombre resuelto) se muestra tal cual en el campo de nombre — no se inventa un nombre, y la IP de debajo queda vacía o se omite esa línea para no duplicar la misma dirección dos veces en la tarjeta. | §7.1 |
| **Edición de alias + gestión de favoritos/recientes más allá de la estrella** | Fuera de alcance de H1 tal cual está maquetado: el icono de estrella (favorito) y el escudo (confianza) ya son interacciones de un solo gesto en la tarjeta (§7.3), pero **editar el alias** o **gestionar en bloque** (quitar varios recientes, renombrar, limpiar historial de "vistos hace N días") necesita su propia superficie — la spec ya la sitúa en Ajustes → Equipos de confianza (§9.3, H3): "tabla con nombre, alias, huella abreviada, fecha de emparejamiento, nivel, conmutador de aceptación automática, botón Eliminar… misma gestión accesible desde la ficha de un favorito". No se maqueta aparte en H1 — es la MISMA pantalla de Ajustes que ya está marcada como hito H3 en el placeholder de `data-pane="ajustes"`. | §7.3, §9.3 |
| **Regla de tiempo: "Buscando" → "Ninguno encontrado"** | No estaba documentada. Regla: se pasa a "Ninguno encontrado" a los **5 segundos** sin ninguna respuesta mDNS (mismo umbral que ya usa la fila "Perfil de red público, sin resultados a los 5 s" de esta tabla — un único temporizador cubre los dos avisos, no dos relojes distintos). Si llega una respuesta después de esos 5 s, se sale de "Ninguno encontrado" y se muestra la tarjeta normalmente — nunca hace falta refrescar a mano. | §7.1 |

### "Analizar conexión" con 0/1/varios candidatos (auditoría v3, A02)

El botón principal llevaba siempre a `selector` con SERVER-01 fijo, sin
mirar cuántos equipos había realmente para elegir. Resuelto así (maquetado
y probado en `maqueta-navegable.html`, no solo documentado — botones de
demo "Simular: 0/1/varios equipos" junto al botón):

| Candidatos seleccionables ahora mismo | Qué hace "Analizar conexión" |
|---|---|
| **0** (nada disponible/compatible, con o sin equipos listados) | No navega a ningún sitio — no hay nada que confirmar. Se queda en Inicio y desplaza/resalta la sección "Equipos disponibles", que ya muestra su propio estado vacío o sus tarjetas en `Ocupado`/`Versión incompatible`. |
| **1** (un único candidato disponible y compatible) | Se preselecciona sin preguntar y abre `selector` directamente con ese equipo — no es "adivinar", es el único resultado posible. |
| **2 o más** | **Ninguno se preselecciona.** No navega a `selector` — desplaza y resalta la sección "Equipos disponibles" (que ya ES el selector real, §16.2/§16.3) con un aviso "Hay varios equipos disponibles — elige uno de la lista", que desaparece en cuanto se elige una tarjeta. |

"Repetir con SERVER-01" (acción rápida del hero) y clicar una tarjeta
concreta siguen preseleccionando ese equipo exacto sin pasar por esta
lógica — son selecciones explícitas del usuario, no del botón genérico.

**Adaptador local automático (§10.2):** la pantalla de confirmación
(`selector`) muestra ahora una fila "Tu adaptador" de solo lectura con el
que se usará, más una etiqueta "Automático" con la regla exacta al pasar
el ratón: el adaptador con `operStatus = up` de mayor `linkSpeedBps`; en
empate, el de la ruta por defecto. Sin selector manual en H1 — es
información, no una decisión que tome aquí el usuario.

## Conexión manual (§7.2)

**Auditoría v3 (A04):** la maqueta valida de verdad nombre DNS, IPv4 e IPv6
(con o sin corchetes, con o sin `:puerto`) — antes solo aceptaba IPv4 pese
a que este mismo fichero y el texto de ayuda del campo ya prometían más
formatos. La etiqueta pasó de "Dirección IP" a "Dirección del equipo" para
dejar de contradecirse con el propio ejemplo. Sigue siendo solo una
comprobación de **formato** ("¿tiene pinta de dirección?"), no de
resolución DNS real — por eso "formato válido pero no resuelve" es un
estado del campo aparte (`data-error-kind="unresolved"`), maquetado con su
propio mensaje, distinto de "sintaxis inválida" y sin borrar el valor
introducido:

| Entrada | Comportamiento esperado | Referencia |
|---|---|---|
| Nombre DNS (`equipo.local`) | Válido; se resuelve antes de conectar (IPv4 primero, luego IPv6). | §7.2 |
| IPv6 (`fe80::1` o `[fe80::1]:7411`) | Válido con o sin corchetes; con puerto van siempre entre corchetes. | §7.2 |
| `host:puerto` / `IP:puerto` | Válido; sin puerto se usa el de control por defecto. | §7.2 |
| Ni IP ni nombre con forma válida | Error de validación de formato — maquetado (`data-error-kind="syntax"`). | §7.2 |
| Formato válido pero no resuelve por DNS | `NB-CONN-002`, distinto del genérico `NB-CONN-001` — maquetado como estado de campo aparte (`data-error-kind="unresolved"`), con botón de demo "Simular: nombre válido que no resuelve". | §7.2, §21.2 |
| Responde pero no es NetworkBench | `NB-CONN-004` — texto final ya definido (§21.2, catálogo en `AVISOS.md`: "Cambiar puerto"/"Ver detalles"); sigue sin diálogo de demo propio en esta maqueta (mismo tratamiento genérico que el resto de avisos de conexión), eso sí queda para H2. | §7.2, §21.2 |
| "Opciones avanzadas": forzar IPv4/IPv6 | No maquetado — es un control adicional (probablemente un `nb-chip` de selección) en la misma pantalla `manual`, hito H2 según prioriza el correo. | §7.2 |

---

Todo lo de este fichero usa exclusivamente componentes y tokens que ya
existen (`Card`/`nb-card`, `nb-placeholder`, `nb-chip-row`, `nb-field`,
`ChecklistItem`) — no hace falta ningún token o componente nuevo para
construir estos estados, solo el dato que los alimenta.
