# NetworkBench — Especificación detallada v1

**Estado:** Especificación de producto y técnica para desarrollo (derivada de `Historias.md` + decisiones del 20-09-2026)
**Plataforma:** Windows 10 (1809+) y Windows 11, x64
**Stack:** Tauri 2 + Rust · Svelte + TypeScript + Vite · WebView2
**Motor:** Microsoft NTTTCP (empaquetado)
**Nombre:** NetworkBench (provisional; identificador interno `networkbench`)
**Licencia:** GPL-3.0-or-later · **Repositorio:** `github.com/danimardo/networkbench` (público)

---

## 0. Convenciones de este documento

### 0.1. Nivel de obligación

- **DEBE / NO DEBE**: requisito obligatorio para dar por terminada la v1.
- **DEBERÍA**: recomendado; si no se cumple hay que justificarlo.
- **PUEDE**: opcional.

### 0.2. Hitos internos

La v1 pública se construye en tres hitos internos. Cada requisito lleva su hito entre corchetes.

| Hito | Contenido |
|---|---|
| **[H1]** | Canal de control, descubrimiento, emparejamiento, prueba TCP A→B y B→A con resultado estructurado, UI básica (inicio, descubrimiento, aceptación, ejecución, resultado simple). |
| **[H2]** | Firewall, motor de interpretación, estabilidad, gráficas en tiempo real, catálogo de errores completo, detalles técnicos, temas claro/oscuro. |
| **[H3]** | Historial, evolución, exportaciones (PDF/JSON/CSV), UDP y opciones avanzadas, pantalla de Ajustes completa, actualizador, bandeja del sistema. |

### 0.3. Valores provisionales

Los valores marcados **[PROV]** (umbrales, duraciones, streams) son puntos de partida fijados para que el sistema sea implementable. DEBEN residir en un único fichero de configuración interna (`thresholds.json`, embebido en el binario) y DEBEN validarse empíricamente antes de la publicación (ver §27).

### 0.4. Glosario

| Término | Definición |
|---|---|
| **Instancia** | Una ejecución de NetworkBench en un equipo. |
| **Peer** | La instancia remota con la que se comunica. |
| **Iniciador** | Instancia que solicita la prueba (PC A). |
| **Receptor de la solicitud** | Instancia que acepta o rechaza (PC B). No confundir con *receptor NTTTCP*. |
| **Emisor / receptor NTTTCP** | Rol de `ntttcp.exe` en una dirección concreta (`-s` / `-r`). En A→B, A es emisor y B receptor; en B→A se invierte. |
| **Sesión** | Una prueba completa (una o dos direcciones) identificada por un `sessionId` UUIDv4 compartido por ambos extremos. |
| **Dirección** | Un sentido de la prueba (A→B o B→A). |
| **Ejecución** | Una invocación concreta de `ntttcp.exe`. Una dirección tiene dos ejecuciones (una por extremo). |
| **Capacidad de referencia** | Velocidad contra la que se calcula el aprovechamiento (§13.1). |
| **Identidad** | Clave pública Ed25519 de una instancia (§6). |

---

## 1. Visión y principios

Se mantiene íntegra la visión de `Historias.md` §1–§3 y §53:

1. NetworkBench es «una aplicación que analiza una conexión de red y explica claramente si funciona como debería», no «una GUI para NTTTCP».
2. **Lenguaje humano primero.** Ningún error habitual se muestra solo como código técnico. Todo error responde a *qué ha ocurrido, cuál puede ser la causa, qué puedo hacer*.
3. **Modo sencillo por defecto + detalle progresivo.** No hay dos aplicaciones ni dos modos. "Opciones avanzadas" es un panel dentro del flujo, no un modo.
4. **Hecho ≠ interpretación ≠ posible causa.** El motor de interpretación nunca afirma una causa concreta cuando hay varias explicaciones posibles.
5. **Criterio de pantalla principal:** un dato solo entra en el nivel 1 o 2 si ayuda a una persona a entender si su conexión funciona correctamente.
6. **La precisión del benchmark tiene prioridad sobre los efectos visuales.**

---

## 2. Alcance

### 2.1. Dentro de la v1

Todo lo listado en §26 (criterios de aceptación).

### 2.2. Fuera de la v1 (explícitamente)

- Cloud, cuentas, SaaS, telemetría, gestión centralizada de agentes, monitorización continua.
- Linux, macOS, ARM64, Windows Server como plataforma oficialmente probada (DEBERÍA funcionar en Server 2019+ con Desktop Experience, pero no está en la matriz de pruebas).
- Speedtest contra Internet, SNMP, topología, traceroute, latencia/jitter por paquete, Path MTU.
- Ejecución como servicio de Windows / sin sesión iniciada.
- Interoperar con un `ntttcp.exe` lanzado a mano en un equipo sin NetworkBench ("modo receptor externo") → ampliación futura.
- Motores distintos de NTTTCP (iPerf3 → ampliación futura; la abstracción §11.1 lo prepara).
- Argumentos NTTTCP libres.
- Aplicación portable.

---

## 3. Plataforma, distribución y actualización

### 3.1. Requisitos de sistema [H1]

- Windows 10 versión 1809 o superior, Windows 11. Solo x64.
- WebView2 Evergreen Runtime. El instalador DEBE comprobarlo e instalarlo (bootstrapper) si falta.
- No requiere .NET.
- Permisos de administrador: **no** para ejecutar; solo para instalar y para crear/eliminar reglas de firewall (§14).

### 3.2. Empaquetado de NTTTCP [H1]

- `ntttcp.exe` (licencia MIT, Microsoft) se incluye en el instalador, en `<InstallDir>\engine\ntttcp.exe`.
- La versión concreta queda fijada en el build (`engine/VERSION`) y su hash SHA-256 embebido en el binario. Al arrancar y antes de cada prueba se verifica el hash; si no coincide → `NB-ENGINE-002` (binario alterado o ausente).
- El texto de la licencia MIT de NTTTCP DEBE aparecer en "Acerca de".
- Los errores «NTTTCP ausente / no ejecutable» de `Historias.md` §29 quedan como casos de corrupción, borrado por antivirus o bloqueo por política (AppLocker/SmartScreen), no como situación normal.

### 3.3. Instalador [H1]

- Instalador NSIS de Tauri, por máquina (`installMode: perMachine`), requiere elevación. Instala en `%ProgramFiles%\NetworkBench\`. Instalador en español e inglés con selector de idioma. WebView2 en modo `offlineInstaller` (no depende de Internet durante la instalación).
- Crea acceso directo en el menú Inicio. No arranca con Windows por defecto.
- **Nombre de fichero fijo:** `networkbench-setup.exe`, sin versión ni arquitectura en el nombre (§3.5). La versión va dentro (recursos del ejecutable y pantalla del instalador).
- **Sin firma Authenticode en v1.** Windows SmartScreen mostrará «Windows protegió tu PC» la primera vez; este aviso DEBE documentarse en el README, en el cuerpo de cada Release y en la página web («Más información → Ejecutar de todas formas»). Las reglas de firewall por programa (§14) no dependen de la firma, sino de la ruta de instalación. Si en el futuro se dispone de certificado, el workflow firmará cuando exista el secreto correspondiente, sin cambiar nada más.
- El desinstalador DEBE: eliminar todas las reglas de firewall con prefijo `NetworkBench - ` (§14.6), eliminar la entrada de autoarranque si existe, y **preguntar** si se borran los datos de usuario (`%APPDATA%\NetworkBench`, `%LOCALAPPDATA%\NetworkBench`). Por defecto se conservan.

### 3.4. Actualizador [H3]

- Tauri Updater con firma **minisign** (clave privada solo en los secretos de GitHub; clave pública embebida en `tauri.conf.json`). Endpoint único: `https://github.com/danimardo/networkbench/releases/latest/download/latest.json`.
- Comprueba al arrancar (una vez cada 24 h como máximo) y desde Ajustes → "Comprobar ahora".
- Nunca se actualiza durante una sesión de prueba activa.
- Ajuste "Actualizaciones automáticas" (por defecto activado: descarga y pide confirmación para instalar; nunca instala sin preguntar).
- Si el peer tiene una versión con protocolo incompatible se muestra `NB-VERSION-001` con indicación de cuál de los dos debe actualizar (§8.4).

### 3.5. Licencia, repositorio y pipeline de publicación [H1]

**Licencia**

- Código propio bajo **GPL-3.0-or-later**. Fichero `LICENSE` con el texto íntegro de la GPLv3; cabecera SPDX (`SPDX-License-Identifier: GPL-3.0-or-later`) en cada fichero fuente. Titular del copyright: Daniel Diez Mardomingo.
- `THIRD_PARTY_NOTICES.md` con licencia y atribución de todo lo redistribuido: NTTTCP (MIT, Microsoft), tipografías (OFL si procede), y las dependencias Rust/npm generadas por herramienta (`cargo about` / `license-checker`). El instalador copia estos ficheros a `<InstallDir>\licenses\`. «Acerca de» muestra la licencia de la app y enlaza a las de terceros.
- No se incorporan dependencias con licencias incompatibles con GPLv3.

**Repositorio**

- `https://github.com/danimardo/networkbench`, **público desde el primer commit**. Rama principal `main`. Si el producto cambia de nombre se renombra el repositorio (GitHub mantiene la redirección; el updater y el enlace de descarga se actualizan en la misma Release).
- `README.md` con: qué es, captura, enlace de descarga fijo, requisitos, aviso de SmartScreen, cómo compilar, licencia.

**Versionado**

- Semver. La versión vive en `package.json`; `tauri.conf.json` la lee de ahí (`"version": "../package.json"`). `Cargo.toml` se mantiene sincronizado por script de verificación en CI.

**CI (en cada push y pull request)** — workflow `ci.yml`, `windows-latest`:

1. `pnpm install --frozen-lockfile`.
2. Verificaciones propias: diccionarios i18n sincronizados (es/en con las mismas claves), hash de `engine/ntttcp.exe` igual al declarado, versión sincronizada.
3. Prettier, ESLint, `svelte-check` (cero errores y cero avisos), `vitest`.
4. `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --locked`.
5. `vite build` y `cargo build` (sin empaquetar).

**Release (solo al crear una etiqueta `vX.Y.Z` a mano)** — workflow `release.yml`:

1. Mismos pasos de instalación que CI.
2. `tauri build` con `bundle.createUpdaterArtifacts: true` y los secretos `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (minisign). Si existiera un secreto de certificado Authenticode, se firma el `.exe`; si no, se publica sin firmar.
3. Script `scripts/rename-installer.mjs`: renombra el único `.exe` de `src-tauri/target/release/bundle/nsis/` a **`networkbench-setup.exe`** (falla si hay cero o más de uno) y renombra igual su `.sig`.
4. Script `scripts/build-update-manifest.mjs`: genera `latest.json` (formato «Static JSON File» del plugin updater: `version`, `pub_date`, `platforms["windows-x86_64"] = { url, signature }`, donde `signature` es el **contenido** del `.sig` y `url` es el enlace fijo de abajo).
5. `softprops/action-gh-release`: crea la Release de la etiqueta con notas generadas y sube `networkbench-setup.exe` y `latest.json`. Cuerpo fijo con el aviso de SmartScreen y los requisitos.

**Enlace estable** (para la web y la documentación; nunca cambia entre versiones):

```
https://github.com/danimardo/networkbench/releases/latest/download/networkbench-setup.exe
```

Nunca se publica desde `main` automáticamente ni desde una pull request: publicar es una decisión explícita (crear la etiqueta). Las etiquetas de prueba (`v0.x.y`) publican Releases marcadas como *pre-release* si la versión es `< 1.0.0`.

---

## 4. Idioma e internacionalización [H1]

- Todos los textos de UI, catálogo de errores, textos de interpretación, informes PDF y nombres de columnas CSV DEBEN estar externalizados en ficheros de recursos (`locales/es.json`, `locales/en.json`), incluidos los plurales y los formatos numéricos.
- Idiomas entregados en v1: **español (es)** e **inglés (en)**.
- Selección automática por el idioma de interfaz de Windows (`GetUserPreferredUILanguages`):

| Locale de Windows | Idioma de la app |
|---|---|
| `es-*` (cualquier variante, incluida Latinoamérica) | Español |
| `ca-*`, `eu-*`, `gl-*`, `ast-*`, `an-*` (lenguas de la península ibérica distintas del portugués) | Español |
| `pt-*` y cualquier otro | Inglés |

- Ajustes → Idioma: "Automático / Español / English". El cambio se aplica sin reiniciar.
- Formato numérico y de fecha según el idioma de la app (no según la región de Windows), para que el separador decimal sea coherente con los textos: español → `9,42`, `20 sep 2026 12:20`; inglés → `9.42`, `20 Sep 2026 12:20`.
- El idioma se envía al peer en el handshake solo a efectos informativos; cada extremo muestra la sesión en su propio idioma. Los textos generados (nombres de equipo, alias) se transmiten tal cual.

---

## 5. Modelo de ejecución

### 5.1. Instancia única [H1]

- Solo una instancia de NetworkBench por sesión de usuario de Windows (mutex nombrado). Lanzar una segunda vez trae la ventana existente al frente.
- Si dos usuarios de Windows tuvieran sesión simultánea (cambio rápido de usuario), la segunda instancia no podrá abrir el puerto de control y mostrará `NB-PORT-001`.

### 5.2. Ventana, bandeja y arranque [H3]

**Cierre de la ventana**

- Toda vía de cierre de la ventana (botón cerrar de la barra propia §16.13, Alt+F4, «Cerrar ventana» de la barra de tareas) DEBE pasar por el mismo punto (`WindowEvent::CloseRequested` interceptado con `prevent_close`). Nunca hay dos comportamientos distintos según cómo se cierre.
- Ajuste `closeAction` ∈ { `ask` (defecto), `minimize`, `exit` }.
- Con `ask`, se muestra un diálogo modal propio:

```
¿Qué quieres hacer?

