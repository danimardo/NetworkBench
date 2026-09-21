# Contrato IPC: WebView ↔ Rust

**Estado**: propuesto para L00; los nombres pueden normalizarse antes de la primera implementación  
**Modelo de confianza**: la entrada WebView no es confiable; una capability es necesaria pero no suficiente

## Envoltorio

Los comandos aceptan un único objeto de entrada y devuelven un resultado discriminado:

```text
Success<T> = { ok: true, value: T }
Failure    = { ok: false, error: AppError }
AppError   = {
  code: NB-*;
  severity: info | warning | error | fatal;
  messageKey: i18n key;
  issues?: [{ path, code, messageKey, safeParams? }];
  actions: closed action identifiers[];
  diagnosticId?: opaque local identifier;
}
```

Ninguna respuesta expone trazas, secretos, SQL, payloads del peer ni salida de proceso sin sanear.
Cancelación, timeout y rechazo son resultados tipados, no fallos genéricos de transporte.

## Modelo de lectura

`app.getSnapshot` devuelve una proyección inicial inmutable:

- aplicación, versión, idioma, tema y ajustes seguros para la ventana;
- información visible de la instancia local sin clave privada ni huella completa;
- proyección de sesión/solicitud actual, si existe;
- peers descubiertos/conocidos con etiquetas de confianza;
- estado de capacidades necesario para mostrar acciones disponibles;
- `revision` monotónica para ordenar eventos posteriores.

El frontend se suscribe antes de pedir el snapshot o usa un adaptador atómico de suscripción y
snapshot. Descarta eventos con `revision <= snapshot.revision`. Un hueco de revisión solicita otro
snapshot; la UI nunca reconstruye estado autoritativo mediante conjeturas.

## Grupos de comandos

Los identificadores exactos se centralizan en `src/lib/api`; los componentes no invocan strings.

### Aplicación y ventana

| Operación | Entrada | Salida / efecto | Autorización |
|---|---|---|---|
| `app.getSnapshot` | ninguna | modelo de lectura actual | capability de ventana principal |
| `app.requestClose` | acción solicitada | estado de decisión/confirmación | Rust comprueba la sesión |
| `app.confirmClose` | token de confirmación | minimizar o cancelar-limpiar-salir | token de un uso; sesión protegida |
| `window.setAction` | minimizar/maximizar/restaurar/arrastrar | estado resultante | operación allowlisted |

### Peers y emparejamiento

| Operación | Entrada | Salida / efecto | Validación |
|---|---|---|---|
| `peers.list` | filtro opcional | proyecciones | sin secretos |
| `peers.connect` | host/IP, puerto/interfaz opcionales | id del intento | DNS/IP/longitud/rango; frecuencia limitada |
| `peers.beginPairing` | id de candidato | proyección de verificación | exclusión de sesión activa |
| `peers.confirmPairing` | id + coincide/rechaza | resultado de confianza | un uso, caducidad y huella vinculada |
| `peers.update` | huella + patch alias/favorito/confianza/autoAccept | proyección actualizada | no autoaceptar peer no confiable |
| `peers.forget` | huella + token de confirmación | resultado de borrado | referencias activas protegidas |

### Sesión

| Operación | Entrada | Salida / efecto | Validación |
|---|---|---|---|
| `session.previewPlan` | peer + opciones | vista/duración del plan validado | sin efectos |
| `session.request` | token de preview | inicia solicitud | token vincula peer/plan; una sesión activa |
| `session.respond` | id + aceptar/rechazar | transición | consentimiento local y caducidad |
| `session.cancel` | id de sesión + motivo | acuse idempotente | cualquier estado no terminal |
| `session.savePartial` | id de sesión + decisión | persistido/descartado | solo dirección completa elegible |
| `session.getDetails` | id de sesión | proyección segura | resultado actual o persistido |

### Historial y exportación

| Operación | Entrada | Salida / efecto | Validación |
|---|---|---|---|
| `history.list` | página/filtro/orden acotados | página + cursor | tamaño máximo; enums de filtro |
| `history.get` | id de sesión | proyección `SessionResult` | esquema/versión comprobados |
| `history.deletePreview` | selección | cantidad/impacto + token | token vincula selección exacta |
| `history.deleteConfirm` | token | resumen del borrado | un uso, transaccional |
| `export.preview` | selección, formato, locale, anonimizar | campos/rutas propuestos | sin escritura |
| `export.write` | token + destino aprobado | resumen de archivos | token exacto; selector nativo |

### Ajustes, firewall y actualizador

| Operación | Entrada | Salida / efecto | Validación |
|---|---|---|---|
| `settings.get` | sección | preferencias seguras | sin secretos |
| `settings.update` | versión + patch tipado | ajustes actualizados | conflicto con sesión rechazado/diferido |
| `firewall.inspect` | id del conjunto deseado | estado observado | solo lectura, sin elevación |
| `firewall.requestChange` | id + token de consentimiento | resultado del helper | operación cerrada; UAC local |
| `updater.check` | motivo usuario/manual | disponible/no disponible/error | respeta desactivación |
| `updater.confirmInstall` | id + token de consentimiento | programado/instalado | auténtica; sin sesión activa |

## Eventos

Todos incluyen `revision`, `occurredAt` y payload tipado. Los de sesión incluyen `sessionId` y
secuencia monotónica por sesión.

| Evento | Finalidad | Presión/descartes |
|---|---|---|
| `app.snapshotInvalidated` | pedir snapshot tras hueco irrecuperable | combinado |
| `peers.changed` | cambio de proyección | combinado por peer |
| `pairing.changed` | código/estado/caducidad | transición sin pérdida |
| `session.changed` | estado/progreso/checks autoritativos | transición sin pérdida |
| `session.samples` | lote acotado de muestras/huecos | último lote acotado; UI ≤4 Hz |
| `session.incomingRequest` | consentimiento local | sin pérdida; una pendiente |
| `firewall.changed` | inspección/progreso helper | terminal sin pérdida; progreso combinado |
| `updater.changed` | disponibilidad/progreso/terminal | terminal sin pérdida; progreso combinado |
| `app.error` | error global recuperable tipado | acotado y deduplicado |

La UI cancela suscripciones explícitamente. Perder muestras nunca bloquea cancelación o estados.

## Matriz de capabilities

- La ventana principal recibe solo comandos listados y operaciones de ventana necesarias.
- La vista de impresión recibe datos inmutables y permiso de impresión, no APIs de sesión/ajustes.
- Ningún origen remoto obtiene acceso a comandos propios.
- Capabilities de desarrollo están separadas y ausentes en release.
- Las pruebas demuestran llamadas permitidas y que las denegadas no alcanzan el handler.

## Pruebas de contrato

1. Zod acepta cada fixture canónico Rust y rechaza variantes malformadas/desconocidas.
2. Rust rechaza rangos inválidos, estados inesperados, tokens caducados y autoridad inventada.
3. Cadenas decimales de enteros hacen round-trip sin perder precisión.
4. Uniones de error/acción son exhaustivas y las claves existen en es/en.
5. Huecos de revisión restauran snapshot; eventos obsoletos no rebobinan la UI.
6. Capabilities denegadas y permisos ausentes se prueban aparte del rechazo de negocio.
