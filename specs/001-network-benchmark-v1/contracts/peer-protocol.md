# Contrato del protocolo entre peers

**Protocolo**: protocolo entre peers NetworkBench  
**Versión**: v1 provisional; se congela solo tras G2/G3  
**Transporte**: TCP + TLS 1.3 con certificados mutuos de instancia

## Confianza y conexión

- Una conexión de control por sesión activa, abierta por el iniciador; sin conexión ociosa persistente.
- mDNS, hostname, IP, `instanceId` y nombre visible nunca establecen confianza.
- Los peers conocidos se autentican por huella. Una huella distinta devuelve el peer a desconocido
  hasta completar la verificación humana.
- El certificado de un peer nuevo solo se acepta dentro del flujo provisional acotado de G2; no
  autoriza el benchmark antes de comparar el código y obtener consentimiento local.
- El puerto de control provisional es 7411, configurable entre 1024–65535.

## Trama

```text
uint32 big-endian bodyLength
UTF-8 JSON body
```

Tamaño máximo provisional: 1 MiB. G3 debe demostrar que el resultado acotado cabe o definir
segmentación con límites por parte y total. Tamaño excesivo, UTF-8/JSON o envoltorio inválido y
mensaje ilegal en el estado actual cierran la conexión con diagnóstico local tipado.

## Envoltorio

```text
{
  type: tipo de mensaje cerrado,
  id: UUID del mensaje,
  sessionId: UUID de sesión o null donde se permita,
  ts: timestamp informativo UTC ISO 8601,
  inReplyTo?: UUID del mensaje solicitado,
  payload: objeto
}
```

Autorización y timeout nunca dependen del reloj remoto `ts`. Los campos opcionales desconocidos
se ignoran para evolución compatible; variantes/tipos obligatorios desconocidos se rechazan. Los
campos del envoltorio son cerrados. Los ids se deduplican en una ventana de sesión acotada.

## Negociación

`HELLO` incluye `protocolVersion`, `protocolMin`, versión de app, id de instancia, nombre acotado,
resumen de plataforma y ocupado. Se elige `min(maxSupportedA, maxSupportedB)` si no baja de ningún
mínimo; de lo contrario ambos terminan con `NB-VERSION-001`.

Añadir un campo opcional sin cambio semántico es compatible. Cambiar significado, unidades, estado
requerido o autorización incrementa la versión del protocolo.

## Catálogo de mensajes

| Tipo | Dirección | Estado | Payload / respuesta |
|---|---|---|---|
| `HELLO` | both | post-TLS | version and identity metadata; reciprocal `HELLO` |
| `PAIR_REQUEST` | initiator → responder | pairing | no free payload; `PAIR_RESULT` |
| `PAIR_RESULT` | responder → initiator | pairing | accepted + closed reason |
| `REQUEST` | initiator → responder | precheck | validated plan, estimate, interface; `RESPONSE` |
| `RESPONSE` | responder → initiator | waiting | accepted + closed reason + interface |
| `PREPARE` | initiator → peer | preparing | direction (`forward`, `reverse`, `both_sequential`, `both_simultaneous`), bounded port block, structured engine parameters |
| `PREPARE_RESULT` | peer → initiator | preparing | ok, bounded checks, optional suggested port |
| `READY` | engine receiver → peer | preparing | direction + opaque owned process reference; G2 decides PID exposure |
| `START` | initiator → peer | preparing | direction, start marker, warmup/measure. En plan simultáneo activa `RUNNING_BOTH`. El marcador NO presupone relojes sincronizados: negociación cerrada, ver más abajo (constitución VII) |
| `STARTED` | peer → initiator | running | direction |
| `SAMPLE` | both | running | bounded observation; no response |
| `ENGINE_DONE` | both | running | direction, role, bounded engine result |
| `ENGINE_FAILED` | both | running | direction, role, NB code, bounded sanitized tail |
| `SESSION_RESULT` | initiator → responder | analyzing | versioned result or G3 segmentation; `SESSION_ACK` |
| `SESSION_ACK` | responder → initiator | analyzing | result id/hash and persistence outcome |
| `CANCEL` | both | any non-terminal | closed reason + optional NB code; `CANCEL_ACK` |
| `CANCEL_ACK` | both | cancelling/terminal | idempotent terminal observation |
| `HEARTBEAT` | both | active | nonce/sequence defined by G2; must not echo indefinitely |
| `ERROR` | both | applicable active state | safe NB code + bounded safe fields |
| `BYE` | both | active/terminal | closed reason |

`FIREWALL_FIX_REQUEST/RESULT` se añade en H2 solo después de que G2 defina solicitud remota y
consentimiento local. El mensaje de red nunca concede elevación por sí solo.