NetworkBench puede seguir disponible en segundo plano para que otros equipos
puedan hacer pruebas con este.

[ ] Recordar mi decisión

        [ Minimizar a la bandeja ]   [ Cerrar NetworkBench ]
```

  - «Minimizar a la bandeja» oculta la ventana y la app sigue escuchando. «Cerrar NetworkBench» sale del todo. Escape o clic fuera = no hacer nada.
  - Si la casilla está marcada, la elección se guarda en `closeAction` y no se vuelve a preguntar. Se puede cambiar en Ajustes → General → «Al cerrar la ventana: Preguntar / Minimizar a la bandeja / Cerrar NetworkBench».
- Con `minimize` o `exit` se actúa directamente sin diálogo.
- **Con una sesión activa** (`CONNECTING`…`ANALYZING`): minimizar no interrumpe nada (la prueba sigue y el icono de bandeja muestra «Prueba en curso»). Cerrar —por diálogo, por ajuste `exit` o desde «Salir» de la bandeja— muestra siempre una confirmación: «**Se cancelará la prueba con SERVER-01.** ¿Quieres cerrar NetworkBench de todas formas?» [Volver] [Cancelar prueba y cerrar]. Al confirmar se ejecuta la cancelación de §10.8 antes de salir.
- Antes de ocultar o salir se guarda la geometría de la ventana (§16.9).

**Bandeja**

- Icono de bandeja con menú: «Abrir NetworkBench», estado («Disponible» / «Prueba en curso con X» / «Sin red»), «Salir». Clic simple en el icono restaura la ventana.
- «Salir» desde la bandeja no pasa por `CloseRequested`; se gestiona en `RunEvent::ExitRequested`, donde también se guarda la geometría.

**Arranque**

- Ajuste «Arrancar con Windows» (por defecto desactivado). Se implementa con la clave `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (no requiere elevación); arranca oculto en la bandeja, sin mostrar la ventana.
- Hasta [H3] la app solo escucha con la ventana abierta y cerrar = salir (sin diálogo).

### 5.3. Estado "escuchando"

- Al arrancar, la instancia:
  1. Carga o genera su identidad (§6).
  2. Abre el puerto de control TLS (§8.1). Si falla → `NB-PORT-001` con opción de cambiar puerto.
  3. Publica el servicio mDNS (§7.1) si el ajuste está activo.
- El estado de escucha es visible en la pantalla inicial: "Este equipo es visible como **DESKTOP-DANIEL** en 192.168.10.15".

### 5.4. Roles dinámicos [H1]

Cualquier instancia puede iniciar o recibir. No hay configuración de rol.

### 5.5. Concurrencia [H1]

- Máximo **una sesión activa por instancia** (desde `CONNECTING` hasta `COMPLETED`/`FAILED`/`CANCELLED`).
- Una solicitud entrante mientras hay sesión activa se rechaza automáticamente con motivo `busy`. El iniciador la muestra como `NB-PEER-003` ("El equipo está realizando otra prueba") con acción "Reintentar".
- Un usuario no puede iniciar una sesión mientras tiene un diálogo de solicitud entrante pendiente; primero debe responderlo.

### 5.6. Resultados en ambos extremos [H1]

Al terminar, ambos extremos disponen de la sesión completa (ambas ejecuciones de cada dirección, muestras de ambos adaptadores, información de ambas interfaces), guardan la misma sesión con el mismo `sessionId` en su historial y muestran la misma pantalla de resultados. Solo cambia la perspectiva ("este equipo" / "el otro equipo").

---

## 6. Identidad de los equipos [H1]

### 6.1. Identidad criptográfica

- En el primer arranque se genera:
  - `instanceId`: UUIDv4.
  - Par de claves **Ed25519**.
  - Certificado X.509 autofirmado (validez 20 años) cuya clave es la Ed25519 anterior; `CN = <instanceId>`.
- La **identidad** de una instancia es la huella SHA-256 de su clave pública (`fingerprint`). `instanceId`, hostname e IP son etiquetas.
- Clave privada almacenada en `%APPDATA%\NetworkBench\identity\` cifrada con DPAPI (ámbito usuario). Nunca se registra en logs ni se exporta.
- Reinstalar sin borrar datos conserva la identidad. Borrar datos genera una nueva; los peers verán "la identidad de este equipo ha cambiado" (§9.4).

### 6.2. Nombre visible

- `displayName`: por defecto el hostname de Windows. Editable en Ajustes → General (1–48 caracteres, sin caracteres de control). Se envía al peer en el handshake y en mDNS.
- Alias local: cada usuario puede asignar un alias a un peer en su lista de favoritos (§7.3). En la UI se muestra `alias (displayName)` si hay alias; si no, `displayName`.
- Todo nombre recibido del peer se sanea antes de mostrarse (§24.2) y se trunca a 48 caracteres.

---

## 7. Descubrimiento y conexión manual

### 7.1. Descubrimiento en LAN por mDNS/DNS-SD [H1]

- Tipo de servicio: `_networkbench._tcp.local.`
- Nombre de instancia: `<displayName> [<instanceId corto 8 hex>]`.
- Puerto: el puerto de control.
- Registros TXT:

| Clave | Valor |
|---|---|
| `v` | Versión de protocolo (entero) |
| `app` | Versión de NetworkBench (semver) |
| `id` | `instanceId` |
| `fp` | Huella de la clave pública (hex, 64) |
| `name` | `displayName` |
| `link` | Velocidad negociada del adaptador principal en Mbit/s, o `0` |
| `busy` | `0`/`1` |

- La app publica **y** navega. Lista "Equipos disponibles" refrescada por eventos mDNS; una entrada desaparece 30 s después del último anuncio.
- Cada entrada muestra: nombre, IP (IPv4 preferida), velocidad de enlace (si `link > 0`), estado ("Disponible", "Ocupado", "Versión incompatible", "De confianza ★").
- mDNS no cruza subredes: la spec lo declara y la pantalla de descubrimiento lo explica en un texto secundario ("¿No aparece? Si está en otra red, usa *Conectar manualmente*").
- Ajuste "Descubrimiento automático" on/off (por defecto on). Al desactivarlo la instancia ni publica ni navega.
- Si el adaptador activo está en perfil de firewall "público", Windows puede bloquear mDNS: si tras 5 s no se ve ningún equipo y el perfil es público, se muestra un aviso con enlace a §14.

### 7.2. Conexión manual [H1]

- Campo "Dirección del equipo" que admite: nombre DNS, IPv4, IPv6 (con o sin corchetes), y opcionalmente `:puerto` (`host:7411`, `[fe80::1]:7411`). Sin puerto se usa el puerto por defecto.
- Resolución DNS: se obtienen todas las direcciones; se intenta IPv4 primero (todas las A, en orden) y después IPv6 (AAAA). Timeout de conexión TCP: 5 s por dirección, máximo 20 s en total.
- El resultado indica qué dirección se usó. Opciones avanzadas permite forzar IPv4 o IPv6.
- Errores: `NB-CONN-001` (no se pudo contactar), `NB-CONN-002` (DNS no resoluble), `NB-CONN-004` (responde pero no es NetworkBench).

### 7.3. Favoritos y recientes [H1]

- **Recientes**: últimos 10 peers con los que se completó una sesión, ordenados por fecha.
- **Favoritos**: marcados por el usuario (★). Sin límite.
- Un peer se identifica por su huella. Se guardan: huella, `instanceId`, último `displayName`, alias local, última dirección usada (host:puerto tal como lo escribió el usuario, o IP descubierta), última conexión, nivel de confianza (§9).
- Al seleccionar un favorito se intenta primero la última dirección; si falla y el peer aparece por mDNS con otra IP, se usa esa.
- Sección de la pantalla inicial: "Otros equipos" (favoritos y recientes) bajo "Equipos disponibles".

---

## 8. Canal de control

### 8.1. Transporte [H1]

- TCP + **TLS 1.3**, certificado autofirmado de cada instancia (§6.1). Verificación **por huella**, no por CA. Ambos lados presentan certificado (mTLS).
- Puerto por defecto **TCP 7411** [PROV]. Configurable en Ajustes → Red (1024–65535). El nuevo puerto se anuncia por mDNS; la conexión manual admite `host:puerto`.
- Si el puerto está ocupado al arrancar → `NB-PORT-001`, con acción "Elegir otro puerto" (propone el primer libre entre 7411–7420).
- Una conexión de control por sesión, abierta por el iniciador. Fuera de sesión no hay conexiones persistentes.
- Todo mensaje que no sea del protocolo, exceda el tamaño máximo o llegue fuera de estado válido cierra la conexión y se registra.

### 8.2. Formato de mensajes [H1]

- Trama: `uint32 big-endian longitud` + cuerpo JSON UTF-8. Longitud máxima: 1 MiB (los resultados con XML crudo caben; un mensaje mayor cierra la conexión).
- Cuerpo:

```json
{
  "type": "REQUEST",
  "id": "uuid-del-mensaje",
  "sessionId": "uuid-de-sesion | null",
  "ts": "2026-09-20T12:20:00.000Z",
  "payload": { }
}
```

- Cada extremo ignora campos desconocidos del `payload` (tolerancia hacia delante).
- Cada mensaje que requiere respuesta la recibe con `inReplyTo = id`. Timeout de respuesta genérico: 10 s salvo indicación.

### 8.3. Catálogo de mensajes [H1]

| Tipo | Emisor → Receptor | Payload principal | Respuesta |
|---|---|---|---|
| `HELLO` | Ambos, tras TLS | `protocolVersion`, `protocolMin`, `appVersion`, `instanceId`, `displayName`, `os`, `lang`, `busy` | `HELLO` del otro lado |
| `PAIR_REQUEST` | Iniciador | — (las huellas ya se conocen por TLS) | `PAIR_RESULT {accepted, reason}` |
| `REQUEST` | Iniciador | `BenchmarkPlan` (§11.3), `estimatedSeconds`, `localInterface` (§15) | `RESPONSE {accepted, reason: "user_rejected"\|"busy"\|"timeout"\|"params_invalid"\|"auto_accepted", remoteInterface}` |
| `PREPARE` | Iniciador | `direction`, `ports {base, count}`, `engineParams` | `PREPARE_RESULT {ok, checks[], suggestedBasePort?}` |
| `READY` | El extremo que ejecuta el receptor NTTTCP | `direction`, `pid` | — |
| `START` | Iniciador | `direction`, `startAt` (timestamp), `warmupSeconds`, `measureSeconds` | `STARTED {direction}` |
| `SAMPLE` | Ambos, cada 500 ms | `direction`, `t` (ms desde START), `rxBps`, `txBps`, `cpuPercent` | — |
| `ENGINE_DONE` | Cada extremo al terminar su ntttcp | `direction`, `role`, `EngineResult` (§11.6) incl. XML crudo | — |
| `ENGINE_FAILED` | Cada extremo | `direction`, `role`, `errorCode`, `stderrTail` | — |
| `SESSION_RESULT` | Iniciador, tras `ANALYZING` | `SessionResult` completo (§13.7) | `SESSION_ACK` |
| `CANCEL` | Cualquiera | `reason: "user"\|"error"`, `errorCode?` | `CANCEL_ACK` |
| `HEARTBEAT` | Ambos, cada 1 s durante la sesión | — | `HEARTBEAT` |
| `ERROR` | Cualquiera | `code`, `detail` | — |
| `BYE` | Cualquiera | — | — |

### 8.4. Versionado del protocolo [H1]

- `protocolVersion` entero, empieza en `1`. `protocolMin` = mínima versión que la instancia sabe hablar.
- Tras `HELLO`, se usa `min(vA, vB)` si es ≥ `max(minA, minB)`; si no → `NB-VERSION-001` en ambos, indicando qué extremo tiene la versión antigua ("Actualiza NetworkBench en SERVER-01 (v1.2) para probar con este equipo (v1.4)").
- Cambios compatibles (campos nuevos opcionales) no incrementan la versión; cambios de semántica sí.

### 8.5. Latido y pérdida del canal [H1]

- `HEARTBEAT` cada 1 s durante toda la sesión. Sin respuesta durante 5 s → estado visual "Reconectando…" en ambos extremos (la prueba NTTTCP en curso no se detiene).
- El iniciador reintenta abrir una nueva conexión TLS con el mismo `sessionId` cada 2 s hasta **15 s** desde la primera pérdida. El receptor acepta una reconexión solo si la huella coincide y la sesión sigue activa.
- Si se recupera y `ntttcp.exe` sigue vivo en ambos extremos, la dirección continúa; las muestras perdidas se marcan como huecos (no se interpolan).
- Si no se recupera en 15 s, o se recupera pero algún ntttcp ha terminado por error: se aborta con `NB-CONN-005`. Las direcciones completadas se conservan y la sesión se guarda como **incompleta** (§19.3). Nunca se reanuda una dirección a medias.

### 8.6. Límites y protección [H1]

| Límite | Valor [PROV] |
|---|---|
| Solicitudes pendientes por peer | 1 |
| Tiempo de bloqueo tras rechazo del usuario a ese peer | 30 s (rechazo automático `rejected_recently`) |
| Solicitudes entrantes totales por minuto | 5 |
| Conexiones TLS fallidas desde una misma IP | 10 → esa IP bloqueada 60 s |
| Conexiones TCP entrantes simultáneas | 8 |
| Timeout de handshake TLS+HELLO | 5 s |
| Tamaño máximo de mensaje | 1 MiB |

Todos los eventos de límite se registran en el log con la IP y la huella (si se conoció).

---

## 9. Emparejamiento y confianza [H1]

### 9.1. Primer contacto: código de verificación

1. Tras `HELLO`, si el iniciador no tiene al peer como conocido (o su huella ha cambiado), envía `PAIR_REQUEST`.
2. Ambos extremos calculan `code = SHA-256(min(fpA,fpB) || max(fpA,fpB) || sessionId) mod 10^6`, presentado como `123 456`.
3. Ambos muestran el código. El receptor ve: «**DESKTOP-DANIEL quiere emparejarse con este equipo.** Comprueba que en el otro equipo aparece el mismo código: **123 456**» con botones **Coincide** / **No coincide**. El iniciador ve «Esperando confirmación en SERVER-01… Código: **123 456**».
4. "Coincide" → ambos guardan al otro como **conocido** (huella + nombre + fecha). "No coincide" o timeout (60 s) → `PAIR_RESULT {accepted:false}` y se corta.
5. En la práctica el diálogo de emparejamiento y el de aceptación de prueba (§10.4) se muestran **combinados** en el receptor la primera vez: un solo diálogo con el código y los datos de la prueba, y un solo botón "Coincide y aceptar". Rechazar cualquiera de las dos cosas rechaza ambas.

### 9.2. Niveles de confianza

| Nivel | Significado | Efecto |
|---|---|---|
| **Desconocido** | Nunca emparejado | Requiere código de verificación |
| **Conocido** | Emparejado | No vuelve a pedir código. Cada prueba pide **Aceptar**. |
| **De confianza** | El usuario marcó "Confiar siempre en este equipo" | Igual que conocido, pero aparece con ★ y habilita el conmutador de aceptación automática |
| **De confianza + aceptación automática** | Conmutador activado en Ajustes (por defecto **desactivado**) | Las solicitudes de ese peer se aceptan sin preguntar; se muestra un toast informativo y la ventana pasa a la vista de ejecución con botón Cancelar |

- "Confiar siempre en este equipo" es una casilla en el diálogo de aceptación, desmarcada por defecto.
- Al activar la aceptación automática de un peer se muestra un aviso: «SERVER-01 podrá iniciar pruebas de rendimiento en este equipo sin preguntarte. Durante cada prueba se generará tráfico intenso.» con **Activar** / **Cancelar**.
- La aceptación automática se aplica solo al plan estándar y a planes avanzados dentro de los límites de §17; cualquier valor fuera de límites se rechaza (`params_invalid`) aunque el peer sea de confianza.

### 9.3. Gestión

Ajustes → Equipos de confianza: tabla con nombre, alias, huella abreviada (8 primeros hex), fecha de emparejamiento, nivel, conmutador de aceptación automática, botón "Eliminar" (vuelve a Desconocido). Misma gestión accesible desde la ficha de un favorito.

### 9.4. Cambio de identidad

Si un peer conocido se presenta con la misma `instanceId`/nombre pero otra huella: se trata como desconocido y el diálogo de emparejamiento lo indica: «**La identidad de SERVER-01 ha cambiado.** Esto ocurre si se ha reinstalado NetworkBench o si otro equipo está usando su nombre. Comprueba el código antes de continuar.» La antigua entrada se sustituye solo tras confirmar.

---

## 10. Sesión de prueba

### 10.1. Máquina de estados [H1]

Ambos extremos mantienen la misma máquina y el mismo `sessionId`. Todo cambio de estado se propaga por el canal de control antes de reflejarse en la UI del otro lado.

```
IDLE
 └─ DISCOVERING            (solo iniciador, mientras la pantalla de descubrimiento está abierta)
     └─ CONNECTING         TCP + TLS + HELLO
         └─ PAIRING        solo si el peer es desconocido (§9.1)
             └─ PRECHECK   comprobaciones locales del iniciador (§10.2)
                 └─ WAITING_FOR_ACCEPTANCE   REQUEST enviado; el receptor muestra el diálogo
                     └─ PREPARING            comprobaciones conjuntas (§10.5)
                         └─ RUNNING_FORWARD  A→B
                             └─ SWITCHING_DIRECTION   1 s
                                 └─ RUNNING_REVERSE   B→A
                                     └─ ANALYZING     cálculo de métricas e interpretación
                                         └─ COMPLETED
