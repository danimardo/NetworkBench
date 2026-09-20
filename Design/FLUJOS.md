# NetworkBench — mapa de flujo completo (H1)

Recorrido de una prueba de extremo a extremo, en **los dos equipos a la
vez** (iniciador y receptor), con los caminos alternativos. Responde al
apartado 2 del correo de ampliación ("primera prioridad"). Referencia
cruzada: `Especificacion.md` §10.1 (máquina de estados), §9 (emparejamiento),
§10.2–§10.8 (sesión).

Cómo leer esto: cada fila es un **momento** del recorrido. La columna
"Pantalla / componente" apunta al fichero Svelte o a `maqueta-navegable.html`
si ya existe una vista construida; si no, apunta a `PANTALLAS-H1.md` donde
se documenta qué cambia sin necesidad de una maqueta nueva (el correo lo
permite explícitamente: "no necesitamos una pantalla diferente para cada
estado... sí necesitamos que quede documentado qué cambia").

---

## 0. Máquina de estados (§10.1), tal cual, para referencia rápida

```
IDLE
 └─ DISCOVERING            (solo iniciador, pantalla de descubrimiento abierta)
     └─ CONNECTING         TCP + TLS + HELLO
         └─ PAIRING        solo si el peer es desconocido (§9.1)
             └─ PRECHECK   comprobaciones locales del iniciador (§10.2)
                 └─ WAITING_FOR_ACCEPTANCE   REQUEST enviado; el receptor ve el diálogo
                     └─ PREPARING            comprobaciones conjuntas (§10.5)
                         └─ RUNNING_FORWARD  A→B
                             └─ SWITCHING_DIRECTION   1 s
                                 └─ RUNNING_REVERSE   B→A
                                     └─ ANALYZING     cálculo de métricas e interpretación
                                         └─ COMPLETED
Terminales alternativos: FAILED (errorCode) · CANCELLED (origen) · DISCONNECTED → FAILED NB-CONN-005
```

En plan de una sola dirección se omite `SWITCHING_DIRECTION` y la dirección
no solicitada. En plan simultáneo (§17, H3) hay un único `RUNNING_BOTH`.

---

## 1. Recorrido feliz — iniciador (DESKTOP-DANIEL) y receptor (SERVER-01)

| # | Estado | DESKTOP-DANIEL (iniciador) ve | SERVER-01 (receptor) ve | Pantalla / componente |
|---|---|---|---|---|
| 1 | `IDLE` | Inicio: cabecera de "este equipo", lista de equipos descubiertos | Inicio, igual — cualquier instancia puede ser cualquiera de los dos roles (§5.4) | `maqueta-navegable.html` pane Inicio |
| 2 | `IDLE`→clic en una tarjeta, o "Analizar conexión" con un único candidato | Selector de equipo: tarjeta resumen del peer, "Prueba estándar", "Iniciar prueba" | (sin cambios todavía) | §4 del correo — selector, ver más abajo |
| 3 | `CONNECTING` | "Conectando con SERVER-01…" breve (TLS+HELLO, normalmente < 1 s, sin pantalla propia si es instantáneo; si tarda, mismo patrón de `ChecklistItem` en `running`) | (nada visible todavía: el receptor no sabe que alguien se está conectando hasta `HELLO`) | ver `PANTALLAS-H1.md` § Selector |
| 4 | `PAIRING` (solo la 1ª vez con ese peer) | "Esperando confirmación en SERVER-01… Código: **123 456**" | Diálogo combinado emparejamiento+aceptación con el mismo código y los datos de la prueba (§9.1.5) | §5 del correo — ver más abajo |
| 5 | `PRECHECK` | Comprobaciones locales silenciosas (motor, adaptador, IP, versión, disco — §10.2); si todas pasan no hay pantalla propia, se encadena directo a `WAITING_FOR_ACCEPTANCE` | (sin cambios) | — (auditoría v3, A05: estas 5 comprobaciones se muestran igualmente, ya resueltas, en el primer grupo de la checklist de la fila 7 — por transparencia de qué se comprobó, no porque `PRECHECK` gane una pantalla propia) |
| 6 | `WAITING_FOR_ACCEPTANCE` | "Esperando a que SERVER-01 acepte…" + botón Cancelar | Diálogo de aceptación (§10.4) — si ya son conocidos, sin el bloque de código | §5 del correo |
| 7 | `PREPARING` | Checklist de comprobaciones conjuntas (§10.5), con el grupo local de la fila 5 arriba, ya resuelto | Checklist idéntico (misma pantalla en los dos extremos) | §6 del correo — ver más abajo (auditoría v3, A05: dos grupos explícitos, ver `maqueta-navegable.html`) |
| 8 | `RUNNING_FORWARD` (A→B) | Calentamiento sin duración estimable, luego Ejecución: dos tarjetas de equipo + flecha (§16.4), velocidad instantánea, gráfica, "Probando envío desde DESKTOP-DANIEL" | Ejecución: misma pantalla, "Recibiendo prueba desde DESKTOP-DANIEL" | §6 del correo — ver más abajo (auditoría v3, A05: composición de tarjetas+flecha, `ExecutionLink.svelte`) |
| 9 | `SWITCHING_DIRECTION` (1 s) | "Cambiando de dirección" — enfriamiento de A→B y calentamiento de B→A en un único tramo sin duración estimable | igual | — (auditoría v3, A05) |
| 10 | `RUNNING_REVERSE` (B→A) | "Recibiendo prueba desde SERVER-01"; la flecha entre las dos tarjetas se invierte (espejo del propio SVG, nunca reordena las tarjetas) | "Probando envío desde SERVER-01" | igual pantalla que #8, roles de texto invertidos |
| 11 | `ANALYZING` | "Analizando resultados…" (barra indeterminada) | igual — el receptor puede tardar hasta 10 s más si `SESSION_RESULT` no ha llegado (§10.7) | — |
| 12 | `COMPLETED` | Resultado básico: cifra principal (% de la capacidad del enlace, auditoría v3 A06), veredicto, acciones | Resultado básico — mismo `SessionResult`, mismo veredicto; solo cambia "este equipo"/"el otro equipo" (§5.6) | §7 del correo — ver más abajo, y `README.md` § "Resultado — regla de veredicto (§13, A06)" |

**Auditoría v3 (A02):** la fila 2 solo cubre el caso feliz de un único
candidato o una tarjeta concreta. Con 0 o con varios candidatos
seleccionables, "Analizar conexión" no entra en `IDLE`→`CONNECTING` en
absoluto — se queda en Inicio (ver `PANTALLAS-H1.md`, sección "'Analizar
conexión' con 0/1/varios candidatos" para la tabla completa).

---

## 2. Caminos alternativos

Cada fila indica **desde qué estado** puede ocurrir, qué ve cada lado y a
qué estado terminal se llega. Todos vuelven a Inicio tras cerrar el aviso,
salvo que se indique otra cosa.

| Camino | Desde | Quién lo origina | Lo que ve el que lo origina | Lo que ve el otro | Estado terminal / código |
|---|---|---|---|---|---|
| **Rechazo explícito** | `WAITING_FOR_ACCEPTANCE` | Receptor pulsa "Rechazar" | Diálogo se cierra, sin aviso adicional | `NB-PEER-001` ("SERVER-01 ha rechazado la prueba"), severidad `info` | `FAILED` NB-PEER-001 |
| **Código no coincide** | `PAIRING` | Receptor pulsa "No coincide" | `NB-PEER-005`, severidad `error`, acción "Ver detalles" | Diálogo se cierra | `FAILED` NB-PEER-005 |
| **Sin respuesta (emparejamiento)** | `PAIRING` | Nadie responde en 60 s | mismo trato que rechazo, con texto de timeout | mismo diálogo se cierra solo | `FAILED` (equivalente a rechazo, sin código específico propio — se resuelve como `NB-PEER-002` en el iniciador) |
| **Sin respuesta (aceptación)** | `WAITING_FOR_ACCEPTANCE` | Nadie responde en 60 s | `NB-PEER-002` ("Nadie ha respondido en el otro equipo"), acción "Reintentar" | El diálogo de aceptación se cierra solo, sin aviso adicional (nunca hubo elección) | `FAILED` NB-PEER-002 |
| **Peer ocupado** | Antes de `WAITING_FOR_ACCEPTANCE` (respuesta inmediata) | El receptor ya tiene una sesión activa (§5.5) | `NB-PEER-003` ("El otro equipo está realizando otra prueba"), acción "Reintentar" | (nada: la solicitud entrante se rechazó automáticamente, sin interrumpir su sesión en curso) | `FAILED` NB-PEER-003 |
| **Error de conexión inicial** | `CONNECTING` | Equipo apagado / app cerrada / sin ruta / firewall | `NB-CONN-001/002/004` según el caso (§21.2), acciones reintentar / introducir IP / detalles | — (el receptor nunca se entera: no hubo `HELLO`) | `FAILED` NB-CONN-00x |
| **Versión incompatible** | `CONNECTING`→`PRECHECK` | Negociación de protocolo tras `HELLO` (§8.4) | `NB-VERSION-001` con qué equipo debe actualizar, acción "Actualizar app" | mismo aviso, mismo código, en su propio idioma | `FAILED` NB-VERSION-001 |
| **Identidad cambiada** | `PAIRING` | El peer conocido se presenta con otra huella (§9.4) | Aviso "La identidad de SERVER-01 ha cambiado" integrado en el propio diálogo de emparejamiento, no un error aparte | — | continúa como `PAIRING` normal si se confirma; si no, como "código no coincide" |
| **Cancelación local** | Cualquiera entre `CONNECTING` y `ANALYZING` | El usuario pulsa "Cancelar prueba" en cualquiera de los dos equipos | Confirmación si hay resultados parciales que ofrecer guardar (§10.8.5); vuelve a Inicio | "La prueba ha sido cancelada desde SERVER-01" (o el nombre que corresponda) | `CANCELLED {origin:"local"}` en quien cancela, `{origin:"remote"}` en el otro |
| **Cancelación implícita** | Cualquiera | Cerrar la app / apagar / `BYE` del peer | (nada: la app ya se está cerrando) | mismo trato que cancelación remota | `CANCELLED {origin:"remote", reason:"error"}` |
| **Pérdida de canal, con recuperación** | `RUNNING_*` | Wi-Fi/VPN caen < 15 s | "Reconectando…" sustituye la velocidad instantánea (§8.5); si se recupera, la dirección continúa y las muestras perdidas quedan como huecos | igual en el otro extremo | vuelve a `RUNNING_*` normal (auditoría v3, A05: ahora maquetado en `maqueta-navegable.html` — "Cancelar prueba" sigue accesible durante todo el tramo, §16.4) |
| **Pérdida de canal, sin recuperación** | `RUNNING_*` | Wi-Fi/VPN caen ≥ 15 s, o algún `ntttcp` terminó con error durante la reconexión | `NB-CONN-005`, acciones "Repetir prueba"/"Comprobar conexión". Las direcciones ya completas se conservan y la sesión se guarda **incompleta** (§19.3, H3) | igual | `FAILED` NB-CONN-005 (auditoría v3, A06: ahora también maquetado como variante `incomplete` de Resultado — distinta de "cancelada", que es una decisión del usuario y no un fallo — ver `README.md` § "Resultado — regla de veredicto") |
| **Fallo de motor** | `PREPARING` o `RUNNING_*` | `ntttcp.exe` ausente/alterado/no arranca/termina con error (§21.2 NB-ENGINE-00x) | aviso correspondiente con "Reintentar"/"Ver detalles" | igual, con el mismo código (viene de `ENGINE_FAILED` del extremo que falló) | `FAILED` NB-ENGINE-00x |
| **Firewall bloquea** | `PREPARING` | Sondeo de puerto falla (§10.5) | `NB-FW-001/002/003/004/005` según el diagnóstico (§14.4, H2) | igual | `FAILED` NB-FW-00x |

---

## 3. Notas para quien lo implemente

- **Ambos extremos corren la misma máquina de estados** y todo cambio se
  propaga por el canal de control antes de reflejarse en la UI del otro
  lado (§10.1) — no hay "vista de iniciador" y "vista de receptor" como
  pantallas distintas salvo en tres momentos puntuales: el diálogo de
  aceptación (§10.4, solo el receptor lo ve), el texto de espera durante
  `WAITING_FOR_ACCEPTANCE` (solo el iniciador) y qué frase de fase usa
  "envío"/"recepción" durante `RUNNING_*` (según el rol en esa dirección
  concreta, que se invierte entre A→B y B→A).
- **Ninguna transición debe dejar una pantalla "colgada"**: si el otro
  extremo cancela, cierra la app o pierde la conexión sin recuperación,
  quien queda esperando (diálogo de aceptación abierto, `PREPARING`,
  `RUNNING_*`) tiene que recibir el aviso correspondiente de esta tabla y
  volver a Inicio — no quedarse en un estado sin salida.
- **La barra lateral se deshabilita** (Historial, Ajustes) desde que se
  entra en `CONNECTING` (auditoría v3, A07 — antes se bloqueaba desde
  `PREPARING`, dejando todo el tramo `CONNECTING`→`PAIRING`→
  `WAITING_FOR_ACCEPTANCE` con la barra lateral operativa pese a que la
  sesión ya está en curso) hasta llegar a `COMPLETED`/`FAILED`/`CANCELLED`
  (§16.1) — la propia pantalla de Resultado ya es ese estado terminal, así
  que ahí se desbloquea de nuevo, no se queda bloqueada mostrando el
  resultado. "Inicio" es la única excepción, y no porque quede inerte
  como el resto: pulsarlo durante una sesión activa abre una confirmación
  ("¿Salir de la prueba en curso?") en vez de navegar directo — es el
  único punto de la barra al que sí tiene sentido poder ir (para
  abandonar la prueba), así que en vez de bloquearlo se le pone una
  puerta, para que no pueda abandonarse el recorrido sin querer. Ya
  resuelto en `Sidebar.svelte` vía la prop `disabled` + `Tooltip`
  (pendiente extender allí la confirmación de "Inicio", ver C04/A07), y
  en `maqueta-navegable.html` vía `<button disabled>` + `data-tt` + el
  overlay `confirm-leave` (ver su `<script>`, `updateSidebarLock` y el
  estado `sessionActive`).
- Los códigos de error de esta tabla son los de `Especificacion.md` §21.2;
  el catálogo completo (con título/causas/acciones humanos) se resuelve en
  H2, pero los que aparecen aquí son los que marca §26 como criterio de
  aceptación de H1 y por tanto sí hace falta poder mostrarlos ya, aunque
  sea con el envoltorio genérico de aviso (icono + título + descripción +
  acciones) en vez del texto final de cada uno.

**Auditoría v3 (A01), actualización de la fila 4/6:** el diálogo combinado
de emparejamiento+aceptación del receptor (§9.1.5) ya tiene composición
real en `maqueta-navegable.html` (overlay `peer-accept`), no solo esta
mención en la tabla — solicitante, IP, adaptador, tipo y duración de la
prueba, aviso de tráfico intenso, código de verificación (solo si el peer
es desconocido o su identidad ha cambiado, §9.4), casilla "Confiar siempre
en este equipo" **desmarcada por defecto**, y un único botón "Coincide y
aceptar" (o "Aceptar" si ya es conocido — sin bloque de código). Alcanza
las tres variantes que pide el correo (desconocido / conocido / identidad
cambiada) desde un único diálogo, contenido intercambiado por JS
(`PEER_VARIANTS`), igual que ya hacía el overlay "aviso". Accesible desde
los overlays del iniciador con un botón "Ver la pantalla real de
SERVER-01" — la maqueta sigue siendo una sola ventana (nota de
implementación 1 más abajo), pero ahora puede *mostrar* la pantalla del
otro extremo en vez de solo describirla en una nota de texto.

Sobre **D01** (la propia auditoría v3 lo deja como decisión de
producto+desarrollo, no del diseñador): esta composición asume que el
diálogo combinado en `PAIRING` es el único momento de decisión para un
peer desconocido — `WAITING_FOR_ACCEPTANCE` no le muestra una segunda
pantalla de aceptación después. Si se decidiera otra secuencia, es este
diálogo el que habría que reordenar, no añadir uno nuevo al lado.

**Cierre de sesión / cancelación durante la decisión del receptor:** si
quien inició la prueba cancela mientras el diálogo de aceptación sigue
abierto en el otro extremo, ese diálogo se cierra solo con un aviso
("DESKTOP-DANIEL ha cancelado la solicitud") — nunca se queda abierto
esperando una respuesta que ya no tiene sentido (mismo principio que la
nota de implementación 2 de más abajo, aplicado también a este momento).

**Escape / clic fuera en cualquier diálogo modal de sesión (A01):** ningún
overlay de `PAIRING`/`WAITING_FOR_ACCEPTANCE`/aceptación del receptor
cierra con una acción que conceda algo por error. Escape y el clic fuera
del diálogo equivalen siempre a la opción más segura — "Cancelar" en los
overlays del iniciador, "No coincide"/"Rechazar" en el del receptor —
nunca a "Coincide y aceptar". Los avisos informativos sí pueden cerrarse
así, usando su primera acción (la menos comprometedora). Implementado en
`maqueta-navegable.html` como referencia (`OVERLAY_SAFE_DISMISS`); pendiente
en el componente real `Dialog.svelte` (ver auditoría v3, C04 — trampa de
foco y cierre por teclado).

**Auditoría v3 (A05), filas 5/7/8-10 y "pérdida de canal, con
recuperación":** dos cambios en `PREPARING` y en `RUNNING_*`.

Primero, la checklist de `PREPARING` (§10.5) ahora se agrupa en dos
bloques explícitos — "Comprobaciones locales (§10.2)", que llega ya
resuelto (`PRECHECK` sigue sin pantalla propia; esto es solo mostrar por
transparencia lo que ya pasó en silencio), y "Comprobaciones conjuntas
(§10.5)", que es la que de verdad avanza en esta pantalla. Antes era una
lista plana de 4 líneas que mezclaba las dos cosas.

Segundo, `RUNNING_FORWARD`/`RUNNING_REVERSE` ganan el elemento principal
que pedía §16.4 y que antes faltaba del todo: dos tarjetas de equipo (con
modelo de NIC y velocidad de enlace) unidas por una flecha de dirección
— componente `ExecutionLink.svelte`. La tarjeta local es siempre la
misma tarjeta (nunca se reordena) y la flecha es lo único que cambia con
`SWITCHING_DIRECTION`; ver el comentario del propio componente para por
qué esto hace imposible invertir el significado de A→B/B→A al cambiar de
perspectiva, que es justo lo que señalaba la auditoría. También se
maquetan por fin, con acciones siempre accesibles ("Cancelar prueba"
nunca desaparece): el calentamiento/enfriamiento sin duración estimable
de las filas 8-10, y la fila "pérdida de canal, con recuperación" de más
arriba ("Reconectando…").

Una añadido que **no** está en la máquina de estados de §10.1 ni en
ninguna fila de esta tabla: la maqueta incluye, marcado explícitamente
como "solo en esta maqueta", un botón de demo "SERVER-01 necesita una
acción" — ilustra el caso general de "acción requerida en el otro
equipo mientras la prueba está en curso" que pide también §16.4, pero
sin un estado ni un código de error propios en la especificación tal
como está redactada hoy. No se ha inventado uno: si en H2 se decide que
esto necesita su propio estado/código, este botón de demo es el punto de
partida de la composición visual, no una propuesta de máquina de
estados.