## Negociación de inicio (G2, constitución VII)

**Cerrado el 2026-09-24** en `src-tauri/src/control/protocol.rs`, con evidencia en
`VALIDACION.md`. No se compara ningún reloj de pared entre los dos equipos.

1. Cada extremo mide el RTT enviando un `HEARTBEAT` con un nonce y esperando su eco
   con el mismo nonce.
2. A partir del RTT, ambos calculan **el mismo** `start_delay_ms` (`3 × RTT`, acotado
   entre 100 ms y 2000 ms) y `tolerance_ms` (`RTT / 2`, acotado entre 50 ms y 1000 ms).
   El cálculo es puro y determinista: mismo RTT, mismo plan, en cualquiera de los dos
   extremos.
3. `START` lleva ese `start_delay_ms`/`tolerance_ms` ya calculado. Cada extremo arranca
   en `su propio` instante de recepción de `START` + `start_delay_ms`, medido con su
   reloj monótono local (`Instant`), nunca contra una marca de tiempo del otro.
4. Cada extremo compara **su propio** arranque real contra el esperado. Si el desvío
   supera `tolerance_ms`, se declara `Degradado { desvio_ms }`, nunca se oculta
   (FR-025). No se compara el arranque de un extremo contra el del otro: eso exigiría
   la sincronización de reloj que este diseño evita.

## Emparejamiento y aceptación

El código de seis dígitos deriva de las huellas ordenadas y `sessionId`, caduca a los 60 segundos y
se muestra en ambos extremos. G2 debe fijar si el plan se intercambia antes o dentro del diálogo
combinado para que el usuario vea exactamente lo que autoriza. Rechazar cancela emparejamiento y
solicitud.

Los estados son desconocido, conocido, confiable y confiable+autoaceptación. La autoaceptación
empieza desactivada, solo aplica a planes válidos allowlisted y mantiene cancelación local.

## Timeouts y recuperación

| Condición | Límite |
|---|---:|
| TLS + HELLO | 5 s |
| Generic request/response | 10 s unless message defines another |
| Pairing or user acceptance | 60 s |
| Heartbeat interval | 1 s |
| No heartbeat before reconnecting UI state | 5 s |
| Reconnect interval/deadline | every 2 s, maximum 15 s |
| Wait for `CANCEL_ACK` | maximum 2 s |

Reconectar exige misma huella y `sessionId` activo. El motor solo continúa si ambos procesos propios
siguen válidos; muestras perdidas se marcan como huecos. Una dirección parcial nunca se reanuda
como continua. Agotar el plazo produce `NB-CONN-005` y limpieza.

## Límites provisionales contra abuso

| Límite | Valor |
|---|---:|
| Pending requests per peer | 1 |
| Block after user rejection | 30 s |
| Incoming requests total | 5/minute |
| Failed TLS from one IP | 10, then 60 s block |
| Simultaneous incoming TCP connections | 8 |
| Frame | 1 MiB pending G3 |

Los contadores son acotados y caducan. El diagnóstico usa id local opaco y contexto saneado; no
registra huellas completas, ids de sesión ni payloads pese a la redacción antigua de `Historias.md`.

## Estado e idempotencia

- Cada mensaje define emisor, receptor y estados legales; mensajes ilegales no tienen efectos.
- `REQUEST`, `PREPARE`, `START`, `SESSION_RESULT` y firewall vinculan respuesta mediante
  `inReplyTo` e id estable de operación.
- Duplicados de `CANCEL`, ACK, heartbeat y terminal conservan el mismo resultado.
- Lanzar procesos, cambiar confianza, persistir o elevar usa tokens de un uso o claves de
  idempotencia almacenadas.
- Las muestras pueden reordenarse en un lote acotado; las transiciones de estado nunca.

## Fixtures de cierre G2/G3

Antes de congelar v1, fixtures y pruebas de integración cubren:

1. identidad conocida/desconocida/cambiada y código coincidente/distinto/caducado;
2. plan visible antes de aceptar y plan inválido rechazado de nuevo por el receptor;
3. versiones compatibles/incompatibles y campos compatibles desconocidos;
4. cada mensaje válido, obsoleto, duplicado, sobredimensionado y fuera de estado;
5. heartbeat sin eco infinito, reconexión correcta/fallida y huecos explícitos;
6. cancelación desde cada estado/extremo y repetida sin repetir efectos;
7. resultado máximo, límites de salida cruda y framing segmentado/ajustado si procede;
8. pérdida antes/después de persistir y reconciliación/fuente deterministas.
