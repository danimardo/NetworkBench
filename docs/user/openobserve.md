# Envío de diagnóstico a OpenObserve

Guía para configurar el envío opcional de registros de NetworkBench a un servidor
OpenObserve propio, y para saber qué recibe exactamente ese servidor.

**Estado de esta guía:** la sección 2 (qué hace NetworkBench, qué envía y con qué formato)
está **VERIFICADA** contra el código de la aplicación (`src-tauri/src/logging/openobserve.rs`
y `src-tauri/src/logging/mod.rs`). La sección 1 (pasos dentro de la interfaz de
OpenObserve) es **DOCUMENTADA** a partir del funcionamiento habitual de OpenObserve, no
verificada contra una instancia en marcha desde aquí: los nombres exactos de menú pueden
variar entre versiones. Confirme siempre en la página **Ingestion** de su propia
organización, que suele mostrar un ejemplo `curl` ya relleno con la URL, la organización,
el stream y la cabecera `Authorization` correctos para copiar directamente.

---

## 0. Antes de empezar

- Necesita un servidor OpenObserve **propio**, que usted mismo administre. NetworkBench no
  incluye ni proporciona ninguno.
- La función está **apagada por defecto**. Activarla es una decisión explícita: un reenvío
  opt-in hacia un servidor que usted mismo opera no es la telemetría automática hacia
  terceros que la constitución del proyecto prohíbe (principio IV, enmienda 0.8.0).
- Use siempre `https://` en la URL si el servidor es accesible por una red que no controla
  por completo: el token viaja en la cabecera `Authorization` de cada petición.

---

## 1. Configuración en OpenObserve

1. **Organización.** Puede usar la organización `default` que trae OpenObserve o crear una
   propia para NetworkBench. Anote el nombre exacto (sensible a mayúsculas/minúsculas):
   es el valor que irá en el campo **Organización** de NetworkBench.
2. **Stream.** Elija un nombre de stream para los registros de NetworkBench (por ejemplo,
   `netbench`). No hace falta crearlo a mano de antemano: la API de ingesta JSON de
   OpenObserve lo crea automáticamente en la primera petición si no existe.
3. **Token de ingesta.** En la página **Ingestion** (o equivalente, según la versión) de su
   organización, OpenObserve muestra el valor completo de la cabecera `Authorization` que
   hay que usar para ingerir datos — normalmente algo con la forma `Basic <base64>`.
   Copie ese valor **completo**, tal cual aparece, incluida la palabra `Basic`.
4. **Endpoint (solo para verificarlo, NetworkBench lo construye solo).** La API nativa de
   ingesta JSON de OpenObserve tiene la forma:

   ```
   POST {url}/api/{organización}/{stream}/_json
   ```

   NetworkBench arma esta URL a partir de los tres campos que usted rellena; no hace falta
   escribirla en ningún sitio.

---

## 2. Configuración en NetworkBench

En **Ajustes → Diagnóstico**, tarjeta **"Enviar diagnóstico a OpenObserve"**:

1. Active el interruptor.
2. Rellene los cuatro campos:
   - **URL del servidor**: la raíz de su OpenObserve, por ejemplo
     `https://openobserve.miempresa.example` (sin `/api/...` al final; NetworkBench lo añade).
   - **Organización**: la del paso 1.
   - **Stream**: el del paso 1.
   - **Token de ingesta**: el valor completo copiado en el paso 3, **tal cual**. No hay que
     anteponerle `Basic` ni volver a codificarlo: se usa exactamente como se pega.
3. Pulse **Probar conexión**. Envía un único evento de prueba y espera la respuesta;
   muestra el error exacto (estado HTTP y cuerpo, o el error de red) si algo falla, en vez
   de un mensaje genérico.
4. Los campos se guardan solos al salir de cada uno (igual que el resto de Ajustes); el
   interruptor se aplica al instante.

