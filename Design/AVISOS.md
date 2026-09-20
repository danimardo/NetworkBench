# Catálogo de avisos — H1 (auditoría v3, A09 + corrección tras revisión de código)

Cierra el punto 21 (apartado 3 de correcciones puntuales) del correo:
versión incompatible sin decir qué equipo actualizar, acciones que
faltaban en código-no-coincide, código de diagnóstico equivocado para
fallo de puertos, y el nombre del proceso `ntttcp.exe` filtrándose a
pantallas que ve el usuario final. Los cuatro están corregidos en
`maqueta-navegable.html` (objeto `SCENARIOS`, checklist de Preparación) —
este documento es el catálogo de referencia + la tabla es/en, no una
segunda copia del código.

**Corrección posterior (revisión de código del desarrollador, con
`Especificacion.md` ya completo):** tras el A09 de arriba, la revisión a
nivel de código del implementador encontró que dos entradas de la tabla
seguían sin el texto final aunque el código en sí ya fuera correcto —
`NB-FW-002` tenía código correcto pero acciones/título genéricos, y
`NB-CONN-004` seguía con el placeholder "pendiente de texto final, H2" —
porque en la ronda anterior `Especificacion.md` no estaba disponible
completo para confirmarlos. Ya lo está; ambos quedan con el texto de
§14.4/§21.2 confirmado, no una interpretación — detalle en "1. Qué
cambió" más abajo.

## 1. Qué cambió

- **Versión incompatible (`NB-VERSION-001`) ahora dice qué equipo
  actualizar.** Antes: "Actualiza uno de los dos equipos". La
  negociación de protocolo (§8.4) conoce las dos versiones tras el
  `HELLO`, así que se puede nombrar el equipo concreto — igual que ya
  hacía el aviso de tarjeta incompatible en Inicio (`A03`,
  `device-incompatible-info`), que sí era claro desde antes. Ahora los
  dos avisos siguen el mismo patrón: nombran el equipo, dan las dos
  versiones, y aclaran explícitamente que el otro equipo no hace falta
  tocarlo.
- **"El código no coincide" (`NB-PEER-005`) tiene ahora la acción "Ver
  detalles" que ya documentaba `FLUJOS.md`** (§2, fila "Código no
  coincide") pero que la maqueta no había llegado a construir — se
  quedaba solo con "Cerrar". "Ver detalles" expande, dentro del propio
  diálogo, el código que este equipo esperaba frente al que se ha
  recibido — la información concreta para distinguir "me he
  equivocado de equipo" de "esto es sospechoso". No navega a ninguna
  pantalla nueva — no existe una en H1 fuera del diálogo mismo (no
  confundir con "Detalles técnicos" de Resultado, que es de la prueba ya
  completada y sigue pendiente de H2).