Estados terminales alternativos: FAILED (con errorCode), CANCELLED (con origen), DISCONNECTED (canal perdido sin recuperación → se resuelve en FAILED NB-CONN-005)
```

- En planes de una sola dirección se omite `SWITCHING_DIRECTION` y la dirección no solicitada. En plan simultáneo (§17) hay un único estado `RUNNING_BOTH`.
- Toda transición se registra en el log con timestamp y `sessionId`.

### 10.2. PRECHECK local (iniciador) [H1]

Se ejecuta antes de molestar al peer. Todas son locales:

| Comprobación | Error si falla |
|---|---|
| Hash de `ntttcp.exe` correcto y ejecutable (lanzar `ntttcp.exe -?` y comprobar salida) | `NB-ENGINE-001/002` |
| Adaptador seleccionado (o automático) presente y conectado | `NB-NIC-001` |
| IP local elegida en la misma familia que la del peer | `NB-NIC-003` |
| Versión de protocolo compatible (ya negociada en HELLO) | `NB-VERSION-001` |
| Peer no ocupado (`busy` en HELLO) | `NB-PEER-003` |
| Espacio en disco ≥ 200 MB en `%APPDATA%` | `NB-DISK-001` |

Selección automática del adaptador local: el que Windows usa para la ruta hacia la IP del peer (`GetBestInterfaceEx`). Se muestra en la UI antes de enviar la solicitud.

### 10.3. Solicitud [H1]

`REQUEST` incluye el `BenchmarkPlan` (§11.3), la duración estimada total en segundos y la información de la interfaz local (§15). El iniciador muestra «Esperando a que SERVER-01 acepte…» con botón **Cancelar**. Timeout: 60 s → `RESPONSE {accepted:false, reason:"timeout"}` → `NB-PEER-002` ("Nadie ha respondido en el otro equipo").

### 10.4. Diálogo de aceptación (receptor) [H1]

```
Solicitud de prueba de red

DESKTOP-DANIEL quiere realizar una prueba de rendimiento con este equipo.

Dirección:               192.168.10.15
Tipo:                    TCP, en ambas direcciones          ← o "UDP, solo envío desde este equipo", etc.
Duración aproximada:     1 minuto                           ← estimatedSeconds redondeado: < 90 s → "1 minuto"; si no, "N minutos"
Adaptador que se usará:  Intel X550-T2 · 10 Gbit/s

Durante la prueba se generará tráfico intenso de red.

[ ] Confiar siempre en este equipo

              [ Rechazar ]   [ Aceptar ]