El envío es **mejor esfuerzo**: si el servidor no responde, la cola se llena o la
configuración queda incompleta, los eventos se descartan en silencio sin bloquear ni
ralentizar la aplicación ni el registro local (`%LOCALAPPDATA%\NetworkBench\logs\`), que
sigue funcionando igual con o sin esto activado.

**El detalle sensible** (IP:puerto, huella de certificado, línea de comandos del motor
NTTTCP) solo se envía —a OpenObserve igual que al fichero local— con el nivel de registro
en **Debug** o **Trace**, nunca en el nivel de producción por defecto (constitución,
principio XIII).

---

## 3. Qué recibe OpenObserve exactamente

Todo llega como un **array JSON** (incluso un solo evento) al endpoint del paso 1.4. Tres
formas distintas de evento comparten dos campos: `instanceId` (UUID estable de esta
instalación) y `hostname` (el nombre de equipo Windows) — con ellos se puede filtrar o
separar los eventos de varias máquinas que escriban en el mismo stream.

### 3.1 Eventos del registro JSON propio (la mayoría del volumen)

```json
{
  "schemaVersion": 1,
  "timestamp": "28/09/2026 10:15:23.456 +02:00",
  "level": "info",
  "origin": "backend",
  "module": "control.server",
  "eventCode": "APP_EVENT",
  "message": "texto del evento",
  "errorCode": "NB-XXX-000",
  "durationMs": 123,
  "diagnosticId": "uuid",
  "safeParams": { "clave": "valor" },
  "_timestamp": "2026-09-28T08:15:23.456Z",
  "instanceId": "1a2b3c4d-...",
  "hostname": "PC-DANIEL"
}
```

`errorCode`, `durationMs`, `diagnosticId` y `safeParams` solo aparecen si tienen valor.
`timestamp` es la hora legible de Madrid (la misma que usan los ficheros locales);
`_timestamp` es la hora en RFC 3339 UTC — el campo que OpenObserve reconoce como hora real
del evento (sin él, OpenObserve usaría la hora de ingesta).

### 3.2 Líneas de diagnóstico avanzado (`tracing`)

Descubrimiento, emparejamiento, protocolo y motor NTTTCP — donde vive el detalle que exige
Debug/Trace:

```json
{
  "_timestamp": "2026-09-28T08:15:23.456Z",
  "level": "warn",
  "target": "networkbench_lib::discovery::mdns",
  "message": "texto del mensaje",
  "campos": { "otro_campo": "valor en texto" },
  "instanceId": "1a2b3c4d-...",
  "hostname": "PC-DANIEL"
}
```

### 3.3 Evento del botón "Probar conexión"

```json
{
  "_timestamp": "2026-09-28T08:15:23.456Z",
  "level": "info",
  "module": "settings.openobserve",
  "message": "Prueba de conexión desde NetworkBench",
  "instanceId": "1a2b3c4d-...",
  "hostname": "PC-DANIEL"
}
```

---

## 4. Ejemplos de consulta en OpenObserve

Orientativos: la sintaxis exacta depende de la versión de OpenObserve (interfaz SQL o
lenguaje de consulta propio). La idea es filtrar por `hostname` o `instanceId`:

```sql
SELECT * FROM netbench WHERE hostname = 'PC-DANIEL' ORDER BY _timestamp DESC
```

```sql
SELECT * FROM netbench WHERE level IN ('warn', 'error') ORDER BY _timestamp DESC
```

Si le da instrucciones a un agente de inteligencia artificial para que consulte estos
registros, dígale explícitamente que filtre por `hostname` (o `instanceId`, si necesita
distinguir dos máquinas con el mismo nombre de red) para no mezclar eventos de equipos
distintos.

---

## 5. Notas de seguridad

- El token se guarda en `settings.json` **en texto claro**, igual que el resto de ajustes
  de NetworkBench hoy (no hay cifrado DPAPI todavía). Protéjalo como protegería cualquier
  credencial con permiso de escritura en su OpenObserve.
- Nada de esto sustituye a los registros locales: son un envío adicional, no exclusivo.
- Apagar el interruptor detiene el envío al instante; no purga lo que ya se envió.