- **Fallo de puerto ya no usa `NB-FW-001`.** Ese código es del canal de
  control (firewall bloqueando la conexión TCP+TLS inicial, §8) — esa
  causa ya la cubre `NB-CONN-001` de forma genérica (su descripción ya
  mencionaba el firewall como posible causa). El sondeo de puerto que
  falla en `PREPARING` (§10.5) es un momento distinto y necesita su
  propio código dentro de `NB-FW-00x`; se usa `NB-FW-002`. **Confirmado
  contra §14.4/§21.2 de `Especificacion.md`**, ya disponible completo tras
  la revisión del desarrollador: en la ronda anterior esa tabla no estaba
  disponible y `NB-FW-002` se usó como la mejor asignación posible sin
  ella delante — resulta ser exactamente el código correcto para este
  escenario ("canal de control OK, sondeo TCP al puerto NTTTCP del
  receptor falla, regla NTTTCP ausente/deshabilitada"), pero sus **acciones
  y título sí estaban mal** (se usaba "Cerrar/Reintentar" genérico en vez
  de "Configurar automáticamente/Ver instrucciones/Cancelar", que es lo
  que pide la spec) — corregido en la tabla de abajo.
- **`NB-CONN-004` tenía título y descripción sin rellenar.** Estaba
  marcado "pendiente de texto final, H2" en la ronda anterior porque
  §21.2 no estaba disponible completo entonces; ya lo está y define el
  disparador exacto ("responde algo en el puerto pero no es
  NetworkBench"), severidad y acciones (`change_port`, `show_details`) —
  texto final ya en la tabla de abajo, no genérico.
- **`ntttcp.exe` fuera de toda pantalla visible.** Aparecía en dos
  sitios: la etiqueta de la checklist de Preparación ("Motor de pruebas
  (ntttcp.exe)" → "Motor de pruebas") y la descripción del aviso de
  fallo de motor ("ntttcp.exe no se ha podido iniciar…" → "El motor de
  pruebas no se ha podido iniciar…"). No hay pantalla de "Detalles
  técnicos"/"Acerca de" en H1 (sigue deshabilitada, hito H2 — ver tabla
  de estado del README, fila 9), así que en esta ronda no existe ningún
  sitio legítimo donde mostrar el nombre del proceso — se ha quitado sin
  más, no trasladado. Las referencias a `ntttcp` en `FLUJOS.md` se
  mantienen: es documentación para quien implementa, no una pantalla de
  la app, y el correo solo pide quitarlo de lo que ve el usuario final.

## 2. Catálogo completo H1 — tabla es/en

"Final" aquí significa: es el texto que usan `maqueta-navegable.html` y
los componentes reales para H1 — no una traducción de producción
verificada por un hablante nativo ni un sistema de claves i18n (eso sigue
pendiente, ver `RESPONSIVE-I18N.md` §2). Donde el propio español todavía
no tiene texto final (una fila), el inglés tampoco lo tiene — están
marcadas como tal en vez de rellenarse con una traducción de un texto que
ni siquiera es definitivo en el idioma original.

| Código | Título (es) | Descripción (es) | Title (en) | Description (en) | Acciones | Severidad |
|---|---|---|---|---|---|---|
| `NB-CONN-001` | No se puede conectar con SERVER-01 | Puede estar apagado, con la app cerrada, o el firewall está bloqueando la conexión. | Can't connect to SERVER-01 | It may be off, have the app closed, or a firewall may be blocking the connection. | Cerrar / Reintentar | error |
| `NB-CONN-002` | (sin diálogo propio — estado de campo) | El formato es correcto, pero ese nombre no se ha podido resolver. | (no dialog — field state) | The format is correct, but that name couldn't be resolved. | Corregir el campo (sin diálogo que cerrar) | error |
| `NB-CONN-004` | Eso no es NetworkBench en SERVER-01 | Algo responde en esa dirección y puerto, pero no habla el protocolo de NetworkBench — puede que el puerto sea de otro programa, o que se haya escrito mal (el puerto por defecto es 7411, §8.2). Cambia el puerto o revisa los detalles técnicos de lo recibido. | That's not NetworkBench on SERVER-01 | Something answered at that address and port, but it isn't speaking the NetworkBench protocol — it may be a different program using that port, or a typo (the default port is 7411, §8.2). Change the port or check the technical details of what was received. | Cambiar puerto / Ver detalles | error |
| `NB-CONN-005` | Se ha perdido la conexión | La red ha fallado 15 segundos o más y no se ha podido recuperar. Las direcciones que ya se habían completado se conservan. | Connection lost | The network was down for 15 seconds or more and couldn't recover. Directions that had already completed are kept. | Cerrar / Repetir prueba | error |
| `NB-VERSION-001` | Versión incompatible | SERVER-01 usa NetworkBench 2.1, una versión anterior e incompatible con la tuya (2.3). Actualiza NetworkBench en SERVER-01 para poder conectar — no hace falta actualizar este equipo. | Incompatible version | SERVER-01 is running NetworkBench 2.1, an earlier version incompatible with yours (2.3). Update NetworkBench on SERVER-01 to connect — no need to update this computer. | Entendido | error |
| `NB-VERSION-001` *(variante Inicio)* | LAPTOP-INVENTARIO usa una versión incompatible | Este equipo tiene instalada una versión de NetworkBench distinta e incompatible con la tuya. Actualiza NetworkBench en LAPTOP-INVENTARIO para poder hacer una prueba con él — no hace falta actualizar este equipo. | LAPTOP-INVENTARIO is using an incompatible version | This device has a different, incompatible version of NetworkBench installed. Update NetworkBench on LAPTOP-INVENTARIO to run a test with it — no need to update this computer. | Entendido | error |
| `NB-PEER-001` | SERVER-01 ha rechazado la prueba | No se ha realizado ninguna prueba. Puedes intentarlo de nuevo cuando quieras. | SERVER-01 declined the test | No test was run. You can try again anytime. | Cerrar | info |
| `NB-PEER-002` | Nadie ha respondido en SERVER-01 | Puede que no esté atento a la pantalla ahora mismo. Puedes intentarlo de nuevo. | No response from SERVER-01 | They may not be looking at the screen right now. You can try again. | Cerrar / Reintentar | advertencia |
| `NB-PEER-003` | SERVER-01 está ocupado | Ya está realizando otra prueba con otro equipo. Puedes intentarlo de nuevo en unos segundos. | SERVER-01 is busy | It's already running a test with another device. You can try again in a few seconds. | Cerrar / Reintentar | advertencia |
| `NB-PEER-005` | El código no coincide | Por seguridad, la prueba no continúa. Comprueba que estáis emparejando con el equipo correcto e inténtalo de nuevo. | The code doesn't match | For security, the test won't continue. Check that you're pairing with the right device and try again. | **Ver detalles** (nuevo, A09) / Cerrar | error |
| `NB-FW-002` | El Firewall de Windows de SERVER-01 puede estar bloqueando la prueba | El canal de control funciona, pero el sondeo de conexión al puerto de pruebas de SERVER-01 falla — la regla de firewall de NetworkBench no existe o está desactivada en ese equipo (§14.4). | Windows Firewall on SERVER-01 might be blocking the test | The control channel works, but the connection probe to SERVER-01's test port fails — NetworkBench's firewall rule is missing or disabled on that device (§14.4). | Configurar automáticamente / Ver instrucciones / Cancelar | error |
| `NB-ENGINE-001` | El motor de pruebas ha fallado | El motor de pruebas no se ha podido iniciar en uno de los dos equipos. | The test engine failed | The test engine couldn't start on one of the two devices. | Cerrar / Reintentar | error |
| `CANCELLED · origin local` | Prueba cancelada | Has cancelado la prueba. | Test cancelled | You cancelled the test. | Volver a Inicio | info |
| `CANCELLED · origin remota` | DESKTOP-DANIEL ha cancelado la solicitud | Se ha cancelado antes de que respondieras. No se ha hecho ninguna prueba. | DESKTOP-DANIEL cancelled the request | It was cancelled before you responded. No test was run. | Entendido | info |

**Notas de la tabla:**

- Los códigos `NB-PEER-004` y `NB-CONN-003` no aparecen: no tienen aviso
  propio en H1 (ver `FLUJOS.md`, fila "Sin respuesta (emparejamiento)" —
  se resuelve reutilizando `NB-PEER-002`, no un código nuevo). No es un
  hueco de esta tabla, es que no existen todavía como avisos
  independientes.
- `NB-CONN-002` no tiene diálogo modal propio — es un estado de campo
  inline en la conexión manual (`data-error-kind="unresolved"`, ver
  `PANTALLAS-H1.md`), por eso no tiene fila de "acciones" como el resto.
- Las variantes "cancelada" no llevan prefijo `NB-` porque no son un
  fallo (`FAILED`) sino un estado terminal distinto (`CANCELLED`, con
  `origin` en vez de `errorCode`) — mismo criterio que usa la propia
  máquina de estados de §10.1 (ver `FLUJOS.md` §0).

## Verificación

- `node scripts/verify-tokens.mjs` — sin regresión.
- Regresión completa `/tmp/verify-a01.mjs` … `/tmp/verify-a08.mjs` — sin
  errores de consola tras los cambios de A09.
- `/tmp/verify-a09.mjs` — comprueba: texto de "versión incompatible"
  nombra a SERVER-01 y no dice "uno de los dos"; "Ver detalles" expande
  el bloque de diagnóstico sin cerrar el diálogo ni navegar, y el bloque
  empieza oculto en cada apertura del aviso; el código de "firewall
  bloquea" ya no es `NB-FW-001`; no aparece la cadena `ntttcp` en ningún
  texto de checklist ni de aviso visible.
- `/tmp/verify-especificacion-fixes.mjs` (nuevo, corrección tras revisión
  de código con `Especificacion.md` completo) — comprueba: el diálogo de
  "firewall bloquea" (`aviso:firewall`) muestra el título exacto de
  §14.4 y los tres botones "Configurar automáticamente / Ver
  instrucciones / Cancelar" (ya no "Cerrar/Reintentar"); la cifra grande
  de Resultado es la velocidad y no el %, en las cinco variantes; ningún
  texto de la maqueta ni de este documento sigue citando el umbral 50 %
  como frontera de veredicto.