```

- Si el peer es desconocido, se incorpora el bloque de código de verificación (§9.1) y el botón pasa a "Coincide y aceptar".
- Si la app está minimizada/bandeja: toast de Windows con título «Solicitud de prueba de red», cuerpo «DESKTOP-DANIEL quiere realizar una prueba de rendimiento con este equipo», botones **Aceptar** / **Rechazar** (solo si el peer es conocido; si es desconocido el toast tiene solo "Ver") y al pulsar se restaura la ventana con el diálogo. **[H3]**
- Timeout 60 s sin respuesta: el diálogo se cierra, se responde `timeout`.
- Ambos botones responden inmediatamente; el iniciador muestra `NB-PEER-001` ("SERVER-01 ha rechazado la prueba") si procede.

### 10.5. PREPARING: comprobaciones conjuntas [H1] (firewall [H2])

Pantalla idéntica en ambos extremos con lista de pasos y estado (✓ / ○ / ✗):

| Paso | Cómo | Error |
|---|---|---|
| Otro equipo encontrado | ya hecho | — |
| NetworkBench disponible | ya hecho | — |
| Comunicación establecida | ya hecho | — |
| Adaptador de red identificado (ambos) | intercambio de §15 | `NB-NIC-001` |
| Puertos disponibles | el receptor NTTTCP de la dirección comprueba que `base..base+streams-1` están libres (bind de prueba TCP y UDP). Si no, `PREPARE_RESULT` propone `suggestedBasePort` (primer rango libre entre 5001 y 5999, de 64 en 64) y el iniciador reintenta con él una vez. | `NB-PORT-002` |
| Firewall preparado | el emisor NTTTCP abre una conexión TCP de sondeo al puerto `base` del receptor NTTTCP (que escucha 2 s con un socket de prueba). Si falla mientras el canal de control sí funciona → diagnóstico de firewall (§14.4). | `NB-FW-001/002` |
| Motor NTTTCP disponible (ambos) | hash + ejecución de prueba | `NB-ENGINE-001/002` |
| Preparando receptor | se lanza `ntttcp.exe -r`, se espera `READY` (§11.5) | `NB-ENGINE-003` |
| Iniciando benchmark | `START` | — |

El firewall se comprueba para **cada dirección** (los roles se invierten), pero se hace en PREPARING para ambas antes de empezar, de modo que un fallo en B→A no se descubra tras 25 s de prueba.

### 10.6. RUNNING [H1]

Ver §11.5 para la secuencia con NTTTCP, §12 para las muestras y §16.4 para la pantalla.

### 10.7. ANALYZING y COMPLETED [H1]

- El iniciador recopila los cuatro `EngineResult` (dos por dirección), las series de muestras de ambos extremos y la información de interfaces, calcula el `SessionResult` (§13.7) y lo envía en `SESSION_RESULT`. El receptor lo guarda tal cual (no recalcula) y responde `SESSION_ACK`. Así ambos historiales son idénticos.
- Si el `SESSION_RESULT` no llega en 10 s el receptor calcula localmente con los datos que tenga y marca `resultSource: "local"`.

### 10.8. Cancelación [H1]

Botón **Cancelar prueba** en ambos extremos en todos los estados desde `CONNECTING` hasta `ANALYZING`.

Secuencia en el extremo que cancela:
1. Enviar `CANCEL {reason:"user"}` (no se espera el ACK más de 2 s).
2. Terminar `ntttcp.exe` local (§11.5.5).
3. Liberar puertos, borrar temporales.
4. Estado `CANCELLED {origin:"local"}`.
5. Si alguna dirección terminó completa, ofrecer «Guardar los resultados parciales» (por defecto sí, §19.3).

El otro extremo, al recibir `CANCEL`: pasos 2–5 con `origin:"remote"` y muestra «**La prueba ha sido cancelada desde SERVER-01.**».

Cancelación implícita: cerrar la app, apagar, o `BYE` del peer → mismo procedimiento con `reason:"error"`.

---

## 11. Motor de benchmark

### 11.1. Abstracción [H1]

```
trait BenchmarkEngine {
    fn id() -> &str                                  // "ntttcp"
    fn version(&self) -> EngineVersion
    fn validate(&self, plan: &BenchmarkPlan) -> Result<(), ParamError>
    fn prepare_receiver(&self, run: &RunSpec) -> Result<EngineProcess>   // lanza -r, resuelve cuando escucha
    fn start_sender(&self, run: &RunSpec) -> Result<EngineProcess>
    fn stop(&self, proc: &EngineProcess, grace: Duration)
    fn parse(&self, proc_output: &ProcessOutput) -> Result<EngineResult, ParseError>
}
```

- Solo `NtttcpEngine` en v1. Ningún otro módulo (UI, interpretación, historial, exportación) DEBE conocer argumentos ni XML de NTTTCP; consumen `EngineResult`.
- `rawOutput` (XML + stdout + stderr + línea de comandos) se conserva en `EngineResult.raw` para detalles técnicos, soporte y exportación JSON.

### 11.2. Tipos

```
BenchmarkPlan {
  protocol: "tcp" | "udp"
  directions: "forward" | "reverse" | "both_sequential" | "both_simultaneous"
  measureSeconds: u16          // 5..300
  warmupSeconds: u8            // 0..10
  cooldownSeconds: u8          // 0..10
  streams: u8                  // 1..64
  bufferBytes: u32             // 4096..4194304
  basePort: u16                // 1024..65000
  ipFamily: "auto" | "v4" | "v6"
  udp?: { targetMbps: u32, datagramBytes: u16 }
  expectedCapacityMbps?: u32   // capacidad manual (§13.1)
  localInterfaceId?: string    // GUID del adaptador; null = automático
}
RunSpec { plan, direction, role: "sender"|"receiver", localIp, remoteIp, port, xmlPath }
EngineResult {
  direction, role, throughputBps, bytes, durationSeconds,
  packetsSent?, packetsReceived?, packetsRetransmitted?, errors?,
  cpuPercent?, cyclesPerByte?, interruptsPerSec?, dpcsPerSec?,
  streams, bufferBytes, raw: { cmdline, xml, stdout, stderr, exitCode, engineVersion }
}
```

### 11.3. Plan estándar [H1] [PROV]

| Parámetro | Valor |
|---|---|
| `protocol` | tcp |
| `directions` | both_sequential (A→B primero, luego B→A; pausa 1 s) |
| `warmupSeconds` | 2 |
| `measureSeconds` | 20 |
| `cooldownSeconds` | 2 |
| `bufferBytes` | 65536 |
| `streams` | según la **capacidad de referencia** (§13.1): ≤ 1 000 Mbit/s → 4; 1 001–5 000 → 8; > 5 000 → 16; desconocida → 8 |
| `basePort` | 5001 |
| `ipFamily` | auto |
| Afinidad de CPU | ninguna (`*`) |
| Duración total estimada | 2·(2+20+2) + 1 + arranques ≈ 55 s → «1 minuto» |

### 11.4. Mapeo a argumentos NTTTCP [H1]

Todos los argumentos se construyen en Rust a partir de `RunSpec`; nunca desde texto libre ni desde datos del peer sin validar.

| Campo | Emisor | Receptor |
|---|---|---|
| Rol | `-s` | `-r` |
| Mapeo | `-m <streams>,*,<remoteIp>` | `-m <streams>,*,<localIp>` |
| Duración | `-t <measureSeconds>` | `-t <measureSeconds>` |
| Calentamiento / enfriamiento | `-wu <n> -cd <n>` | `-wu <n> -cd <n>` |
| Buffer | `-l <bufferBytes>` | `-l <bufferBytes>` |
| Puerto base | `-p <basePort>` | `-p <basePort>` |
| UDP | `-u` (+ tamaño de datagrama vía `-l`) | `-u` |
| IPv6 | `-6` | `-6` |
| Salida | `-xml <xmlPath>` | `-xml <xmlPath>` |
| Sin ventana de consola | `CREATE_NO_WINDOW` | igual |

NTTTCP asigna un puerto por stream: `basePort + i` (i = 0..streams-1) [a verificar en la versión empaquetada, V-01]. La regla de firewall cubre `basePort .. basePort+63`.

**Nota UDP:** si la versión empaquetada de NTTTCP no ofrece limitación de tasa, la tasa objetivo (§18) se aproximará con el número de streams y el tamaño de datagrama y el resultado se etiquetará «tasa aproximada» [V-02].

### 11.5. Ciclo de vida del proceso [H1]

1. **Lanzamiento**: `ntttcp.exe` se ejecuta oculto (`CREATE_NO_WINDOW`), dentro de un **Job Object** con `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. Si NetworkBench muere, Windows mata a NTTTCP. Directorio de trabajo y XML en `%TEMP%\NetworkBench\<sessionId>\<direction>-<role>.xml`.
2. **Receptor primero**: el extremo receptor de la dirección lanza `-r`, espera a que los puertos estén en escucha (sondeo de `bind`/`GetExtendedTcpTable` hasta 3 s) y envía `READY`. Si no llega a escuchar → `NB-ENGINE-003`.
3. **Emisor**: al recibir `READY`, el iniciador envía `START {startAt = ahora + 500 ms}`; el extremo emisor lanza `-s` en `startAt`. El instante `startAt` es el t=0 de las muestras.
4. **Fin**: se espera la salida de ambos procesos. Timeout duro: `warmup + measure + cooldown + 15 s`; superado → se mata, `NB-ENGINE-004`. Código de salida ≠ 0 o XML ausente/ilegible → `NB-ENGINE-005` con `stderr` en detalles.
5. **Parada**: `stop()` intenta cierre limpio (`GenerateConsoleCtrlEvent` no es aplicable sin consola → se usa `TerminateProcess` tras 1 s de gracia). Tras terminar, se verifica con `GetExtendedTcpTable` que ningún puerto del rango sigue en `LISTEN` por un PID del Job. Se borra la carpeta temporal de la sesión una vez leídos los XML y copiados a la BD.
6. **Al arrancar la app**: se busca cualquier `ntttcp.exe` hijo huérfano de una ejecución anterior (por nombre + ruta del binario empaquetado) y se termina; se limpia `%TEMP%\NetworkBench\`.

### 11.6. Parseo del XML [H1]

- Parser XML estricto pero tolerante a elementos desconocidos y a orden distinto. Elementos esperados (nombres a confirmar contra la versión empaquetada [V-03]): `total_bytes`, `realtime`, `throughput[@metric='mbps']`, `packets_sent`, `packets_received`, `packets_retransmitted`, `errors`, `cpu[@metric='%']`, `cycles[@metric='cycles/byte']`, `interrupts`, `dpcs`, `bufferCount`, `bufferLen`.
- Campos ausentes → `null` en `EngineResult` (nunca 0). La UI muestra «no disponible» para nulos.
- Si `throughput` está ausente pero hay `total_bytes` y `realtime`, se calcula. Si tampoco → `NB-ENGINE-006` (XML inválido).
- **Coherencia** [H2]: si `|throughputSender − throughputReceiver| / throughputReceiver > 5 %` se añade la observación «Los dos extremos han medido velocidades distintas (X y Y)»; si > 25 % la sesión se marca `NB-RESULT-002` (resultado incoherente) y se sugiere repetir.

### 11.7. Cifra oficial [H1]

- **Velocidad de una dirección** = `throughputBps` del **receptor NTTTCP** de esa dirección.
- El valor del emisor se muestra en Detalles técnicos → Resultado ("Medido en el emisor / en el receptor").
- Retransmisiones y `packetsSent`: del **emisor**. `packetsReceived`, `errors`: del **receptor**. CPU: de cada extremo por separado.

---

## 12. Muestreo en tiempo real [H2]

### 12.1. Fuente

- NTTTCP para Windows no emite estadísticas por intervalo. Las muestras se obtienen de los **contadores del adaptador** seleccionado en cada extremo: `GetIfEntry2` → `InOctets`, `OutOctets` (64 bits), leídos cada **500 ms** desde `startAt` hasta el fin del enfriamiento.
- Además, `cpuPercent` del sistema (contador `\Processor(_Total)\% Processor Time` o `GetSystemTimes`) en cada muestra.
- Cada muestra se envía en `SAMPLE` al otro extremo (§8.3), de forma que ambos disponen de las cuatro series (rx/tx de cada equipo).
- La gráfica de la dirección A→B usa por defecto la serie **rx del receptor** (coherente con §11.7). Se calcula `bps = Δoctets·8 / Δt`.

### 12.2. Declaración de limitación

La spec, la UI (tooltip de la gráfica) y el informe PDF DEBEN declarar: «La gráfica muestra el tráfico total del adaptador durante la prueba. La velocidad oficial es la medida por el motor de prueba.»

### 12.3. Detección de tráfico ajeno

Si la media de las muestras (excluido calentamiento/enfriamiento) supera en más del **10 %** [PROV] el `throughputBps` del motor, se añade la observación «Durante la prueba había otro tráfico en el adaptador (aprox. X Mbit/s)» y la estabilidad se marca «orientativa».

### 12.4. Rendimiento

- Lectura de contadores y envío de `SAMPLE` en el hilo Rust, nunca en la webview.
- La UI recibe eventos Tauri agrupados a **máximo 4 refrescos/s**; la gráfica se anima entre puntos con interpolación en CSS/JS, no con más datos.
- Presupuesto: la app completa (Rust + WebView2) NO DEBE superar el 5 % de CPU de un núcleo de referencia durante la prueba [V-04].

---

## 13. Métricas y motor de interpretación [H2]

### 13.1. Capacidad de referencia

```
capA, capB = velocidad negociada de cada adaptador (bit/s) con su tipo (§15)
reliable(x) = tipo Ethernet físico y velocidad > 0
if plan.expectedCapacityMbps         → ref = manual, refSource = "manual"
elif reliable(capA) and reliable(capB) → ref = min(capA, capB), refSource = "negotiated"
elif alguno es Wi-Fi y el otro fiable/Wi-Fi → ref = min(...), refSource = "wifi" (se etiqueta «orientativo»)
else (virtual, VPN, desconocido)        → ref = null, refSource = "unknown"
```

- «Capacidad aprovechada» = `velocidad / ref`, mostrada como «94 % de un enlace de 10 Gbit/s». Si `ref = null`: la tarjeta muestra «Capacidad: no determinable» con explicación («Uno de los adaptadores es virtual o VPN; indica la capacidad esperada en Opciones avanzadas para obtener este dato»), y el motor no emite veredicto de rendimiento.
- Si capA ≠ capB se explica: «La velocidad máxima viene limitada por el enlace más lento (SERVER-01: 1 Gbit/s)».
- Texto fijo de §18 de `Historias.md` («la capacidad nominal no equivale al throughput útil…») visible como nota en la tarjeta.

### 13.2. Rendimiento [PROV]

Sobre `util = velocidad / ref`, por dirección, y sobre el **mínimo** de ambas direcciones para el veredicto global:

| util | Nivel | Título |
|---|---|---|
| ≥ 0,85 | OK | «Rendimiento acorde a la capacidad del enlace» |
| 0,60 – 0,85 | WARN | «El rendimiento es algo inferior a lo esperable» |
| < 0,60 | PROBLEM | «El rendimiento es claramente inferior a lo esperable» |

Sin `ref`: sin veredicto de rendimiento; el veredicto global se basa en estabilidad, retransmisiones y asimetría.

### 13.3. Estabilidad [PROV]

Sobre las muestras de la serie oficial de cada dirección, excluyendo calentamiento y enfriamiento:

```
mean, sd = media y desviación típica muestral
cv = sd / mean
drops = nº de muestras < 0,5 · mean
level: cv < 0,05 → MUY_ESTABLE; < 0,10 → ESTABLE; < 0,20 → VARIABLE; else MUY_VARIABLE
if drops ≥ 2: level = un nivel peor (mín. MUY_VARIABLE)
```

- Requiere ≥ 10 muestras; si hay menos (pruebas muy cortas o huecos) → «Estabilidad: no evaluable».
- Se guardan en detalles: media, sd, cv, mín, máx, p5, p50, p95, nº de caídas, nº de muestras, nº de huecos.
- Nivel de la sesión = el peor de las dos direcciones.

### 13.4. Asimetría [PROV]

```
asym = |vAB − vBA| / max(vAB, vBA)
asym > 0,20 → ASYMMETRIC («La conexión funciona mejor en una dirección»)
```
Solo se evalúa si ambas direcciones completaron.

### 13.5. Retransmisiones [PROV]

```
ratio = packetsRetransmitted / packetsSent   (del emisor de cada dirección)
< 0,001 → NORMAL; 0,001–0,01 → ELEVATED; > 0,01 → HIGH; datos ausentes → NOT_AVAILABLE
```

### 13.6. CPU

- «Impacto en CPU» por equipo: media de `cpuPercent` de las muestras durante la medición: < 25 % → «bajo»; 25–60 % → «moderado»; > 60 % → «alto» [PROV].
- Si CPU > 60 % en el emisor o el receptor y `util < 0,85`, se añade **posible causa** «La CPU del equipo X estuvo muy ocupada, lo que puede limitar la velocidad». Nunca se afirma que la CPU sea *la* causa.
- Texto fijo: «El uso de CPU incluye todo el sistema, no solo la prueba».

### 13.7. Composición del resultado (`SessionResult`)

```
SessionResult {
  schemaVersion: 1, sessionId, startedAt, finishedAt, status: "completed"|"incomplete",
  initiator: PeerInfo, responder: PeerInfo,          // identidad, nombre, interfaz (§15)
  plan: BenchmarkPlan,
  capacity: { refBps, refSource, capABps, capBBps },
  directions: [ DirectionResult { direction, sender: EngineResult, receiver: EngineResult,
                 officialBps, utilization, stability: {...}, retrans: {...},
                 cpuSender, cpuReceiver, samples: [ {t, rxBps, txBps, cpu} ] } ],
  asymmetry: { ratio, level },
  verdict: { level: "ok"|"warn"|"problem", titleKey, facts[], observations[], causes[], actions[] },
  resultSource: "initiator"|"local",
  engine: { id, version }
}
```

- `verdict.level` = peor de: rendimiento, estabilidad (VARIABLE → warn, MUY_VARIABLE → problem), retransmisiones (ELEVATED → warn, HIGH → problem), asimetría (→ warn), incoherencia (→ warn).
- `facts`, `observations`, `causes`, `actions` son **claves i18n con parámetros**, no texto; la UI y el PDF las renderizan en su idioma. Cada regla del motor aporta sus claves. Las reglas viven en un módulo Rust único (`diagnostic/rules.rs`) con tests unitarios por regla.
- Las «posibles causas» se listan en orden fijo por regla y con los verbos de `Historias.md` §20 («puede deberse a», «conviene revisar»). Ninguna regla puede producir una afirmación de causa única.

### 13.8. Textos de referencia

Se conservan tal cual los ejemplos de `Historias.md` §13, §14, §18, §19, §20, §21, §22 como textos base de las claves i18n en español.

---

## 14. Firewall de Windows [H2]

### 14.1. Reglas

Todas entrantes, acción *permitir*, nombre con prefijo `NetworkBench - `, grupo `NetworkBench`, descripción con la versión que las creó:

| Nombre | Protocolo | Puertos | Programa | Perfiles |
|---|---|---|---|---|
| `NetworkBench - Control` | TCP | puerto de control | `<InstallDir>\NetworkBench.exe` | Dominio, Privado (+Público si se acepta) |
| `NetworkBench - NTTTCP TCP` | TCP | `basePort..basePort+63` | `<InstallDir>\engine\ntttcp.exe` | ídem |
| `NetworkBench - NTTTCP UDP` | UDP | `basePort..basePort+63` | `<InstallDir>\engine\ntttcp.exe` | ídem |
| `NetworkBench - Descubrimiento` | UDP | 5353 | `<InstallDir>\NetworkBench.exe` | ídem (solo si mDNS no está ya permitido por la regla del sistema) |

Alcance mínimo: programa **y** puerto a la vez. Si el usuario cambia un puerto en Ajustes, las reglas afectadas se marcan «desactualizadas» y se ofrece recrearlas.

### 14.2. Elevación

- Un ejecutable auxiliar `NetworkBench.FirewallHelper.exe` (o el propio binario con `--firewall-apply <fichero-json>`), lanzado con `ShellExecuteEx` + `runas`. Recibe **solo** un fichero JSON con la lista de operaciones (`add`/`remove`, nombre, protocolo, rango, programa, perfiles) que él mismo valida contra una lista blanca (solo nombres con prefijo `NetworkBench - `, solo los dos programas de la instalación, solo rangos de 64 puertos).
- Usa la API COM `INetFwPolicy2` (no `netsh`). Devuelve código de salida y JSON de resultado por fichero.
- La app principal NUNCA se ejecuta elevada.
- UAC rechazado → `NB-FW-004` con «Reintentar» y «Ver instrucciones».

### 14.3. Detección de estado

`FirewallManager` (no elevado, la lectura no lo requiere) consulta por `INetFwPolicy2` el estado de cada regla: **presente / ausente / modificada** (algún campo no coincide) / **deshabilitada**. Detecta también si el perfil está gestionado por directiva (`INetFwPolicy2.IsRuleGroupCurrentlyEnabled` + `LocalPolicyModifyState`): si la directiva bloquea cambios locales → `NB-FW-003` con instrucciones para el administrador (lista exacta de reglas a crear en texto copiable).

### 14.4. Diagnóstico durante PREPARING

| Síntoma | Conclusión mostrada | Error |
|---|---|---|
| Canal de control OK, sondeo TCP al puerto NTTTCP del receptor falla, regla NTTTCP ausente/deshabilitada en el receptor | «El Firewall de Windows de SERVER-01 puede estar bloqueando la prueba» → botones **Configurar automáticamente** (en SERVER-01, con UAC allí) / **Ver instrucciones** / **Cancelar** | `NB-FW-002` |
| Igual pero la regla está presente | «Los equipos se comunican pero el tráfico de prueba no llega. Puede haber otro firewall o un dispositivo intermedio bloqueando los puertos N–M» | `NB-FW-005` |
| Peer visible por mDNS pero canal de control no conecta | «El Firewall de Windows de X puede estar bloqueando NetworkBench» (solo se puede arreglar en X) | `NB-FW-001` |

"Configurar automáticamente" en el equipo remoto: el iniciador envía `FIREWALL_FIX_REQUEST`; el receptor muestra el diálogo de consentimiento y lanza su helper; responde con el resultado; PREPARING reintenta el sondeo.

### 14.5. Perfiles

Por defecto Dominio + Privado. Si el adaptador en uso está en perfil Público (`INetworkListManager`), se explica («Windows considera esta red *pública*, lo que suele ocurrir con conexiones VPN o Wi-Fi. Puedes permitir NetworkBench también en redes públicas o cambiar el tipo de red en Configuración de Windows») con botones **Permitir en redes públicas** / **Abrir configuración de red** / **Cancelar**.

### 14.6. Ciclo de vida y gestión

- La BD guarda cada regla creada (nombre, fecha, versión, puertos).
- Ajustes → Firewall: lista de las cuatro reglas con estado, botones **Crear las que faltan**, **Eliminar todas**, **Ver instrucciones manuales** (texto `netsh` equivalente, copiable), y aviso si hay directiva corporativa.
- Desinstalación: se eliminan todas las reglas del grupo `NetworkBench`.

---

## 15. Información de interfaces [H1]

Obtenida en Rust con `GetAdaptersAddresses` + `GetIfEntry2` + WMI/`SetupAPI` para driver:

| Campo | Fuente | Visible en |
|---|---|---|
| `interfaceId` (GUID), `friendlyName`, `description` (modelo) | `GetAdaptersAddresses` | Nivel 2 (modelo), Detalles |
| `type`: `ethernet` / `wifi` / `virtual` / `vpn` / `loopback` / `other` | `IfType` + heurística (nombre de driver, `TunnelType`, `PhysicalMediumType`, `ConnectorPresent`) | Nivel 2 (icono) |
| `linkSpeedBps` (`TransmitLinkSpeed`, `ReceiveLinkSpeed`) | `GetIfEntry2` | Nivel 1 (capacidad) |
| `mtu` | `GetIfEntry2` | Detalles |
| `ipv4[]`, `ipv6[]` (usadas en la prueba resaltadas) | `GetAdaptersAddresses` | Nivel 2 |
| `mac` | `GetAdaptersAddresses` | Detalles |
| `operStatus` | `GetIfEntry2` | PRECHECK |
| `driverProvider`, `driverVersion`, `driverDate` | `SetupAPI` / WMI `Win32_PnPSignedDriver` | Detalles |
| `firewallProfile` (`domain`/`private`/`public`) | `INetworkListManager` | §14.5 |

- Se intercambia en `REQUEST`/`RESPONSE` y se persiste con la sesión (snapshot; si la NIC cambia después, el historial conserva lo que había).
- Si el adaptador desaparece o cambia de estado durante la prueba (`NotifyIpInterfaceChange`) → cancelación con `NB-NIC-002`.

---

## 16. Interfaz de usuario

### 16.1. Navegación [H1]

Barra lateral compacta (iconos + etiqueta) con: **Inicio**, **Historial** [H3], **Ajustes**. Las pantallas de sesión (preparación, ejecución, resultado) se abren sobre Inicio y no permiten navegar fuera hasta terminar o cancelar (el resto de la barra se deshabilita con tooltip «Prueba en curso»).

### 16.2. Pantalla inicial [H1]

- Cabecera: «Este equipo: **DESKTOP-DANIEL** · Intel X550-T2 · 10 Gbit/s · 192.168.10.15» y punto de estado (verde «Visible en la red», gris «Descubrimiento desactivado», rojo «Sin conexión de red»).
- Botón principal **Analizar conexión** → abre el selector de equipo (16.3). Si hay un único favorito o el último peer está disponible, el botón muestra debajo «Última prueba: SERVER-01, hoy 10:32 ✓» con acción rápida «Repetir con SERVER-01».
- Sección «Equipos disponibles» (mDNS) y «Otros equipos» (favoritos/recientes). Botón secundario **Conectar manualmente**.
- Estado sin conexión [§37.20]: si no hay ningún adaptador con `operStatus = up`, se sustituye el contenido por «Este equipo no tiene conexión de red» con acción «Abrir configuración de red».

### 16.3. Selector de equipo y opciones [H1] (avanzadas [H3])

Al elegir un equipo: tarjeta resumen del peer (nombre, IP, enlace, confianza) + «Prueba estándar: TCP, ambas direcciones, ≈ 1 minuto» + enlace «Opciones avanzadas» (desplegable §17) + botón **Iniciar prueba**.

### 16.4. Pantalla de ejecución [H1] (gráfica [H2])

Idéntica en ambos extremos, según `Historias.md` §12 y §34:

- Elemento visual principal: dos tarjetas de equipo (nombre, modelo NIC, enlace) unidas por la línea de conexión con flecha de dirección y la velocidad instantánea grande («9,36 Gbit/s»). Animación sutil del flujo (partículas/gradiente desplazándose) limitada a 30 fps y desactivable por `prefers-reduced-motion`.
- Debajo: gráfica temporal (§16.7), barra de progreso de la dirección actual con «Tiempo restante aproximado: 8 s», y texto de fase («Calentando…», «Probando envío desde DESKTOP-DANIEL», «Recibiendo prueba desde DESKTOP-DANIEL», «Cambiando de dirección», «Analizando resultados…»).
- Estado «Reconectando…» (§8.5) sustituye la velocidad instantánea por un indicador.
- Botón **Cancelar prueba** siempre visible.

### 16.5. Pantalla de resultado [H1] (interpretación [H2])

Jerarquía de `Historias.md` §35:

- **Nivel 1** (una tarjeta grande): velocidad (mín. de ambas direcciones, o la única), «94 % de un enlace de 10 Gbit/s» (o «Capacidad no determinable»), estado con icono y texto (✓ / ⚠ / ✗) y título del veredicto.
- **Nivel 2** (fila de tarjetas): A→B y B→A con sus velocidades y una barra comparativa; Estabilidad; Impacto en CPU (por equipo); Errores/retransmisiones; texto de conclusión con bloques **Hechos / Observaciones / Posibles causas / Qué puedes hacer**. Gráfica completa de la sesión con ambas direcciones (colores distintos + patrón de línea distinto para daltonismo).
- Acciones: **Ver detalles técnicos**, **Exportar** (PDF/JSON/CSV) [H3], **Repetir prueba**, **Nueva prueba**.
- Evolución [H3]: si hay ≥ 3 sesiones previas con el mismo peer, tarjeta «Comparado con las 5 últimas pruebas: −54 %» (§19.4).
- Variantes obligatorias (§37): resultado correcto, con advertencias, problemático, incompleto, cancelado.

### 16.6. Detalles técnicos [H2]

Panel lateral o pantalla completa con las secciones de `Historias.md` §23 (Resultado, TCP/UDP, Sistema, Motor) por dirección y por equipo, valores con unidades exactas, línea de comandos de NTTTCP, XML crudo plegado, y botón **Copiar diagnóstico técnico** (texto plano en el idioma de la app, con todos los valores y sin las claves ni IPs públicas ajenas al usuario).

### 16.7. Gráficas [H2]

- Librería: SVG propio o una librería ligera (p. ej. `d3-shape` para paths + Svelte). Sin canvas pesado.
- Throughput vs tiempo: eje Y autoescalado con unidad única para toda la gráfica (Mbit/s o Gbit/s), ticks «bonitos»; eje X en segundos; zonas de calentamiento/enfriamiento sombreadas; línea A→B y B→A distinguibles por color **y** trazo; tooltip accesible (teclado: flechas recorren las muestras).
- Refresco ≤ 4 Hz de datos; transición suave entre puntos.
- Histórico (§19.4): puntos por sesión, color por veredicto, línea de media móvil.
- Todas con tema claro/oscuro mediante tokens CSS.

### 16.8. Unidades y formatos [H1]

- Interno: `bit/s` entero (u64), bytes u64, tiempos en ms.
- Presentación: < 1 000 Mbit/s → `Mbit/s` sin decimales; ≥ 1 Gbit/s → `Gbit/s` con 2 decimales. Separador decimal según idioma (§4). Porcentajes sin decimales. Bytes en `MB`/`GB` decimales (10^6/10^9) con 1 decimal.
- La misma gráfica usa una única unidad.

### 16.9. Ventana, geometría persistente y responsive [H1]

**Geometría**

- Tamaño mínimo 800×600 (píxeles lógicos); primera ejecución 1100×760 centrada en el monitor principal.
- La app DEBE reabrirse con **el mismo tamaño, la misma posición y el mismo estado (maximizada o no)** que tenía la última vez. Se persisten en `settings.json`: `x`, `y`, `width`, `height` (en píxeles físicos, junto con el factor de escala) y `maximized`. Si estaba maximizada se guarda también la geometría restaurada para que «Restaurar» vuelva a ella.
- Se guarda en todos los caminos de salida: cierre por diálogo/ajuste (§5.2), minimizar a la bandeja, «Salir» de la bandeja (`ExitRequested`), y apagado/cierre de sesión de Windows. Además, se guarda con retardo (500 ms tras el último evento `Moved`/`Resized`) para sobrevivir a un cierre brusco.
- Al restaurar se valida contra los monitores presentes: si la ventana no tiene al menos 100×100 px lógicos visibles en algún monitor (monitor desconectado, resolución cambiada), se recoloca en el monitor principal conservando el tamaño (recortado al área de trabajo si no cabe). Nunca se abre fuera de pantalla.
- La ventana se crea **oculta** (`visible: false`), se aplica la geometría y después se muestra, para que no haya parpadeo de una ventana que cambia de sitio.

**Responsive**

- Puntos de corte por ancho: **compacta** < 1000 px (barra lateral solo iconos, tarjetas en una columna, gráfica 200 px alto, direcciones apiladas), **estándar** 1000–1400, **amplia** > 1400 (A→B y B→A lado a lado, detalles técnicos como panel lateral).
- Escalado de Windows 100–200 % soportado; textos y trazos en unidades lógicas.

### 16.10. Tema y dirección artística [H2]

Se parte de `Historias.md` §32–§33 (minimalismo, tarjetas amplias, jerarquía tipográfica, estados reconocibles) con **dos cambios de intención** respecto al original:

**Animación atrevida.** La app DEBE sentirse viva y espectacular, no discreta. El diseñador tiene libertad para proponer animaciones exageradas y llamativas en:

- Transiciones entre pantallas y apertura/cierre de paneles y diálogos.
- Aparición del resultado: la cifra de velocidad, el porcentaje de capacidad y el veredicto se revelan con una coreografía propia (es el momento «premio» del producto).
- Cambio de dirección de la prueba (A→B → B→A) y la línea de conexión entre los dos equipos.
- Hover, pulsación y foco de botones y tarjetas (microinteracciones con respuesta física: escala, luz, rebote contenido).
- Pantalla inicial y descubrimiento (aparición de equipos encontrados).

Con dos límites innegociables:

1. **Durante `RUNNING_*` la precisión manda** (§12.4): solo se animan la gráfica y el flujo de la conexión; nada de partículas a pantalla completa, blur animado ni bucles pesados. La app entera DEBE mantenerse por debajo del presupuesto de CPU/GPU de V-04. Toda animación de esa pantalla usa `transform`/`opacity` compuestos en GPU, nunca propiedades que fuercen layout.
2. **`prefers-reduced-motion`** se respeta siempre, y existe el ajuste «Reducir animaciones» (§23) que deja transiciones instantáneas o mínimas sin perder información.

Las animaciones nunca retrasan una acción del usuario: un clic responde de inmediato aunque la animación siga.

**Materiales de cristal.** Inspirados en el lenguaje de transparencias tipo *Liquid Glass*, sin copiarlo: identidad propia en color, forma y luz. Se implementan **dentro de la app**, sobre un fondo propio (degradado o luz ambiental que puede reaccionar suavemente al estado: neutro en reposo, cálido en éxito, ámbar en advertencia), no sobre el escritorio:

- Tres materiales definidos como clases/tokens: `glass-chrome` (barra lateral, barra de título, barras de herramientas), `glass-card` (tarjetas), `glass-overlay` (diálogos, paneles laterales), cada uno con `backdrop-filter: blur() saturate()`, borde luminoso de 1 px semitransparente y reflejo sutil superior. Valores de blur/saturación/alpha como tokens, distintos en claro y oscuro.
- Respaldo obligatorio: `@supports not (backdrop-filter)` → material opaco (`--solid`) con el mismo layout. La barra de título usa siempre el plano opaco `--solid`, que a ojo es indistinguible del resto (así se funde con la ventana, ver §16.13).
- La ventana es opaca (`transparent: false`); no se usan Mica/Acrylic del sistema. Funciona igual en Windows 10 y 11, en capturas de pantalla y con cualquier fondo de escritorio.
- El texto denso (tooltips largos, detalles técnicos, XML) va sobre material opaco: el cristal es para superficies, no para lectura prolongada.

**Tema**: automático (sigue Windows) / claro / oscuro. Todos los valores visuales (color, tipografía, espaciado, radios, blur, duraciones y curvas de animación) son tokens en `:root`, redefinidos para oscuro; ningún valor literal en componentes (verificable por script en CI). Estados éxito/advertencia/error siempre con icono + texto, nunca solo color.

### 16.11. Accesibilidad [H2]

`Historias.md` §39 íntegro. Adicionalmente: contraste AA mínimo, toda acción alcanzable por teclado, orden de foco lógico, `aria-live` para cambios de estado de la sesión, respeto de `prefers-reduced-motion`, tamaño de fuente base configurable (100/115/130 %) en Ajustes.

### 16.12. Rendimiento de la UI [H2]

`Historias.md` §48 + §12.4 de esta spec.

### 16.13. Barra de título propia [H1]

La ventana se crea **sin decoración nativa** (`decorations: false`) y la app dibuja su propia barra de título, fundida con el resto de la interfaz. Referencia de implementación: `TitleBar.svelte` de SmartDisk Monitor.

**Composición**

```
┌────────────────────────────────────────────────────────────────┐
│ [icono] NetworkBench                        [ — ] [ ☐ ] [ ✕ ]   │  ← 32 px lógicos
├────────────────────────────────────────────────────────────────┤
│  resto de la interfaz (barra lateral + contenido)               │
```

- Altura 32 px lógicos. Fondo: el plano opaco `--solid` de la app (mismo color que el lienzo), sin línea de separación: no debe percibirse como una barra distinta.
- Zona izquierda: icono de la app (16 px) + nombre en tipografía pequeña y color atenuado. Toda esa franja es **región de arrastre** (`data-tauri-drag-region`); doble clic maximiza/restaura. Los tres botones quedan fuera de la región de arrastre.
- Tres botones de 48×32 px: minimizar, maximizar/restaurar (icono cambia según estado), cerrar. Iconos de 14 px, trazo fino, en el **color de acento** de la app en reposo; al pasar el ratón, fondo suave del acento; el botón cerrar pasa a fondo rojo suave + icono rojo **solo en hover**, nunca en reposo. Transición de color de 120 ms.
- `aria-label` traducido en cada botón; navegables con Tab; sin `title` nativo.
- El botón cerrar dispara el mismo `CloseRequested` que Alt+F4 (§5.2), nunca `exit` directo.
- El estado maximizado se sincroniza escuchando `onResized`/eventos de ventana, no asumiéndolo tras el clic.

**Lo que se pierde conscientemente** (decisión de producto, documentada aquí para no reabrirla):

- El menú *Snap Layouts* de Windows 11 al pasar el ratón sobre maximizar.
- El menú de sistema con Alt+Espacio y el clic derecho sobre la barra.
- El borde y la sombra nativos de la ventana (se dibuja un borde de 1 px con token propio y esquinas redondeadas; Windows 11 mantiene las suyas por DWM).

**Lo que se conserva y DEBE comprobarse en pruebas manuales:** arrastrar a los bordes/esquinas para ajustar (Snap por arrastre), Win+←/→/↑/↓, Aero Shake, miniaturas y previsualización en la barra de tareas, redimensionar desde los cuatro bordes y esquinas (Tauri gestiona los bordes de redimensión en ventanas sin decoración; margen de agarre de 6 px verificado en V-11), correcto comportamiento al maximizar (sin desbordar la barra de tareas) y con escalado 100–200 %.

### 16.14. Aspecto nativo, no web [H1]

La app corre en una webview pero NO DEBE parecer una página web. Reglas verificables:

| Regla | Implementación |
|---|---|
| Enlaces sin subrayar | Ningún `text-decoration: underline` en reposo ni en hover; los enlaces son texto en color de acento o botones de texto. Los enlaces externos (GitHub, ayuda) llevan icono de «abrir fuera». |
| Cursor flecha en controles | `cursor: default` en botones, tarjetas clicables, pestañas, conmutadores y listas (como Win32/WinUI). `cursor: pointer` solo en enlaces que abren algo externo. `cursor: text` solo en campos editables. |
| Sin selección de texto | `user-select: none` global; `user-select: text` en campos, en valores marcados como copiables (cifras de resultado, IPs, huellas, detalles técnicos, XML) y en el diálogo de instrucciones. |
| Sin menú contextual del navegador | `contextmenu` cancelado globalmente. Donde tenga sentido (campos de texto, valores copiables) se ofrece un menú propio Cortar/Copiar/Pegar con estilo WinUI. |
| Sin arrastre de imágenes/texto | `draggable="false"` en imágenes; `dragDropEnabled: false` en la ventana. |
| Sin atajos de navegador | F5/Ctrl+R, Ctrl+F, Ctrl+P, Ctrl+U, Ctrl+±/rueda con Ctrl (zoom), F12 y navegación atrás/adelante con ratón: cancelados en producción (en desarrollo, F12 permitido). |
| Barras de desplazamiento | Finas (overlay) que aparecen al pasar el ratón o al desplazarse y se desvanecen, estilo Windows 11; nunca las del navegador. |
| Controles | Conmutadores, casillas, botones de opción, desplegables, campos, deslizadores y tooltips con aspecto y comportamiento WinUI 3 (tamaños, radios, estados hover/pressed/disabled/focus). Nunca `<select>` nativo ni `alert`/`confirm` del navegador. |
| Foco | Anillo de foco visible solo con navegación por teclado (`:focus-visible`), estilo Windows. |
| Tipografía | La elige el diseñador (puede ser propia); DEBE tener aspecto de aplicación de escritorio y ser legible a tamaños pequeños con ClearType. Se embebe con la app, nunca desde Internet (§24.3). |
| Diálogos | Modales propios centrados en la ventana, con animación de la §16.10, no ventanas nuevas del sistema salvo selectores de fichero (guardar PDF/CSV/JSON), que sí son los nativos. |
| Texto | Sin «Cargando…» genéricos: esqueletos o estados de carga diseñados. Sin emojis como iconos. |

Un script de CI (`verify-native.mjs`) comprueba las reglas mecanizables: ausencia de `text-decoration: underline`, de `cursor: pointer` fuera de la lista blanca, y de `<select>`/`alert(`/`confirm(`/`prompt(` en el código.

---

## 17. Opciones avanzadas [H3]

Panel desplegable en el selector de equipo. Lista **cerrada**; cada campo con explicación contextual (tooltip/ayuda inline) y botón **Restaurar valores recomendados**. El receptor valida con los mismos límites y rechaza con `params_invalid` → `NB-PARAM-001`.

| Parámetro | Control | Rango | Recomendado |
|---|---|---|---|
| Protocolo | TCP / UDP | — | TCP |
| Direcciones | Ambas (secuencial) / Solo este → otro / Solo otro → este / Ambas a la vez | — | Ambas (secuencial) |
| Duración de medición | número | 5–300 s | 20 |
| Calentamiento | número | 0–10 s | 2 |
| Enfriamiento | número | 0–10 s | 2 |
| Streams | número | 1–64 | según enlace (§11.3) |
| Tamaño de buffer | selector | 4 KB – 4 MB (potencias de 2) | 64 KB |
| Puerto base | número | 1024–65000 | 5001 |
| Adaptador local | lista | adaptadores `up` | automático |
| Familia IP | Auto / IPv4 / IPv6 | — | Auto |
| Capacidad esperada | número Mbit/s (opcional) | 1–400 000 | vacío |
| UDP: tasa objetivo | número Mbit/s | 1–100 000 | 50 % de ref o 100 |
| UDP: tamaño de datagrama | número | 64–65 507 bytes | 1 472 |

- «Ambas a la vez» lanza dos ntttcp por equipo (emisor y receptor simultáneos, puertos `base..` y `base+32..`), y se presenta como suma total + por dirección; sin veredicto de asimetría.
- Los valores avanzados se guardan como «último plan usado» por peer y se pueden fijar como predeterminados.
- No existe campo de argumentos NTTTCP libres.

---

## 18. Prueba UDP [H3]

- Solo desde Opciones avanzadas. Al seleccionar UDP se muestra el texto: «TCP y UDP responden a preguntas distintas. TCP mide la velocidad útil que obtiene una aplicación normal. UDP envía a una tasa fija sin regularse, y sirve para ver cuántos paquetes se pierden a esa tasa. Perder paquetes al enviar a la velocidad máxima del enlace es esperable.»
- Métrica principal: **pérdida** = `1 − packetsReceived / packetsSent` (receptor/emisor). Secundarias: throughput recibido, tasa objetivo, tasa real emitida, estabilidad de la serie recibida.
- Umbrales [PROV]: pérdida < 0,1 % «sin pérdidas apreciables»; 0,1–1 % «pérdidas ligeras»; > 1 % «pérdidas significativas». Si la tasa real emitida superó `ref`, el veredicto se acompaña de «la tasa objetivo superaba la capacidad del enlace».
- Jitter: **no** en v1 (NTTTCP no lo mide). La tarjeta no aparece; la spec lo lista como futura.
- Capacidad de VPN/WAN: el usuario fija la tasa objetivo por debajo del ancho contratado para comprobar pérdidas.

---

## 19. Historial y evolución [H3]

### 19.1. Almacenamiento

- SQLite (`rusqlite`, modo WAL) en `%APPDATA%\NetworkBench\networkbench.db`. Ajustes en `%APPDATA%\NetworkBench\settings.json`. Identidad en `identity\` (§6). Logs en `%LOCALAPPDATA%\NetworkBench\logs\`.
- Tablas: `peers` (huella PK, instanceId, displayName, alias, lastAddress, trustLevel, autoAccept, pairedAt, lastSeenAt), `sessions` (sessionId PK, startedAt, finishedAt, status, peerFingerprint, planJson, capacityJson, verdictJson, resultSource, engineVersion, appVersion), `direction_results` (sessionId, direction, senderJson, receiverJson, officialBps, utilization, stabilityJson, retransJson, cpuSender, cpuReceiver), `samples` (sessionId, direction, host: 'local'|'remote', t, rxBps, txBps, cpu), `interfaces_snapshot` (sessionId, host, json), `firewall_rules` (name PK, createdAt, appVersion, ports), `schema_version`.
- Migraciones versionadas al arrancar. Copia de seguridad automática de la BD antes de migrar.

### 19.2. Pantalla Historial

Lista agrupada por fecha relativa («Hoy», «Ayer», «15 sep») con: peer (alias/nombre), velocidades «9,42 / 9,31 Gbit/s», icono+texto de veredicto, etiqueta «Incompleta» si procede, protocolo si no es TCP. Filtros: por equipo, por veredicto, por fecha. Buscar por nombre. Al abrir una sesión se muestra la pantalla de resultado completa (con gráficas reconstruidas de `samples`). Acciones: exportar, eliminar, «Repetir esta prueba» (mismo peer y plan).

### 19.3. Retención

- Se guardan: sesiones `completed`; sesiones canceladas/fallidas si al menos una dirección tiene `EngineResult` de ambos extremos (`status: "incomplete"`). Fallos sin datos → solo log.
- Sin límite temporal. Ajustes → Datos: tamaño de la BD, «Eliminar pruebas anteriores a…», «Eliminar pruebas con [equipo]», «Eliminar todo el historial» (confirmación), «Abrir carpeta de datos».
- Aviso `NB-DISK-001` si el espacio libre en la unidad de `%APPDATA%` < 200 MB.

### 19.4. Evolución

- Al seleccionar un peer en Historial: gráfica de sesiones (x = fecha, y = velocidad oficial por dirección; punto coloreado por veredicto; solo TCP con el mismo par de adaptadores para no mezclar).
- Comparación automática en la pantalla de resultado: media de las **5** [PROV] últimas sesiones completadas TCP con el mismo peer; si la sesión actual difiere más del **20 %** [PROV]: «El rendimiento actual es un 54 % inferior a la media reciente entre estos equipos (9,3 Gbit/s)» / «…superior…». Se etiqueta como observación, nunca como causa.

---

## 20. Exportación [H3]

### 20.1. PDF

- Generado desde una plantilla HTML/Svelte de impresión renderizada por WebView2 (`PrintToPdf`), tamaño A4, con los mismos componentes de gráfica en SVG. Idioma: el de la app (selector en el diálogo de exportación).
- Estructura (`Historias.md` §49): portada (nombre, «Informe de rendimiento de red», «A ↔ B», resultado, veredicto, fecha); resumen; gráficas; características de los enlaces; anomalías y explicaciones (hechos/observaciones/causas/acciones); información técnica; configuración de la prueba; pie con versión de NetworkBench y del motor.
- Antes de generar, diálogo «Este informe contiene: nombres de los equipos, direcciones IP, modelos de adaptador, resultados y configuración de la prueba» con casilla «Ocultar direcciones IP y MAC».
- Nombre de fichero por defecto: `NetworkBench_<peer>_<AAAA-MM-DD_HHmm>.pdf`.

### 20.2. JSON

- `SessionResult` completo (§13.7) con `schemaVersion`, todas las cifras en unidades base (bit/s, bytes, ms), fechas ISO 8601 UTC, `raw` de cada `EngineResult` incluido (XML como cadena). Misma casilla de ocultar IP/MAC.
- Exportación múltiple desde Historial: un array de sesiones en un solo fichero.

### 20.3. CSV

- Codificación UTF-8 con BOM. Separador y decimal según el idioma de la app (español: `;` y `,`; inglés: `,` y `.`), indicado en el diálogo con opción de cambiarlo.
- **Resumen** (`*_resumen.csv`): una fila por sesión y dirección: `sessionId, fecha, equipoLocal, equipoRemoto, direccion, protocolo, velocidadMbit, capacidadRefMbit, aprovechamientoPct, estabilidad, cv, retransmisiones, paquetesEnviados, ratioRetrans, cpuEmisorPct, cpuReceptorPct, streams, bufferBytes, duracionS, veredicto, estado`.
- **Muestras** (opcional, `*_muestras.csv`): `sessionId, direccion, equipo, tMs, rxMbit, txMbit, cpuPct`.
- Desde Historial se exportan varias sesiones en los mismos dos ficheros.

---

## 21. Catálogo de errores [H1] (completo en [H2])

### 21.1. Estructura

Fichero único `errors.json` (embebido) con, por código: `code`, `severity` (`info`/`warning`/`error`/`fatal`), `titleKey`, `descriptionKey`, `causesKeys[]`, `actions[]` (identificadores de acción de la lista cerrada: `retry`, `check_connection`, `configure_firewall`, `show_firewall_instructions`, `enter_ip`, `choose_interface`, `retry_as_admin`, `change_port`, `open_network_settings`, `repeat_test`, `update_app`, `free_space`, `show_details`, `copy_diagnostics`), `techFields[]` (qué datos técnicos se adjuntan). Los textos van en `locales/*.json`.

Formato de código: `NB-<ÁREA>-<NNN>`.

### 21.2. Códigos

| Código | Situación (`Historias.md` §29) | Severidad | Acciones |
|---|---|---|---|
| NB-CONN-001 | No se puede contactar con la dirección (timeout / rechazo) — cubre equipo apagado, app cerrada, sin ruta, VPN caída, firewall | error | retry, check_connection, enter_ip, show_details |
| NB-CONN-002 | Nombre DNS no resoluble | error | enter_ip, retry |
| NB-CONN-003 | IP con formato incorrecto | warning | enter_ip |
| NB-CONN-004 | Responde algo en el puerto pero no es NetworkBench | error | change_port, show_details |
| NB-CONN-005 | Canal de control perdido durante la prueba y no recuperado (Wi-Fi/VPN caídos, peer cerrado) | error | repeat_test, check_connection |
| NB-CONN-006 | Fallo de TLS (certificado inválido / huella distinta a la esperada sin re-emparejar) | error | show_details |
| NB-VERSION-001 | Versiones de protocolo incompatibles | error | update_app |
| NB-PEER-001 | El usuario remoto ha rechazado | info | retry |
| NB-PEER-002 | Nadie respondió en el otro equipo (60 s) | warning | retry |
| NB-PEER-003 | El otro equipo está ocupado con otra prueba | warning | retry |
| NB-PEER-004 | El otro equipo ha cancelado | info | repeat_test |
| NB-PEER-005 | Código de emparejamiento no coincidente | error | retry, show_details |
| NB-PARAM-001 | Parámetros rechazados por el receptor (fuera de límites) | error | show_details |
| NB-PORT-001 | Puerto de control ocupado al arrancar | error | change_port |
| NB-PORT-002 | Rango de puertos NTTTCP ocupado y no se encontró alternativa | error | change_port, show_details |
| NB-FW-001 | Firewall bloquea el canal de control (peer visible por mDNS pero no conecta) | error | show_firewall_instructions, retry |
| NB-FW-002 | Firewall bloquea los puertos NTTTCP (regla ausente/deshabilitada en el receptor) | error | configure_firewall, show_firewall_instructions |
| NB-FW-003 | Firewall gestionado por directiva corporativa, no modificable | error | show_firewall_instructions, copy_diagnostics |
| NB-FW-004 | UAC rechazado / helper falló | warning | retry_as_admin, show_firewall_instructions |
| NB-FW-005 | Reglas presentes pero el tráfico de prueba no llega (otro firewall / dispositivo intermedio) | error | show_firewall_instructions, show_details |
| NB-ENGINE-001 | ntttcp.exe ausente | fatal | show_details (reinstalar) |
| NB-ENGINE-002 | ntttcp.exe alterado (hash) o no ejecutable (antivirus/AppLocker) | fatal | show_details |
| NB-ENGINE-003 | El receptor NTTTCP no llegó a escuchar | error | repeat_test, change_port, show_details |
| NB-ENGINE-004 | NTTTCP no terminó a tiempo | error | repeat_test, show_details |
| NB-ENGINE-005 | NTTTCP terminó con error / inesperadamente | error | repeat_test, show_details |
| NB-ENGINE-006 | XML inválido o sin datos esenciales | error | repeat_test, show_details |
| NB-RESULT-001 | Resultado incompleto (solo una dirección) | warning | repeat_test |
| NB-RESULT-002 | Resultado incoherente (emisor/receptor difieren > 25 %) | warning | repeat_test, show_details |
| NB-NIC-001 | Adaptador seleccionado desconectado o inexistente | error | choose_interface, open_network_settings |
| NB-NIC-002 | El adaptador cambió de estado durante la prueba | error | repeat_test, choose_interface |
| NB-NIC-003 | Familia IP no coincidente (IPv4 disponible pero IPv6 no, o viceversa) | error | choose_interface, show_details |
| NB-NIC-004 | Ningún adaptador conectado | error | open_network_settings |
| NB-PERM-001 | Sin permisos para una operación (escritura en datos, lanzar proceso) | error | retry_as_admin, show_details |
| NB-DISK-001 | Espacio insuficiente (< 200 MB) | warning | free_space |
| NB-DATA-001 | Base de datos corrupta / no migrable | fatal | show_details (se ofrece renombrar y empezar de cero) |
| NB-UNEXPECTED-001 | Error interno no clasificado | error | copy_diagnostics, retry |

Cada uno mostrará un título humano, descripción, causas y acciones según `Historias.md` §28. Ningún código se muestra al usuario sin este envoltorio; el código aparece en pequeño al pie («Ref. NB-CONN-001») para soporte.

---

## 22. Logs y diagnóstico

### 22.1. Logs [H1]

- `tracing` en Rust, nivel INFO por defecto, DEBUG activable en Ajustes → Acerca de (se muestra aviso de que se registran más datos).
- `%LOCALAPPDATA%\NetworkBench\logs\networkbench-AAAA-MM-DD.log`, rotación diaria, máximo 10 ficheros y 50 MB en total.
- Se registran: transiciones de estado con `sessionId`, mensajes del protocolo (tipo, id, tamaño; **no** el payload completo en INFO), líneas de comandos de NTTTCP, resúmenes de resultados, errores con código, eventos de límites (§8.6), operaciones de firewall.
- Nunca se registran: claves privadas, contenido de la BD, huellas completas en INFO (solo 8 hex).
- La webview reenvía `console.error` al log Rust.

### 22.2. Copiar información de diagnóstico [H2]

Ajustes → Acerca de → **Copiar información de diagnóstico** (y Ayuda del menú de bandeja): texto plano con versión de NetworkBench, del motor, de Windows (build), escalado, idioma, adaptadores (§15), estado actual, últimos 5 códigos de error, reglas de firewall y estado, y últimos 500 KB del log. Casilla «Anonimizar nombres y direcciones» (sustituye IPs públicas, nombres de equipo y MAC por marcadores). Se copia al portapapeles y se ofrece «Guardar como .txt».

---

## 23. Ajustes [H3] (General y Firewall parciales en [H2])

| Sección | Ajuste | Tipo | Defecto |
|---|---|---|---|
| General | Nombre visible | texto 1–48 | hostname |
| | Idioma | Auto / es / en | Auto |
| | Tema | Sistema / Claro / Oscuro | Sistema |
| | Tamaño de texto | 100 / 115 / 130 % | 100 |
| | Reducir animaciones | Automático (sigue Windows) / Sí / No | Automático |
| | Arrancar con Windows | bool | off |
| | Al cerrar la ventana | Preguntar / Minimizar a la bandeja / Cerrar NetworkBench | Preguntar |
| Red | Puerto de control | 1024–65535 | 7411 |
| | Puerto base NTTTCP | 1024–65000 | 5001 |
| | Adaptador preferido | lista / Automático | Automático |
| | Familia IP | Auto / IPv4 / IPv6 | Auto |
| | Descubrimiento automático (mDNS) | bool | on |
| Equipos de confianza | tabla §9.3 | — | — |
| Firewall | estado y acciones §14.6 | — | — |
| Datos | ubicación, tamaño, borrado §19.3, exportar todo (JSON) | — | — |
| Actualizaciones | Automáticas | bool | on |
| | Comprobar ahora | acción | — |
| Acerca de | versión, licencias (incl. NTTTCP MIT), registro DEBUG, copiar diagnóstico, abrir carpeta de logs | — | — |

Cambiar puerto de control o base NTTTCP: se aplica al instante (se reabre el socket, se reanuncia mDNS) y marca las reglas de firewall como desactualizadas.

---

## 24. Seguridad y privacidad

### 24.1. Seguridad del protocolo [H1]

- Ninguna ejecución remota arbitraria: el receptor solo ejecuta `ntttcp.exe` con argumentos construidos por él a partir de un `BenchmarkPlan` validado (§17).
- Validación estricta de todo mensaje: tipos, rangos, longitudes, estado en que se recibe. Mensajes inválidos → cierre de conexión + log.
- Autenticación mTLS por huella tras emparejamiento (§8.1, §9). Sin emparejamiento verificado no se acepta ninguna prueba.
- Límites de §8.6.
- Identidad del peer siempre visible (nombre + IP + huella abreviada en el diálogo de aceptación y en Detalles).
- Confirmación del usuario por defecto; aceptación automática solo opt-in por peer.

### 24.2. Saneamiento de datos remotos [H1]

Todo texto recibido (`displayName`, descripción de NIC, driver, stderr de NTTTCP remoto): se eliminan caracteres de control y de dirección bidireccional, se normaliza Unicode NFC, se trunca (nombres 48, descripciones 128, stderr 4 KB) y se renderiza siempre como texto, nunca como HTML.

### 24.3. Privacidad [H1]

- Todo local. Sin cuentas, sin telemetría, sin envíos a servidores externos salvo la comprobación de actualizaciones (Ajustes permite desactivarla; solo envía versión y arquitectura).
- Antes de exportar se informa de qué contiene el fichero (§20.1) con opción de anonimizar.
- El diálogo de aceptación indica qué información se compartirá con el otro equipo (nombre, adaptador, resultados).

---

## 25. Arquitectura software [H1]

```
src/                     Svelte + TS
  lib/api/               wrappers de comandos Tauri y eventos
  lib/stores/            estado de sesión, ajustes, historial
  lib/components/        tarjetas, gráficas, diálogos, TitleBar (§16.13), CloseDialog (§5.2)
  lib/design-system/     tokens.css (color, tipografía, blur, animación), materiales glass-*, controles WinUI
  lib/i18n/              carga de locales, formateo numérico
  lib/window.ts          minimizar/maximizar/cerrar → comandos Tauri
  routes/                inicio, sesión, historial, ajustes, detalles, informe (print)
src-tauri/src/
  platform/ventana.rs    geometría persistente, validación de monitores, CloseRequested (§5.2, §16.9)
  platform/bandeja.rs    icono y menú de bandeja
  identity/              claves, certificado, DPAPI
  discovery/             mDNS (publish/browse)
  control/               TLS, framing, mensajes, máquina de estados de sesión, límites
  pairing/               código de verificación, niveles de confianza
  engine/                trait BenchmarkEngine, ntttcp/{args,process,parser}
  sampling/              contadores NIC/CPU
  netinfo/               adaptadores, rutas, perfiles de red
  firewall/              lectura de reglas (COM), invocación del helper
  diagnostic/            rules.rs, thresholds.json, composición del veredicto
  history/               SQLite, migraciones
  export/                JSON, CSV; orquestación del PDF
  errors/                errors.json, tipos
  logging/
src-tauri/helper/        FirewallHelper (binario aparte, elevado)
engine/                  ntttcp.exe + VERSION + LICENSE
locales/                 es.json, en.json
```

- La webview **nunca** ejecuta procesos ni toca el sistema; todo pasa por comandos Tauri con `capabilities` mínimas.
- Eventos Tauri para: cambio de estado de sesión, muestras (agrupadas), solicitud entrante, lista de peers, progreso de firewall.
- Tests: unitarios en Rust para parser (con XML de muestra de la versión empaquetada), reglas de diagnóstico (tabla de casos), validación de planes, saneamiento; test de integración de dos instancias en el mismo equipo (loopback, puertos distintos) que recorre toda la máquina de estados.

---

## 26. Criterios de aceptación por hito

### [H1]
- Dos instancias en la misma LAN se descubren por mDNS; también se conectan por IP/DNS manual y entre subredes con routing.
- Emparejamiento con código de verificación; el receptor puede aceptar/rechazar; timeout de 60 s.
- Prueba TCP A→B y B→A con el plan estándar; ambos extremos muestran la misma máquina de estados y el mismo resultado; ambos guardan la sesión en la BD.
- Cancelación desde cualquier extremo sin dejar `ntttcp.exe` huérfano (verificado matando la app durante la prueba).
- Resultado estructurado (`SessionResult`) con velocidad, bytes, duración y datos de NIC.
- Errores NB-CONN-*, NB-PEER-*, NB-ENGINE-*, NB-PORT-001, NB-VERSION-001 con título y explicación humana.
- Español e inglés.
- Barra de título propia fundida con la interfaz; arrastre, doble clic, Snap por arrastre y Win+flechas funcionan; escalado 100–200 % correcto.
- La ventana se reabre con el mismo tamaño, posición y estado; con el monitor secundario desconectado se recoloca en el principal.
- Reglas de aspecto nativo (§16.14) verificadas por script en CI.
- Repositorio público con `LICENSE` GPL-3.0-or-later, `THIRD_PARTY_NOTICES.md`, CI verde y una Release de prueba `v0.1.0` publicada por etiqueta con `networkbench-setup.exe` y `latest.json` descargables desde el enlace fijo.

### [H2]
- Gráfica en tiempo real en ambos extremos con ≤ 4 Hz de refresco y consumo de CPU de la app < 5 % de un núcleo.
- Motor de interpretación con hechos/observaciones/causas/acciones; estabilidad; asimetría; retransmisiones; CPU; capacidad de referencia con los tres orígenes.
- Detalles técnicos y copiar diagnóstico.
- Firewall: detección de los tres escenarios de §14.4, creación de reglas con consentimiento y UAC, eliminación, detección de directiva corporativa.
- Catálogo de errores completo (§21.2).
- Tema claro/oscuro; estados de éxito/advertencia/error; accesibilidad §16.11.

### [H3]
- Historial con filtros, reapertura de sesiones con gráfica, evolución por peer y comparación con la media reciente.
- Exportación PDF, JSON y CSV (resumen + muestras), individual y múltiple, con aviso de contenido.
- Opciones avanzadas completas y UDP con pérdida.
- Ajustes completos; bandeja; arranque con Windows; toasts; actualizador funcionando contra `latest.json` de una Release real.
- Diálogo de cierre con «Recordar mi decisión», ajuste «Al cerrar la ventana» y confirmación cuando hay prueba en curso; Alt+F4 y el botón de la barra se comportan igual.
- Animaciones atrevidas en transiciones y resultado; ajuste «Reducir animaciones» y `prefers-reduced-motion` respetados; consumo de la app durante `RUNNING_*` dentro del presupuesto de V-04.
- Materiales de cristal con respaldo opaco verificado (desactivando `backdrop-filter`).
- Instalador NSIS por máquina con desinstalación limpia de reglas; aviso de SmartScreen documentado en README, Release y web.

### Global (`Historias.md` §50 y §54)
El recorrido de §54 se completa sin que el usuario vea el término «NTTTCP» fuera de Detalles técnicos y Acerca de.

---

## 27. Verificaciones empíricas pendientes

| Id | Qué verificar | Afecta a |
|---|---|---|
| V-01 | Asignación de puertos por stream en la versión empaquetada de NTTTCP (`basePort + i`) | §11.4, §14.1 |
| V-02 | Si NTTTCP Windows permite limitar tasa en UDP; en caso contrario, cómo aproximarla | §18 |
| V-03 | Nombres exactos de los elementos del XML y presencia de `packets_retransmitted`, `cpu`, `errors` | §11.6 |
| V-04 | Consumo de CPU de la app durante la prueba; ajuste del intervalo de muestreo si es necesario | §12.4 |
| V-05 | Streams óptimos por velocidad de enlace (1 GbE, 2,5 GbE, 10 GbE, Wi-Fi 6, VPN site-to-site) | §11.3 |
| V-06 | Umbrales de §13.2–§13.6 y §18 en los mismos escenarios, incluyendo una red con problemas provocados (cable defectuoso, duplex mismatch, saturación) | §13 |
| V-07 | Diferencia típica emisor/receptor (para el 5 % / 25 % de §11.6) | §11.6 |
| V-08 | Comportamiento de mDNS en perfil Público y con varios adaptadores (VPN + LAN) | §7.1 |
| V-09 | Fidelidad de `PrintToPdf` de WebView2 con SVG y fuentes embebidas | §20.1 |
| V-10 | Heurística de clasificación de adaptadores virtuales/VPN (WireGuard, OpenVPN, Hyper-V, VMware, Tailscale) | §15 |
| V-11 | Ventana sin decoración en Tauri 2: margen de redimensión, Snap por arrastre, Win+flechas, maximizado sin tapar la barra de tareas, esquinas redondeadas en Win11 y comportamiento en Win10 | §16.13 |
| V-12 | Coste de `backdrop-filter` en WebView2 con varias capas de cristal y animaciones simultáneas, en un equipo modesto (gráfica integrada); ajuste de blur/alpha o desactivación durante `RUNNING_*` si se supera V-04 | §16.10 |

Cada verificación produce un ajuste de `thresholds.json` o de esta spec, y se documenta en `VALIDACION.md`.

---

## 28. Referencias

- `Historias.md` (documento de origen).
- Tauri 2 — https://tauri.app/
- NTTTCP — https://github.com/microsoft/ntttcp y ayuda de línea de comandos.
- Microsoft Network-Performance-Visualization — https://github.com/microsoft/Network-Performance-Visualization (refuerza que la interpretación es el valor del producto).
- DNS-SD — RFC 6763; mDNS — RFC 6762.
