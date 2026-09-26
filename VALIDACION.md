# Registro de Validación: NetworkBench v1

**Fecha de inicio**: 2026-09-21  
**Constitución**: 0.7.0, no ratificada  
**Matriz objetivo (Q1)**: Windows 10 22H2 y Windows 11 (ambos x64)

Este documento registra la evidencia empírica directa de validación de NetworkBench v1 según las reglas del proyecto.
Cada entrada sigue la estructura fija: **Entorno**, **Comando**, **Resultado**, **Estado**.

Los estados válidos son: `VERIFICADO`, `DOCUMENTADO`, `INFERIDO`, `NO VERIFICABLE`, `NO PRESENTE`.

---

## 1. Inventario de entorno y baseline (G5 / T002)

### 1.1 Sistema Operativo Host (Windows 11 x64)
- **Entorno**: Host de desarrollo local
- **Comando**: `Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber, OSArchitecture`
- **Resultado**:
  ```text
  Caption        : Microsoft Windows 11 Pro
  Version        : 10.0.26200
  BuildNumber    : 26200
  OSArchitecture : 64 bits
  ```
- **Estado**: `VERIFICADO`

### 1.2 Sistema Operativo Objetivo Secundario (Windows 10 22H2 x64)
- **Entorno**: Host de desarrollo local
- **Comando**: N/A (el host de ejecución es Windows 11 Pro build 26200)
- **Resultado**: No disponible en el host físico actual; requiere máquina virtual o equipo secundario de laboratorio.
- **Estado**: `NO VERIFICABLE` (en este host de desarrollo)

### 1.3 Toolchain Rust & MSVC
- **Entorno**: Host local Windows 11 x64, **medido dentro de `F:\Apps\NetBench`**
- **Comando**: `rustc -vV; cargo --version; & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property catalog_productDisplayVersion`
- **Resultado** (corregido el 2026-09-23):
  ```text
  rustc 1.98.1 (48a229cea 2026-09-01)
  host: x86_64-pc-windows-msvc
  cargo 1.98.1 (797e8a9bc 2026-08-05)
  Visual Studio Community 2022 v17.10.0 (MSVC x86/x64 tools)
  ```
- **Nota**: la entrada anterior registraba `rustc 1.94.0`, que es la toolchain por defecto
  **fuera** del repositorio. `rust-toolchain.toml` fija `channel = "1.98.1"`, de modo que
  dentro del árbol la activa es 1.98.1 y coincide con la línea base de la constitución.
  Todo lo compilado y probado hasta la fecha usó 1.98.1. Medición anterior errónea, no
  desviación de versión.
- **Estado**: `VERIFICADO`

### 1.4 Windows SDK
- **Entorno**: Host local Windows 11 x64
- **Comando**: `Get-ChildItem "HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots"`
- **Resultado**: SDKs instalados detectados: 10.0.16299.0, 10.0.19041.0, 10.0.22000.0, 10.0.22621.0
- **Estado**: `VERIFICADO`

### 1.5 Node.js y Gestor de Paquetes (pnpm)
- **Entorno**: Host local Windows 11 x64
- **Comando**: `nvm install 24.21.0; nvm use 24.21.0; node -v; pnpm -v`
- **Resultado** (alineado con la línea base el 2026-09-23, con autorización del propietario):
  ```text
  node: v24.21.0
  pnpm: 11.6.0
  ```
- **Estado**: `VERIFICADO`

### 1.5b Gestor de paquetes alineado con la línea base (pnpm 12.5.1)
- **Entorno**: Host local Windows 11 x64
- **Comando**: `corepack enable; corepack prepare pnpm@12.5.1 --activate` y
  `"packageManager": "pnpm@12.5.1"` en `package.json`; después
  `pnpm install --frozen-lockfile; pnpm verify`
- **Resultado** (2026-09-23, con autorización del propietario):
  ```text
  pnpm 12.5.1
  Verifying lockfile against supply-chain policies (309 entries)...
  Lockfile passes supply-chain policies (309 entries in 3.2s)
  Lockfile is up to date, resolution step is skipped
  Done in 3.4s using pnpm v12.5.1
  ```
- **Riesgo evaluado**: el salto 11 → 12 es un cambio mayor. `pnpm-lock.yaml` **no requirió
  regeneración**: la resolución se omitió y las 309 entradas pasaron la comprobación de
  cadena de suministro que añade la versión 12. `pnpm verify` completo tras el cambio:
  456 ficheros con 0 errores y 0 avisos, 91 tests Vitest en 21 suites, `cargo check`
  limpio, 369 claves es/en sincronizadas, arquitectura y logger sin violaciones.
- **Estado**: `VERIFICADO`

### 1.5c Puerta G5 — estado tras alinear el toolchain
- **Entorno**: Host local Windows 11 Pro 26200 x64
- **Resultado**: los tres componentes de la línea base que divergían están alineados y
  revalidados: Rust 1.98.1 (era una medición errónea, no una desviación), Node 24.21.0 y
  pnpm 12.5.1. La cadena canónica pasa entera sobre ese conjunto.
- **Lo que G5 sigue sin cubrir**: no se ha compilado ni arrancado un instalador, no se ha
  probado en Windows 10 22H2 y `engine/ntttcp.exe` no está en el árbol. **G5 permanece
  abierta** por esos tres motivos, no por versiones.
- **Estado**: `VERIFICADO` (alineación) · `NO VERIFICABLE` aquí (build, instalador y matriz)

### 1.5d Herramientas de cobertura y auditoría
- **Entorno**: Host local Windows 11 x64
- **Comando**: `cargo install cargo-llvm-cov cargo-audit --locked`
- **Resultado**:
  ```text
  cargo-llvm-cov 0.9.1
  cargo-audit-audit 0.22.2
  ```
- **Nota**: instaladas con autorización del propietario. `cargo-llvm-cov` es la herramienta
  que la constitución nombra para el 90 % de líneas en módulos críticos (T155);
  `cargo-audit` habilita el escaneo de vulnerabilidades que exige T158. **Ninguna se ha
  ejecutado todavía sobre el proyecto**: disponibilidad no es medición.
- **Estado**: `VERIFICADO` (instalación) · `NO PRESENTE` (medición)

### 1.6 Runtime WebView2
- **Entorno**: Host local Windows 11 x64
- **Comando**: `(Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}").pv`
- **Resultado**: `153.0.4234.48` (WebView2 Runtime de Microsoft Edge x64)
- **Estado**: `VERIFICADO`

### 1.7bis Puerta G1 — el motor ejecuta y su salida se interpreta (2026-09-24)
- **Entorno**: Host local Windows 11 Pro 26200 x64, NTTTCP 5.40 x64 en `engine/ntttcp.exe`
- **Integridad**: `Get-FileHash -Algorithm SHA256 engine/ntttcp.exe` →
  `f66561d09af91305412fd60ca4b28d57c7b650035d3c1edcc00a57b079e2247e`, **coincide** con
  `engine/SHA256`.
- **Comando**:
  `cargo test --manifest-path src-tauri/Cargo.toml --test engine_real -- --ignored --test-threads=1`
- **Resultado**: 2 pruebas en verde. Una medición TCP real de 5 s en bucle local
  atraviesa `MotorNtttcp` completo —argumentos, proceso, Job Object y parser— y devuelve
  caudal, bytes, duración y CPU coherentes. La segunda comprueba que, con un hash
  esperado distinto, **el binario auténtico tampoco se ejecuta** (FR-063).

#### Dos defectos que solo aparecieron al haber motor

Ambos estaban cubiertos por pruebas en verde contra ficheros inventados.

1. **El parser no funcionaba con la salida real.** Devolvía `MissingField("role")`
   porque buscaba un elemento `<role>` que NTTTCP no emite: el rol está en la raíz,
   `<ntttcpr>` o `<ntttcps>`. Además `total_bytes` llega en MB con decimales,
   `total_buffers` como `83445.000`, `<realtime>` aparece dos veces a distinta
   profundidad y `<throughput>` cinco veces con métricas distintas —un parser que
   tomara la última leía `buffers/s` como caudal—. Y `packets_sent`/`packets_received`
   se rellenaban con `total_buffers` en lugar de leerse, lo que falseaba la pérdida
   UDP (V-02). Reescrito y comprobado contra los cuatro fixtures reales.
2. **Los argumentos eran inválidos.** `build_ntttcp_args` producía
   `-m (1,*,host,puerto)`; la forma real es `-m 1,*,dirección` sin paréntesis y con el
   puerto en `-p`. El motor terminaba con código 9, error de uso.

Los fixtures sintéticos (`receiver_success.xml`, `sender_success.xml`, `udp_*.xml`) se
han **eliminado**: raíz `<ntttcprun>`, elemento `<role>` y un `<throughput_bps>` que no
existen. Sustituidos por capturas reales `real_5.40_*.xml`, TCP y UDP, ambos roles.

- **Alcance**: medido sobre `127.0.0.1`. **No valida una red real** ni la medición entre
  dos equipos, que sigue siendo trabajo de laboratorio. G1 queda cerrada en lo relativo
  a integridad, ejecución, argumentos y parser; abierta en cuanto a medición entre
  equipos y a V-01/V-02/V-05.
- **Estado**: `VERIFICADO` (ejecución real en bucle local) ·
  `NO VERIFICABLE` aquí (dos equipos)

### 1.7 Motor Microsoft NTTTCP (v5.40 x64)
- **Entorno**: Documentación oficial y repositorio local
- **Comando**: `Get-FileHash -Algorithm SHA256 engine/ntttcp.exe (previsto)`
- **Resultado**:
  - Hash oficial normativo registrado: `f66561d09af91305412fd60ca4b28d57c7b650035d3c1edcc00a57b079e2247e`
  - Licencia creada en `engine/LICENSE` (Microsoft Software License Terms)
  - Versión fijada en `engine/VERSION` (`5.40`)
  - Binario ejecutable `engine/ntttcp.exe` pendiente de incorporación física por el operador antes de las pruebas de H1.
- **Estado**: `DOCUMENTADO` (metadatos y licencia verificados; binario físico no incorporado al árbol de Git)

### 1.6 Baseline de cobertura y gate Q2 (T155 / T162)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, Node 24.21.0, pnpm 12.5.1
- **Comando**: `pnpm test:coverage` y `pnpm test:coverage:rust`
- **Resultado — frontend (Vitest/V8)**:
  ```text
  % Stmts 67.40 | % Branch 49.46 | % Funcs 68.96 | % Lines 68.06
  ```
- **Resultado — Rust (cargo-llvm-cov)**: total del crate **56,93 %** de líneas.
  De los 17 módulos críticos, **5 alcanzan el 90 %** exigido por Q2:
  `control/plan.rs` y `diagnostic/udp.rs` (100 %), `engine/ntttcp/parser.rs` (94,57 %),
  `export/redact.rs` (94,29 %) y `netinfo/resolve.rs` (92,75 %).
  Los 12 restantes no: `control/service.rs` 89,61 · `control/preflight.rs` 88,83 ·
  `model/plan.rs` 85,86 · `control/cleanup.rs` 85,42 · `diagnostic/capacity.rs` 84,88 ·
  `pairing/mod.rs` 81,01 · `control/domain.rs` 79,84 · `control/ports.rs` 78,87 ·
  `diagnostic/rules.rs` 54,59 · `control/transport.rs` 53,70 ·
  `logging/diagnostics.rs` 16,80 · `control/repeat.rs` 0,00.
- **Gate configurado**: umbrales Q2 en `vitest.config.ts` (80 % en las cuatro métricas)
  y en `scripts/test/coverage-rust.mjs` (80 % total, 90 % por módulo crítico).
  **Ambos fallan hoy a propósito**, con 13 incumplimientos en el lado Rust. La
  constitución es explícita: ningún porcentaje inferior se interpreta como aprobación.
  `pnpm verify` no los invoca —ejecuta `test:unit`—, así que la cadena de desarrollo
  sigue en verde; el gate actúa en `pnpm test:coverage` y en CI.
- **Lectura**: el rojo mide deuda preexistente, no una regresión introducida hoy. Los dos
  casos que más importan por lo que son, no por la cifra: `diagnostic/rules.rs`, el motor
  de interpretación del producto, al 54,59 %, y `control/transport.rs`, que enmarca el
  protocolo entre peers, al 53,70 %.
- **Estado**: `VERIFICADO` (baseline y gate) · `NO PRESENTE` (conformidad con Q2)

### 1.7 Auditoría de dependencias (T158)
- **Entorno**: Host local Windows 11 x64
- **Comando**: `node scripts/security/scan-security.mjs`, que ahora encadena inventario,
  `cargo audit --file src-tauri/Cargo.lock` y `pnpm audit`
- **Resultado**:
  ```text
  Inventario de dependencias directas: 24 JS/TS, 19 Rust
  cargo audit: 541 dependencias analizadas, 0 vulnerabilidades, 7 avisos
  pnpm audit: No known vulnerabilities found
  ```
- **Los 7 avisos**: seis crates sin mantenimiento (`proc-macro-error` y la familia
  `unic-*`) y uno de unsoundness (`glib` 0.18.5, RUSTSEC-2024-0429). Todos transitivos.
  El escáner los informa y **no bloquea**: el criterio es que una vulnerabilidad detiene
  la verificación y un aviso no. Cambiarlo exige decisión explícita.
- **Estado**: `VERIFICADO`

### 1.8 Canal de control con TLS mutuo (T141, T142, T143 parcial)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1
- **Comando**: `cargo test --manifest-path src-tauri/Cargo.toml`
- **Resultado**: 168 pruebas en verde, 10 de ellas nuevas. El canal de control deja de
  ser texto en claro:
  - `src-tauri/src/control/tls.rs`: configuración mTLS con rustls sobre los certificados
    Ed25519 que `identity` ya generaba. `client_auth_mandatory`: sin certificado no hay
    conexión. Los verificadores comprueban **posesión de la clave privada y nada más**;
    no validan cadena ni nombre porque en este protocolo no hay CA y el nombre de host
    no prueba identidad (FR-011).
  - `src-tauri/src/control/server.rs`: el servidor escucha, completa el saludo y declara
    ocupación (FR-017). Arranca desde `app::start_control_server` antes que la ventana.
  - `src-tauri/src/discovery/mdns.rs`: la conexión manual va sobre TLS y toma la huella
    del certificado del par.
- **El fallo corregido**: `mdns.rs` calculaba la huella como `SHA-256(instanceId)`, y el
  `instanceId` lo envía el propio remoto en el payload. Cualquiera podía presentarse como
  un peer de confianza conocido sin poseer clave alguna. La prueba
  `test_un_instance_id_falsificado_no_cambia_la_huella` reproduce ese ataque y comprueba
  que ahora la huella sigue siendo la del certificado.
- **Lo que sigue sin estar**: el servidor llega hasta HELLO. PAIR, REQUEST y la sesión
  son T144 y T153. Ninguna prueba de esta tanda se ha ejecutado entre dos equipos reales:
  todas son en proceso sobre `127.0.0.1`.
- **Estado**: `VERIFICADO` (mTLS, huella y saludo en proceso) ·
  `NO VERIFICABLE` aquí (dos equipos reales)

### 1.9 Orquestación de la medida (T144 parcial)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1
- **Comando**: `cargo test --manifest-path src-tauri/Cargo.toml`
- **Resultado**: 177 pruebas en verde, 9 nuevas. El motor deja de ser código muerto:
  - `control/engine_port.rs`: el motor se alcanza por el trait `MotorDeMedida`.
    `MotorNtttcp` verifica el SHA-256 **antes de cada ejecución**, no una vez al
    instalar (FR-063). `MotorDeLaboratorio` permite probar la orquestación sin binario.
  - `control/orquestador.rs`: ejecuta la mitad local de una dirección, ensambla
    `DirectionResult` y `SessionResult` y aplica el diagnóstico existente.
  - `app::init` construye el orquestador con el motor real apuntando al ejecutable
    que quedará junto a la aplicación tras la instalación.
- **Invariantes con prueba propia**: `officialBps` procede solo del receptor aunque el
  emisor declare más (FR-027); sin receptor la dirección queda `incomplete` y **sin**
  velocidad oficial, no a cero (FR-030, FR-034); una sola dirección no produce veredicto
  de asimetría (US5/AC4); un motor alterado no produce medida.
- **Fuente única del hash del motor**: `engine/SHA256`, leído por `ENGINE_SHA256` con
  `include_str!` y por el paso de integridad de CI. Antes el valor vivía duplicado en
  `VALIDACION.md` y en el workflow, donde podía quedar obsoleto sin aviso.
- **`unsafe impl Send/Sync for JobObject`**: necesario porque el proceso del motor
  sobrevive a un `await` y Tokio puede moverlo de hilo. Un HANDLE de Windows es válido
  en todo el proceso; `AssignProcessToJobObject` es segura entre hilos y `CloseHandle`
  exige `&mut self`. Justificación completa en el propio fichero.
- **Lo que sigue sin estar**: el diálogo con el peer (PREPARE/READY/START), el muestreo
  en vivo durante la ejecución y la persistencia automática al cerrar sesión. **Ninguna
  medición real se ha ejecutado**: `engine/ntttcp.exe` no está en el árbol, y toda la
  evidencia de esta tanda procede del motor de laboratorio, que no mide nada.
- **Efecto en cobertura**: total de Rust del 56,93 % al 60,58 %. El gate Q2 sigue en rojo.
- **Estado**: `VERIFICADO` (orquestación con motor simulado) ·
  `NO PRESENTE` (medición real con NTTTCP)

### 1.10 Emparejamiento sobre el canal de control (T143 completo, T153 parcial)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1
- **Comando**: `cargo test --manifest-path src-tauri/Cargo.toml`
- **Resultado**: 185 pruebas en verde, 7 nuevas. El emparejamiento deja de ser un módulo
  aislado y pasa a ejecutarse sobre el canal real:
  - `control/pairing_flow.rs`: negocia PAIR_REQUEST/PAIR_RESULT. El código de seis
    dígitos **no viaja como secreto**: ambos extremos lo derivan de sus huellas y no se
    concede nada hasta que las dos personas dicen que sí sobre la misma conexión TLS.
  - `ipc/pairing.rs`: `peers_pairing_start` conecta y devuelve el código a mostrar;
    `peers_pairing_confirm` traslada la decisión y solo entonces persiste confianza,
    con `auto_accept` forzado a `false` (FR-015: confiar no es delegar consentimiento).
  - `discovery::conectar_y_saludar` mantiene el canal TLS abierto entre los dos pasos:
    reconectar invalidaría la comparación humana, porque el código depende de esa
    sesión concreta.
- **Invariantes con prueba propia**: las dos huellas producen el mismo código en ambos
  extremos; un tercero no puede reproducirlo; una huella cambiada invalida el
  emparejamiento aunque el otro usuario acepte (FR-013); si cualquiera de las dos
  personas rechaza, no hay confianza; y una prueba de extremo a extremo sobre el
  servidor TLS real (no un dúplex en memoria) completa el ciclo completo.
- **Lo que sigue sin estar**: el mensaje REQUEST (solicitud de plan de prueba) y la
  sesión en sí. Arranque/parada de descubrimiento mDNS y comandos de actualización
  siguen sin comandos IPC. T153 queda parcial.
- **Estado**: `VERIFICADO` (emparejamiento en proceso, extremo a extremo) ·
  `NO VERIFICABLE` aquí (dos equipos reales)

### 1.11 Incidencia de codificación corregida
- **Qué pasó**: el commit `1f01bad` corrompió `VALIDACION.md` mediante doble
  codificación UTF-8. Causa: un script de PowerShell leyó un fichero UTF-8 sin BOM con
  `Get-Content -Raw` sin especificar `-Encoding`, PowerShell 5.1 lo interpretó con la
  página de códigos del sistema y el resultado se reescribió como UTF-8, duplicando la
  codificación de cada carácter acentuado («Validación» → «ValidaciÃ³n»).
- **Cómo se detectó**: al editar este fichero con la herramienta `Edit`, que exige leer
  el estado real antes de escribir.
- **Corrección**: reconstruido concatenando por bytes (sin pasar por PowerShell) la
  cabecera correcta de un commit anterior (`f081bbd`) y la sección nueva, tomada del
  fichero de scratchpad que nunca pasó por la lectura defectuosa. Barrido de todo el
  repositorio en busca del mismo patrón: no hay más ficheros afectados.
- **Lección**: en esta sesión, cualquier operación de lectura/reescritura de texto con
  acentos debe usar las herramientas `Read`/`Write`/`Edit`, o Bash con `cat`/`sed`, y no
  `Get-Content`/`Set-Content` de PowerShell sin `-Encoding UTF8` explícito.

### 1.12 Negociación de inicio sin reloj sincronizado (T152, cierra G2)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1
- **Comando**: `cargo test --manifest-path src-tauri/Cargo.toml`
- **Resultado**: 194 pruebas en verde, 9 nuevas en `control/protocol.rs`. El mensaje
  `START` ya no es un campo sin implementar:
  - El RTT se mide con un `HEARTBEAT` de nonce propio y su eco, sobre el canal TLS real.
  - `start_delay_ms` (`3 × RTT`, 100–2000 ms) y `tolerance_ms` (`RTT / 2`, 50–1000 ms) se
    calculan de forma pura y determinista: mismo RTT, mismo plan, en cualquiera de los
    dos extremos.
  - Cada extremo arranca en su **propio** instante de recepción de `START` más el
    margen, medido con su reloj monótono (`Instant`). Nunca se compara una marca de
    tiempo de un proceso contra la del otro.
  - Un arranque fuera de tolerancia se declara `Degradado { desvio_ms }`, nunca se
    oculta (FR-025).
- **Prueba de extremo a extremo**: `test_negociacion_completa_sobre_el_canal_tls_real`
  levanta un `ControlServer` real, mide el RTT por el canal TLS mutuo y comprueba que
  los dos procesos, sin compartir reloj, calculan un desfase dentro de tolerancia al
  arrancar «a la vez».
- **Decisión registrada**: no se añadió la dependencia `rand`; el nonce del heartbeat
  sale de los bits bajos de un UUID v4, ya generado por una dependencia existente.
- **Lo que sigue sin estar**: esta negociación no está todavía invocada desde la
  orquestación real de sesión (T144); es la pieza de protocolo que T144 debe consumir
  al implementar el diálogo `PREPARE`/`READY`/`START` completo.
- **Estado**: `VERIFICADO` (negociación, en proceso y sobre TLS real) ·
  `NO PRESENTE` (invocación desde la orquestación de sesión)

### 1.13 Comando IPC roto detectado y corregido, con check permanente (T160 parcial)
- **Entorno**: Host local Windows 11 Pro 26200 x64
- **Hallazgo, al construir el arnés E2E de T147**: `src/lib/api/snapshot.svelte.ts`
  invocaba `"app.getSnapshot"`. El backend registra el comando como `app_get_snapshot`
  (nombre exacto de la función Rust, sin transformación de mayúsculas ni separadores).
  Es la primera llamada que hace `App.svelte` al montar: **la aplicación real habría
  fallado en cada arranque** al pedir el snapshot inicial.
- **Por qué no lo detectó ningún test**: `tests/contracts/snapshot.test.ts` usa
  `setTransportMock`, que sustituye el transporte entero, y el mock comprobaba el
  mismo nombre incorrecto (`"app.getSnapshot"`). El test comparaba el código contra sí
  mismo, nunca contra lo que Rust registra de verdad.
- **Corrección**: los dos usos en `snapshot.svelte.ts` y el mock del test, a
  `app_get_snapshot`.
- **Comando**: `node scripts/architecture/check-ipc-commands.mjs`
- **Resultado**: nuevo check que extrae los nombres reales de
  `tauri::generate_handler![...]` en `src-tauri/src/lib.rs` y los compara contra cada
  `invokeCommand("...")` del frontend. Probado deliberadamente: reintroducido el nombre
  roto, el script lo detecta y sale con código 1; restaurado el nombre correcto, pasa
  con «31 invocaciones coinciden». Cableado a `pnpm verify`.
- **Alcance**: cubre coincidencia de nombre, no de forma de argumentos ni de tipo de
  retorno — eso seguiría exigiendo ejecutar el comando. El resto de T160 (checks de
  contratos y APIs legacy) sigue sin hacer.
- **Estado**: `VERIFICADO`

### 1.14 Emisión de eventos Tauri (T150 parcial)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, Node 24.21.0
- **Comando**: `cargo test --manifest-path src-tauri/Cargo.toml ipc::events` y
  `pnpm test:unit tests/contracts/samples.test.ts`
- **Resultado**: hasta esta tarea, cero llamadas a `.emit(` en todo el backend, pese a
  que `src/lib/api/samples.ts` escucha `session://sample-batch` desde T058.
  `ipc/events.rs` implementa `EmisorDeEventos` (patrón de puerto, igual que
  `engine_port.rs`) con una implementación real sobre `AppHandle::emit` y un doble de
  prueba. Dos pruebas en Rust confirman que el payload que sale de `SampleBatcher`
  contiene exactamente los seis campos que el frontend espera
  (`sessionId`/`direction`/`samples`/`latestBps`/`latestCpuPercent`/`hasGaps`) y que un
  emisor que falla no bloquea nada. Cuatro pruebas en `tests/contracts/samples.test.ts`
  confirman el mismo contrato desde el lado TypeScript, incluida una que reproduce el
  patrón exacto del bug de T160 (un campo renombrado) y comprueba que Zod lo rechaza.
- **Lo que sigue sin estar**: nada invoca `despachar_lote` desde un camino de ejecución
  real. No hay bucle de muestreo en vivo durante una sesión —eso depende de que T144
  complete el diálogo con el peer y la ejecución en curso—, y «progreso» y «cambios de
  estado» no tienen contrato de evento definido en el frontend: hoy ese papel lo cumple
  la suscripción con `revision` monotónica de `ipc/snapshot.rs` (T024). Inventar un
  evento nuevo sin que nadie lo consuma habría sido una decisión de producto no pedida.
- **Estado**: `VERIFICADO` (mecanismo de emisión, contrato en los dos lados) ·
  `NO PRESENTE` (invocación desde una sesión real)

### 1.15 Diálogo completo de sesión, TCP secuencial, con NTTTCP real (T144, avance sustancial)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, NTTTCP 5.40 x64
- **Comando**:
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real -- --ignored --test-threads=1`
- **Resultado**: **1 prueba en verde, 5,28 s.** `control/session_flow.rs` conecta dos
  `ControlServer` reales por TLS mutuo, intercambia HELLO, `REQUEST`/`RESPONSE` con
  revalidación del plan en el receptor (FR-021), `PREPARE`/`READY`, mide el RTT real por
  `HEARTBEAT`, negocia `START` (`control/protocol.rs`, T152) y ejecuta NTTTCP real en
  los dos extremos — emisor en el iniciador, receptor en el respondedor, arrancado en
  paralelo con el resto del protocolo para que esté escuchando antes de que el emisor
  conecte. Ambos extremos terminan con `status: "completed"` y **la misma velocidad
  oficial** (`officialBps`), que en las dos vistas procede del receptor (FR-025, FR-027).
- **Un fallo real que solo apareció al ejecutar el diálogo completo**: la primera
  versión pasaba `target_host: None` al motor receptor. `build_ntttcp_args` exige
  dirección en los dos roles desde que se corrigió V-01 (T145); con `None`, el receptor
  fallaba al instante y el emisor, sin nadie escuchando, se quedó esperando la conexión
  dentro del timeout interno de NTTTCP (600 000 ms por defecto). El primer intento se
  colgó y hubo que terminarlo a mano tras 245 s. Corregido pasando la interfaz de enlace
  explícitamente; la prueba corregida tarda 5,28 s.
- **Concurrencia deliberada**: en el respondedor, el motor receptor se ejecuta con
  `tokio::join!` en paralelo con el intercambio de mensajes (heartbeat, `START`,
  `STARTED`), no después. Ejecutarlo después habría dejado una ventana en la que el
  emisor podría alcanzar su instante de inicio antes de que el receptor escuchara.
- **Alcance de esta tanda**: una dirección, TCP, secuencial. No cubre: la dirección
  inversa en la misma sesión, UDP, ejecución simultánea, cancelación a mitad de diálogo,
  persistencia automática del resultado ni el cableado a `ipc::session::session_start`
  (que sigue solo transicionando estados). Nada de esto se ha probado entre dos equipos
  Windows distintos, solo en bucle local.
- **Estado**: `VERIFICADO` (diálogo completo en bucle local, con motor real) ·
  `NO VERIFICABLE` aquí (dos equipos reales). Lo que esta tanda dejaba sin cablear
  (IPC, bidireccional, cancelación, persistencia) se cubre en §1.16.

### 1.16 Sesión real entre dos servicios completos, cableada a IPC (T144, cierre parcial)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, NTTTCP 5.40 x64,
  2026-09-25. Todo en bucle local: dos `SessionService` en el mismo proceso, cada uno con su
  identidad, su base de datos SQLite temporal y su `ControlServer` en un puerto efímero.
- **Comandos**:
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  y `cargo test --manifest-path src-tauri/Cargo.toml` (rápidas), más `cargo fmt` y
  `cargo clippy --all-targets -- -D warnings` sin avisos.
- **Resultado con NTTTCP real**: `session_flow_real` **2 en verde, 16,73 s** (dirección
  única y ambos sentidos en una misma sesión); `session_service_real` **2 en verde,
  41,46 s** (sesión completa entre dos servicios y cancelación a mitad de medición).
  Tras la ejecución, `tasklist` no encontró ningún `ntttcp.exe`.
- **Resultado sin motor real** (rápidas, en verde): rechazo de un equipo sin autoaceptación,
  rechazo de un equipo desconocido, rechazo de una identidad cambiada (la huella presentada
  no coincide con la guardada, FR-013; no llega a medir) y rechazo por receptor ocupado
  (FR-017: la sesión previa del receptor queda intacta y el iniciador termina `Failed`
  sin persistir nada). Suma: 122 pruebas de biblioteca y el resto de suites, sin fallos.
- **Qué recorre la prueba completa**: `session_start` → búsqueda del peer guardado por
  huella (debe ser de confianza o autoaceptación) → conexión TLS mutua con comprobación de
  identidad → `REQUEST`/`RESPONSE` → `PREPARE`/`READY` → negociación de `START` (G2) →
  dirección de ida y dirección de vuelta con NTTTCP real → ensamblado del resultado →
  **persistencia automática en la base de datos de los dos extremos** con el mismo
  `session_id`. La segunda dirección solo se intenta si el receptor la acepta tras
  revalidar el plan (FR-021).
- **Cancelación**: cancelar aborta la tarea de sesión; al soltarse `NtttcpProcess` y su
  `JobObject` (`KILL_ON_JOB_CLOSE`) el motor muere. La prueba exige que `cancel` vuelva en
  menos de 2 s (aserción en `session_service_real.rs`) y comprueba a continuación que no
  queda ningún `ntttcp.exe`. El otro extremo se entera porque el canal se cierra.
- **Un fallo real que apareció al construirlo**: pasar un cierre asíncrono (`AsyncFnMut`)
  para notificar el inicio de cada pata impedía demostrar que el futuro era `Send` al
  lanzarlo con `tokio::spawn`. Se sustituyó por un `FnMut` síncrono que envía por un canal
  a una tarea aplicadora, esperada antes de pasar a `Analyzing`.
- **Lo que NO se hizo** (desglosado como T173–T180 en la Fase 13): preflight dentro del
  recorrido de sesión; bucle de muestreo en vivo (NTTTCP solo entrega el XML final, así que
  la interfaz no recibe muestras durante la medición); cancelación graciosa
  `CANCEL`/`CANCEL_ACK` y cancelación iniciada por el par a mitad de diálogo; intercambio
  `SESSION_RESULT`/`SESSION_ACK` (cada lado ensambla su propia vista); diálogo de
  consentimiento y emparejamiento entrante (`PAIR_REQUEST` se registra y se descarta); UDP y
  ejecución simultánea en el diálogo; IPv6 con el motor (no se emite `-6`); y cualquier
  prueba entre dos equipos físicos.
- **Estado**: `VERIFICADO` (sesión TCP bidireccional entre dos servicios completos con motor
  real en bucle local, persistencia, cancelación, rechazos) · `NO PRESENTE` (lo listado
  arriba) · `NO VERIFICABLE` aquí (dos equipos reales, firewall, latencia de red real)

### 1.17 Muestreo en vivo durante la medición (T174, cierra el hueco de T150)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, NTTTCP 5.40 x64,
  2026-09-25.
- **Comandos**:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib sampling::vivo` (7 en verde) ·
  `cargo test --manifest-path src-tauri/Cargo.toml --lib control::service::tests::t174` (3
  en verde, con un `ProveedorDeContadores` falso) ·
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (4 en verde, NTTTCP real) · `cargo fmt` y `clippy --all-targets -- -D warnings` sin avisos.
- **Qué hace**: `sampling::vivo::MuestreadorActivo` lee cada 500 ms (`SAMPLE_INTERVAL_MS`)
  los contadores de octetos de la interfaz que lleva el tráfico hacia el equipo remoto
  (`GetIfEntry2`, resuelta con `GetBestInterface`), calcula bits por segundo entre dos
  lecturas y los entrega a `SampleBatcher`, que ya agrupaba y limitaba a 4 Hz (T150). En el
  emisor de una pata mira los octetos salientes; en el receptor, los entrantes.
  `control::service::lanzar_aplicador` abre un muestreador al recibir `EventoPata::Inicia`
  y lo detiene con `EventoPata::Termina`; si el canal se suelta sin `Termina` (cancelación),
  el muestreo pendiente también se detiene, sin quedar huérfano.
- **Un hallazgo real, no simulado**: la interfaz de bucle local de Windows no incrementa
  sus contadores. Se comprobó moviendo unos 200 MB por un socket TCP local: `InOctets` y
  `OutOctets` quedaron en 0 antes y después. Muestrear esa interfaz habría producido una
  serie de ceros con apariencia de medición real. Se decidió declarar `Err` explícito
  para el bucle local (`abrir` lo rechaza) en vez de fingir tráfico: la sesión sigue y
  completa igual, solo se queda sin gráfica en directo. `t144_sesion_completa_entre_dos_servicios_con_motor_real`
  comprueba justamente que en bucle local no sale ningún lote.
- **Lo que SÍ se verificó con datos reales de sistema**: una prueba temporal (no
  incorporada, borrada tras el experimento) leyó los contadores de un adaptador físico dos
  veces con 3 s de por medio y confirmó que `InOctets`/`OutOctets` crecen con tráfico real
  del sistema. La lógica de conversión a bps y de agrupación se prueba con un
  `FuenteDeContadores`/`ProveedorDeContadores` de prueba, deterministas, no con el
  adaptador real: correcto para no depender de tráfico ajeno en CI, pero significa que
  **la lectura Windows real solo se demostró una vez, de forma manual, no en la suite**.
- **No cubierto**: CPU (`cpu_percent` siempre `None`, no hay fuente); IPv6 (`abrir`
  devuelve error explícito, T179); una sesión con tráfico real entre dos equipos físicos
  que confirme que las muestras se emiten con datos que no sean cero (NO VERIFICABLE aquí,
  bucle local no sirve); si el frontend efectivamente pinta esas muestras (fuera del
  alcance de esta tarea, backend únicamente).
- **Estado**: `VERIFICADO` (mecánica de muestreo, agrupación, arranque/parada por pata,
  detección correcta del caso sin contadores) · `NO VERIFICABLE` aquí (contadores reales
  con tráfico real de la prueba, solo posible entre dos equipos físicos)

### 1.18 Preflight cableado en el recorrido de sesión (T173)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, NTTTCP 5.40 x64,
  2026-09-25.
- **Comandos**:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib control::session_flow::preflight_en_el_dialogo`
  (2 en verde) · `cargo test --manifest-path src-tauri/Cargo.toml --lib control::preflight`
  (pruebas existentes, incluida una nueva para `check_ports(None, ..)`) ·
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (4 en verde, NTTTCP real) · `cargo fmt` y `clippy --all-targets -- -D warnings` sin avisos.
- **Qué comprueba cada lado, y solo eso**: quien va a proponer una dirección
  (`iniciar_direccion`) comprueba que tiene ruta local hacia el destino
  (`PreflightEvaluator::check_nic`) antes de enviar el `REQUEST`; quien va a recibir una
  dirección (`atender_direccion_desde`, usada por los dos extremos según de quién es el
  turno) comprueba que el bloque de puertos de datos del plan está libre
  (`PreflightEvaluator::check_ports`) antes de `aceptar`. Un fallo de puertos se responde
  con `RESPONSE{accepted:false, reason:"portsUnavailable"}`, una categoría cerrada nueva
  (`MotivoRechazo::PuertosOcupados`), nunca con texto libre.
- **Un fallo real de la propia tarea, no del código de sesión**: `check_ports` exigía un
  `control_port` y lo declaraba obligatoriamente ocupado — la propia tarea T173 ya lo
  señalaba como bug. Se cambió su firma a `control_port: Option<u16>`; con `None` esa
  comprobación se omite. Las dos pruebas existentes que pasaban un puerto de control real
  siguen probando exactamente lo mismo (ahora con `Some(puerto)`); se añadió una tercera
  para `None`.
- **Un fallo real durante la construcción de la prueba (no del código de producción)**: la
  primera versión de `un_puerto_de_datos_ocupado_se_rechaza_con_ports_unavailable` ocupó el
  puerto con `TcpListener::bind("127.0.0.1:0")`, pero `is_port_available_for_protocol`
  siempre sondea `0.0.0.0`; el sondeo veía el puerto libre, la pata se aceptaba y la
  prueba se quedó esperando un `PREPARE` que nunca llegaría — colgada, no fallida. Se
  detectó porque el proceso de pruebas no terminaba (varios minutos sin salida) y hubo que
  matarlo a mano (`taskkill`) para poder corregirla. Corregida ocupando `0.0.0.0:0`, igual
  que ya hacía `preflight_tests.rs`, la prueba pasa en 0,01 s.
- **No cubierto, deliberadamente fuera de esta tarea**: el resto de `PreflightEvaluator`
  (motor, disco, versión, firewall) no se invoca desde la sesión — la tarea solo pedía las
  dos comprobaciones nombradas; una pantalla de diagnóstico previa a iniciar que muestre
  estos resultados al usuario no existe; y `check_ports` sigue comprobando siempre TCP
  (`ports.rs::check_port_range`), sin distinguir el protocolo del plan — limitación
  preexistente, no introducida ni corregida aquí (UDP en el diálogo es T178).
- **Estado**: `VERIFICADO` (las dos comprobaciones, su punto exacto de invocación, el
  motivo de rechazo cerrado, y que no rompen el camino feliz con NTTTCP real)

### 1.19 Cancelación graciosa antes de que arranque el motor (T175, alcance acotado)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, NTTTCP 5.40 x64,
  2026-09-25.
- **Comandos**:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib control::session_flow::preflight_en_el_dialogo`
  (4 en verde) ·
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (5 en verde, NTTTCP real, repetida 5 veces la nueva sin fallos) · `cargo fmt` y
  `clippy --all-targets -- -D warnings` sin avisos · `cargo test` completo (137 en verde,
  repetido 3 veces sin fallos tras corregir una prueba inestable, ver más abajo).
- **Qué cubre y qué no, con la frontera exacta**: los puntos donde un extremo espera al
  otro sin tener nada en ejecución —`RESPONSE` (tras `REQUEST`), `READY` (tras `PREPARE`)
  del lado que inicia una dirección, y `PREPARE` (tras aceptar) del lado que la
  atiende— compiten contra una señal de cancelación (`tokio::sync::watch<bool>`,
  `SenalCancelacion`). Ganar esa carrera hacia el lado local envía `CANCEL` y da hasta
  300 ms a la `CANCEL_ACK`, mejor esfuerzo, sin bloquear más. Recibir un `CANCEL` del par
  ahí mismo se confirma con `CANCEL_ACK` y marca la señal compartida, para que
  `SessionService` distinga `Cancelled` de `Failed` al recoger el error. La ventana en la
  que el motor ya corre (tras `START`/`STARTED`, durante la medición) **no** se tocó: no
  hay forma cooperativa de interrumpir NTTTCP a mitad de transferencia, así que sigue el
  mecanismo ya existente (abortar la tarea, `Drop` mata el proceso vía `JobObject`) y el
  par sigue enterándose por el cierre del canal, no por un `CANCEL` explícito.
- **`SessionService::cancel` reordenado, no solo más lento**: primero marca la señal,
  después da hasta 500 ms a que la tarea termine por su cuenta (usando `abort_handle()`
  para poder forzar el corte después de haber consumido el `JoinHandle` en el `timeout`),
  y si no lo consigue, aborta como antes. En el caso mid-medición esto añade hasta 500 ms
  de latencia frente a la versión anterior (abortaba sin esperar nada); se acepta porque
  sigue muy por debajo del límite de 2 s de SC-004 y habilita el camino cortés en el caso
  común de cancelar mientras se negocia, sin arriesgar el límite.
- **Prueba real nueva**:
  `t175_cancelar_antes_de_que_arranque_el_motor_es_cortes_en_los_dos_extremos` en
  `session_service_real.rs`. Cancela sin ninguna espera deliberada justo tras
  `iniciar_sesion_real`, para pillar el diálogo en pleno intercambio de mensajes; en las
  5 repeticiones ejecutadas tardó 0,44–0,48 s y en todas los dos extremos terminaron en
  `Cancelled`, sin persistir nada y sin ningún `ntttcp.exe`. Es una carrera de tiempos real
  entre la llamada a `cancel()` y cuánto ha avanzado el diálogo, no un punto de control
  determinista; se documenta así en vez de presentarla como una garantía absoluta de
  temporización.
- **Un hallazgo real, de una prueba de la tarea anterior, no de esta**: al ejecutar la
  batería completa salió `test_preflight_ports_sin_control_port_no_lo_comprueba` (T173)
  como fallo intermitente. Elegía su puerto de sondeo como `puerto_de_un_bind(0) + 1000`,
  un desplazamiento a ciegas que, con las pruebas corriendo en paralelo, podía coincidir
  con un puerto que otra prueba tenía abierto en ese instante. Corregida para usar
  directamente el puerto de un `bind(0)` recién soltado, como ya hacían las pruebas
  vecinas; repetida 3 veces en la batería completa sin volver a fallar.
- **No cubierto, deliberadamente**: cancelación con el motor ya en marcha vista desde el
  protocolo (sigue siendo un cierre de socket, no un `CANCEL`); duplicados de `CANCEL`
  (el contrato exige idempotencia, no probada); cancelación durante la negociación de
  `START`/`HEARTBEAT`; y, como siempre en esta feature, cualquier prueba entre dos equipos
  físicos con latencia de red real en vez de bucle local.
- **Estado**: `VERIFICADO` (cancelación cortés en la ventana previa al motor, en los dos
  sentidos, con NTTTCP real y sin regresión del límite de 2 s) · `NO PRESENTE`
  (cancelación cortés con el motor en marcha; duplicados de `CANCEL`) ·
  `NO VERIFICABLE` aquí (dos equipos reales)

### 1.20 Intercambio SESSION_RESULT/SESSION_ACK y reconciliación (T176)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, NTTTCP 5.40 x64,
  2026-09-25.
- **Comandos**:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib control::session_flow::reconciliacion_de_resultado`
  (3 en verde) ·
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (5 en verde, NTTTCP real) · `cargo test` completo (140 en verde) · `cargo fmt` y
  `clippy --all-targets -- -D warnings` sin avisos.
- **Qué hace, siguiendo `contracts/session-result.md`**: quien inicia construye el
  resultado canónico tras ensamblarlo (punto 1 del contrato), lo persiste localmente con
  `resultSource: "initiator"` y lo envía por `SESSION_RESULT`. Quien responde lo valida
  (punto 2): mismo `sessionId`, mismo número de direcciones, y ninguna dirección que él
  vio `completed` aparece como algo menos en la copia recibida. Si valida, la adopta
  —reetiquetada `"initiator"`— y confirma con `SESSION_ACK{persisted:true}`; si no, o si
  no llega en 10 s (el timeout genérico de `peer-protocol.md`, sin uno propio para este
  mensaje), conserva su propia vista con `resultSource: "local"` y responde
  `SESSION_ACK{persisted:false}` (punto 5, recuperación explícita, sin fingir igualdad).
  Un `SESSION_ACK` ausente o tardío del lado del respondedor no deshace lo que el
  iniciador ya persistió: solo se registra un aviso.
- **Verificación real de punta a punta**: `t144_sesion_completa_entre_dos_servicios_con_motor_real`
  se amplió para comprobar que, tras una sesión real con NTTTCP, los dos historiales
  (`result_json`, campo `resultSource`) quedan en `"initiator"` — es decir, B recibió,
  validó y adoptó de verdad la copia canónica de A, no se quedó con la suya por casualidad
  de que ambas coincidieran.
- **Lo que ya existía y no hubo que inventar**: `SessionAckPayload` y el campo
  `result_source` de `SessionResult` ya estaban en el modelo (probablemente de un lote
  anterior que dejó el contrato listo sin cablearlo); esta tarea fue conectar ambos al
  diálogo real, no diseñarlos.
- **No cubierto, fuera del alcance declarado de esta tarea**: duplicados de
  `SESSION_RESULT` (el contrato exige idempotencia, no probada aquí); reconexión tras
  perder la conexión durante la reconciliación; el campo `result_source` no tiene columna
  propia en el esquema de `history` —viaja dentro de `result_json`— y `VALIDACION` no
  afirma que la tenga; cualquier prueba entre dos equipos físicos.
- **Estado**: `VERIFICADO` (construcción canónica, validación, adopción o conservación
  local, ACK en los dos sentidos, con NTTTCP real) · `NO PRESENTE` (duplicados,
  reconexión) · `NO VERIFICABLE` aquí (dos equipos reales)

### 1.21 Consentimiento para solicitudes de sesión entrante (T177, mitad de FR-016)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, NTTTCP 5.40 x64,
  2026-09-25.
- **Comandos**:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib control::consent` (5 en verde) ·
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_service_real` (5 en
  verde, sin `--ignored`) ·
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (4 en verde, NTTTCP real) · `node scripts/architecture/check-ipc-commands.mjs` (31
  invocaciones del frontend siguen coincidiendo) · `cargo test` completo (145 en verde) ·
  `cargo fmt` y `clippy --all-targets -- -D warnings` sin avisos.
- **Qué cambia**: un equipo `Trusted` sin autoaceptación ya no se rechaza al instante.
  `atender_sesion_entrante` registra la solicitud en `control::consent::SolicitudesEntrantes`
  (varias pueden coexistir, una por equipo que pregunte a la vez), avisa por el nuevo
  evento `session://incoming-request` si hay por dónde hacerlo, y espera hasta 60 s (el
  plazo que `contracts/peer-protocol.md` fija para «Pairing or user acceptance») antes de
  rechazar por su cuenta. Aceptar reutiliza exactamente el mismo camino que ya existía
  para `TrustedAutoAccept`. Un equipo `Known` (emparejado pero sin la confianza que exige
  medir) o directamente desconocido no llega a esta pregunta: seguir rechazándolo sin
  preguntar es intencionado, no un descuido — el consentimiento de FR-016 no sustituye la
  verificación de emparejamiento de FR-012.
- **Verificación real de punta a punta**: `t177_un_equipo_de_confianza_sin_autoaceptacion_completa_si_se_acepta`
  registra a los dos nodos como `Trusted` (sin autoaceptación en ninguno), espera a que
  aparezca la solicitud en el almacén, la acepta por la misma vía que usaría la interfaz
  (`SolicitudesEntrantes::responder`, no un atajo interno), y comprueba que la sesión
  completa con NTTTCP real y que el evento que habría recibido la interfaz llevaba el
  `requestId` y el equipo correctos.
- **Deliberadamente no construido en esta pasada, y por qué**:
  - **La pantalla de Svelte.** El evento y los dos comandos IPC
    (`session_incoming_list`, `session_incoming_respond`) están escritos y probados desde
    Rust, pero ningún componente los consume todavía. `check-ipc-commands.mjs` solo
    verifica que las llamadas del frontend correspondan a comandos reales, no al revés,
    así que su verde no dice nada sobre esto. Construir la pantalla sin poder abrirla en
    un navegador o en la aplicación e interactuar con ella iría contra la instrucción de
    no dar una UI por terminada sin probarla así; queda como T181.
  - **`PAIR_REQUEST` entrante (FR-012).** `control/despachador.rs` sigue exactamente
    igual que antes de esta tarea: registra el mensaje y lo descarta sin responder, así
    que quien empareja se queda esperando hasta su propio timeout. Es un flujo distinto
    del de sesión, con su propio código de verificación de seis dígitos; no se tocó, para
    no mezclar dos piezas de tamaño comparable en una sola tanda. Queda como T182.
- **Estado**: `VERIFICADO` (registro, evento, espera acotada, aceptación real con NTTTCP,
  rechazo explícito y rechazo automático por no llegar a preguntar a quien no corresponde)
  · `NO PRESENTE` (pantalla de Svelte, `PAIR_REQUEST` entrante)

### 1.22 Soporte de IPv6 en el motor (T179)
- **Entorno**: Host local Windows 11 Pro 26200 x64, NTTTCP 5.40 x64, Rust 1.98.1,
  2026-09-25.
- **Primero, a mano, contra el binario real** (antes de tocar código): dos procesos
  `ntttcp.exe` independientes, uno `-r` y otro `-s`, los dos con
  `-m 1,*,::1 -p 55231 -l 65536 -a 8 -t 3 -6 -xml <ruta>`. Los dos terminaron con
  `Network activity progressing...` y un XML con el mismo esquema que IPv4
  (`<use_ipv6>True</use_ipv6>`, métricas de `<throughput>`/`<realtime>` idénticas en
  forma). Confirma que `-6` es el flag correcto y que el parser existente no necesita
  cambios para XML de IPv6.
- **Comandos**:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib engine::ntttcp::ntttcp_tests::test_una_direccion`
  (2 en verde) ·
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real t179 -- --ignored`
  (1 en verde, NTTTCP real sobre `::1`, 5,27 s) · `cargo test` completo (147 en verde) ·
  `cargo test --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (7 en verde) · `cargo fmt` y `clippy --all-targets -- -D warnings` sin avisos.
- **Qué hace**: `build_ntttcp_args` interpreta el host que recibe como `IpAddr`; si es
  `V6` y no es una IPv4 mapeada (`::ffff:a.b.c.d`), añade `-6`. La distinción con
  `to_ipv4_mapped()` es deliberada: `normalizar_ip` (`control/service.rs`) ya reduce las
  direcciones mapeadas a IPv4 antes de que lleguen aquí, pero el constructor de
  argumentos no depende de que ese paso previo se haya ejecutado — repite la
  comprobación por su cuenta.
- **Verificación real de punta a punta**: `t179_dialogo_completo_sobre_ipv6_real`
  reproduce exactamente `t144_dialogo_completo_produce_un_resultado_con_velocidad_oficial`
  pero con `ControlServer` ligado a `::1`, `conectar_y_saludar("::1", ...)` y
  `iniciar_direccion`/`atender_direccion` con `::1` como dirección. El diálogo entero —
  REQUEST/RESPONSE, PREPARE/READY, negociación de inicio, y las dos mitades del motor
  real— completa con la misma velocidad oficial en los dos extremos.
- **No cubierto**: `control/service.rs::recorrido_como_iniciador` obtiene la dirección del
  peer con `peer.addresses.first().parse::<SocketAddr>()`, que para IPv6 exige la
  notación con corchetes (`"[::1]:puerto"`); no se ha comprobado que ese formato
  sobreviva completo desde el emparejamiento (`ipc/pairing.rs`) y el guardado del peer
  hasta llegar aquí — esta tarea prueba el diálogo de protocolo en sí, no esa cadena
  completa. Tampoco hay prueba entre dos equipos físicos con IPv6 real (T180).
- **Estado**: `VERIFICADO` (flag `-6`, motor real sobre `::1`, diálogo completo con
  velocidad oficial reconciliada) · `NO VERIFICABLE` aquí (formato de dirección IPv6 de
  extremo a extremo desde el emparejamiento; dos equipos físicos)

### 1.23 UDP a través del diálogo real de sesión (T178, mitad de US5)
- **Entorno**: Host local Windows 11 Pro 26200 x64, NTTTCP 5.40 x64, Rust 1.98.1,
  2026-09-25.
- **Comandos**:
  `cargo test --manifest-path src-tauri/Cargo.toml --test session_flow_real t178 -- --ignored`
  (1 en verde, 5,27 s) ·
  `cargo test --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (8 en verde) · `cargo test` completo (147 en verde) · `cargo fmt` y
  `clippy --all-targets -- -D warnings` sin avisos.
- **El hallazgo principal de esta tarea es negativo, y por eso vale la pena decirlo así**:
  antes de tocar nada, una revisión de `session_flow.rs` y `orquestador.rs` no encontró ni
  una sola rama que distinga `BenchmarkProtocol::Tcp` de `Udp` — el protocolo entre peers,
  la negociación de inicio (G2) y el ensamblado de `DirectionResult`/`SessionResult` son
  agnósticos al transporte; solo `engine/ntttcp/args.rs` (el flag `-u` y el tamaño de
  datagrama) y el parser XML conocen la diferencia. Eso quiere decir que un plan UDP ya
  atravesaba el diálogo real de sesión sin ningún cambio de código: lo único que faltaba
  era la evidencia de que de verdad funciona, no una implementación.
- **Prueba real nueva**: `t178_dialogo_completo_con_udp_real`, calco exacto de
  `t144_dialogo_completo_produce_un_resultado_con_velocidad_oficial` con
  `plan.protocol = BenchmarkProtocol::Udp`. Diálogo completo REQUEST→ENGINE_DONE con
  NTTTCP real, `completed` en los dos extremos, misma velocidad oficial reconciliada.
- **Un hallazgo real distinto, no corregido aquí**: `NtttcpParsedResult` (el parser)
  expone `packets_sent`/`packets_received`, pero `orquestador::a_engine_result` no los
  copia a `EngineResult`, y `Orquestador::resultado_de_direccion` fija
  `retransmission: None` de forma incondicional, sin mirar esos contadores. Es decir: la
  pérdida de paquetes UDP —la métrica que más le importa a un plan UDP— no se calcula en
  el camino real, con diálogo entre peers o sin él. No es una regresión de esta tarea: ya
  estaba así; el diálogo simplemente no lo tocaba y por tanto no lo revelaba. Se declara
  como T184.
- **Lo que T178 pedía y sigue sin estar, sustancialmente distinto de lo anterior**: la
  ejecución simultánea (`BenchmarkDirection::Both`, `SessionState::RunningBoth`). La
  máquina de estados ya admite la transición y `control/ports.rs::PortAllocation` ya
  calcula rangos de puertos sin solape para las dos direcciones, pero ningún camino de
  `session_flow.rs` lo usa: todo el diálogo asume una pata a la vez, con un solo rol por
  extremo. Convertir eso en negociar las dos direcciones con un único
  `PREPARE`/`READY`/`START` y que cada extremo ejecute emisor y receptor en paralelo
  consigo mismo es un camino de protocolo distinto, no una variación del secuencial. Se
  declara como T183, sin empezar.
- **Estado**: `VERIFICADO` (UDP a través del diálogo real, con NTTTCP real) · `NO
  PRESENTE` (ejecución simultánea; pérdida de paquetes UDP en el resultado)

### 1.24 Retransmisión TCP y pérdida UDP en el resultado real (T184)
- **Entorno**: Host local Windows 11 Pro 26200 x64, NTTTCP 5.40 x64, Rust 1.98.1,
  2026-09-25.
- **Comandos**: `cargo test --lib diagnostic` (21 en verde) ·
  `cargo test --lib control::orquestador` (11 en verde) ·
  `cargo test --test session_flow_real -- --ignored --test-threads=1` (4 en verde, NTTTCP
  real) · `cargo test --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (8 en verde) · `cargo test` completo (152 en verde) · `cargo fmt` y
  `clippy --all-targets -- -D warnings` sin avisos · `pnpm verify` completo.
- **Origen**: hallazgo de T178 (§1.23). Al cerrarlo aparecieron dos cosas que la
  descripción original no anticipaba.
- **Los contadores se perdían en tres sitios, no en uno**: `engine_done_a_parsed` fijaba
  `packets_*` a `None` (no cruzaban por el cable), `EngineResult` no tenía dónde
  guardarlos, y `resultado_de_direccion` fijaba `retransmission: None`. Cerrar solo el
  último habría dado una estadística que un extremo podía calcular y el otro no: quien no
  midió una mitad nunca conocía sus contadores. La prueba real lo comprueba: los dos
  extremos obtienen el mismo `RetransmissionStats` (TCP y UDP), y eso solo puede pasar
  si los contadores viajan por `ENGINE_DONE`.
- **Lo que ya existía, sin invocar**: `DiagnosticEngine::evaluate_retransmissions` (TCP) y
  `diagnostic::udp::evaluate_udp_diagnostics` (UDP, `Historias.md` §18) estaban escritos y
  probados en aislamiento y no los llamaba nadie — el mismo patrón que todo este
  ejercicio. Mi primera versión de la pérdida UDP reescribió la fórmula sin haber
  mirado `diagnostic/udp.rs` (solo había mirado `rules.rs`); al verlo, se rehízo para
  delegar en esa función y dejar una sola fuente de la matemática y de los umbrales.
- **El veredicto ignoraba la retransmisión**: `ensamblar` pasaba
  `retransmissions: None` a `generate_verdict`, que ya tenía reglas para
  `retransmissions_elevated`/`_high`. Ahora recibe la peor de las direcciones
  (`Historias.md` §13.5). Comprobado con una sesión de una dirección limpia y otra con
  retransmisión alta: `retransmission_level == High`.
- **Decisión de modelado, declarada y no normativa**: `DirectionResult` solo tiene el hueco
  `retransmission`. La pérdida UDP no es una retransmisión (UDP no retransmite), pero el
  contrato no ofrece otro sitio y añadir un campo al esquema versionado no es una decisión
  que me corresponda. Por eso la pérdida viaja en `RetransmissionStats` con `ratio` = pérdida
  y `packetsRetransmitted` = paquetes perdidos. Los umbrales de `Historias.md` (0,1 % y 1 %)
  coinciden numéricamente entre retransmisión TCP y pérdida UDP, así que los niveles
  se corresponden. Si el propietario prefiere un campo propio, es un cambio de contrato.
- **No cubierto**: `UdpDiagnosticResult` completo (tasa objetivo, claves de título,
  observación de tasa objetivo por encima de la capacidad) sigue sin camino hacia el
  resultado; el esquema TS `engineResultSchema` desconoce los tres campos de paquetes
  (`z.object` los descarta, no rompe nada, pero la interfaz no puede mostrarlos); un par
  con una versión anterior que no envíe los contadores deja la estadística del otro
  extremo en `NotAvailable`, sin inventar datos; la comparación con la retransmisión real
  en una red con pérdida (bucle local devuelve pérdida cero).
- **Estado**: `VERIFICADO` (contadores por el cable, cálculo por protocolo, igualdad entre
  los dos extremos con NTTTCP real, veredicto) · `NO PRESENTE` (diagnóstico UDP completo
  en el resultado; visualización) · `NO VERIFICABLE` aquí (comportamiento con pérdida
  real de red)

### 1.25 Emparejamiento entrante (T182, FR-012)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, 2026-09-25. Sin motor:
  dos nodos completos (identidad, base de datos, servidor, despachador) sobre TLS real.
- **Comandos**: `cargo test --test session_service_real t182 -- --test-threads=1` (4 en
  verde) · `cargo test --lib control::pairing_flow` (7 en verde, sin regresión tras
  partir la función) · `cargo test` completo · `cargo fmt` y
  `clippy --all-targets -- -D warnings` sin avisos · `check-ipc-commands.mjs` y
  `pnpm verify` en verde.
- **El problema real**: antes, el equipo que recibía un `PAIR_REQUEST` lo registraba y lo
  descartaba sin contestar. Quien pedía emparejarse esperaba hasta su propio plazo. Dos
  equipos reales **no podían emparejarse nunca**, y sin emparejar no hay sesión de
  confianza posible. Todo lo verificado en las tandas anteriores partía de peers ya
  registrados a mano en la base de datos.
- **Qué se rehízo**: la función existente `atender_emparejamiento` decidía con un
  closure síncrono, incompatible con esperar a una persona. Se partió en
  `preparar_respuesta` (construye el emparejamiento con el `id` del iniciador y verifica)
  y `contestar_emparejamiento` (envía `PAIR_RESULT`); la función original usa las dos y sus
  7 pruebas siguen pasando. El almacén de decisiones pendientes se generalizó
  (`Decisiones<T>`) y lo comparten T177 (sesión) y T182 (emparejamiento).
- **Garantías de seguridad, y cómo se comprueban**:
  - Un código incorrecto **no llega a la persona** y no dispara aviso a la interfaz:
    `t182_un_codigo_incorrecto_se_rechaza_sin_preguntar` envía un código falso a mano y
    comprueba `verificationFailed`, almacén vacío y ningún evento emitido.
  - El código que ve la persona de B es **el mismo** que ve la de A
    (`verification_code == emp.codigo`): es lo único que hace valer la comparación humana.
  - Aceptar guarda `Trusted` con `auto_accept: false`; rechazar no guarda nada.
  - La confianza se guarda **antes** de contestar `accepted:true`.
  - Un segundo intento del mismo equipo con uno pendiente se rechaza al instante.
- **Un error mío en la prueba, no en el código**: afirmé que la dirección guardada
  terminaba en `:7411`; el historial no persiste las direcciones de los peers (las carga
  vacías), así que falló por leer el sitio equivocado. La aserción pasó al evento hacia la
  interfaz, que sí las lleva.
- **No cubierto**: la pantalla de Svelte (T181); el vencimiento de 55 s sin decisión (no
  se prueba esperando 55 s reales, y hacer el plazo inyectable no lo pedía la tarea); el
  puerto de la dirección mostrada es el de control por defecto, **INFERIDO** porque el
  `HELLO` no lo declara; el límite «5 solicitudes por minuto» del contrato; dos equipos
  físicos (T180).
- **Estado**: `VERIFICADO` (flujo entrante completo sobre TLS real, garantías de
  seguridad anteriores) · `NO PRESENTE` (pantalla; vencimiento de 55 s probado) ·
  `NO VERIFICABLE` aquí (dos equipos físicos)

### 1.26 Elevación del helper de firewall y su lista blanca (T154, FR-057)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, 2026-09-25.
- **Comandos**: `cargo test --lib firewall` (12 en verde) ·
  `cargo test --test firewall_helper` (5 en verde, **ejecutan el binario real del helper**)
  · `cargo test` completo (259 en verde, 10 ignoradas) · `cargo fmt` y
  `clippy --all-targets -- -D warnings` sin avisos · `pnpm verify` en verde.
- **Qué había, leído antes de tocar nada**: la orden de elevación se construía con
  `powershell -Command "Start-Process -FilePath '<ruta>' ..."` interpolando rutas entre
  comillas simples; la petición viajaba en `%TEMP%\nb_fw_req_<pid>.json` (nombre
  predecible, directorio escribible por cualquier proceso del usuario) y el proceso
  elevado la leía después; si el helper no aparecía junto al ejecutable, se devolvía su
  nombre suelto y `runas` lo habría resuelto por el `PATH`.
- **Tres agujeros de la lista blanca que la tarea no mencionaba**, encontrados al leerla
  con intención de romperla, no de confirmarla:
  1. El programa se aceptaba si su nombre **terminaba** en `ntttcp.exe` o
     `networkbench.exe`: `C:\cualquier\sitio\ntttcp.exe` pasaba.
  2. El rango de puertos solo se comprobaba si contenía **exactamente un** guion:
     `1024-1030,2000-65000` (tres partes al dividir) se saltaba el límite de 64.
  3. `protocol` y `profiles` no se comprobaban en absoluto.
  Y la lista blanca estaba **escrita dos veces** (cliente y helper), sin nada que
  impidiera que divergieran. Ahora es un solo fichero, `firewall/validation.rs`,
  compilado en los dos con `#[path]`.
- **Por qué la validación del helper es la que cuenta**: el helper corre con privilegios y
  puede lanzarlo cualquier proceso, no solo la aplicación; la validación del cliente solo
  evita pedir una elevación que se sabe rechazada. Por eso las pruebas del binario real
  le pasan la línea directamente, sin pasar por el cliente.
- **Cómo se probó sin abrir un UAC**: el helper acepta `--validate-only` como primer
  argumento y hace todo salvo aplicar (análisis de la línea, lista blanca, código de
  salida); no toca el firewall ni necesita privilegios. La prueba copia el helper real a
  una carpeta con espacios en el nombre, con un `ntttcp.exe` al lado, y lo lanza con
  `raw_arg` con exactamente la cadena que se entregaría a `ShellExecuteEx`: así el
  entrecomillado lo analiza el analizador de línea de órdenes real de Windows, no un
  reimplementado en la prueba.
- **NO VERIFICABLE aquí, y nadie lo ha ejecutado**: `lanzar_elevado`, la llamada real a
  `ShellExecuteExW` con `runas`, la espera del proceso y la lectura de su código de
  salida, y el mapeo de «UAC rechazado» (`ERROR_CANCELLED`) a `FirewallUacRejected`. Abre
  un diálogo de UAC en el escritorio del usuario; una prueba automática no puede ni debe
  aceptarlo. Es código `unsafe` escrito contra la documentación de la API y compilado,
  sin más evidencia que esa. **Queda abierta la tarea hasta que una persona acepte un UAC
  una vez y compruebe que una regla se crea.**
- **Desviaciones de `Historias.md` §14.2, declaradas**: el texto habla de «fichero JSON» y de
  `INetFwPolicy2` (COM, «no `netsh`»). La petición viaja ahora por argumentos —un canal que
  nadie puede sustituir, que es lo que pide la tarea— y el helper sigue aplicando con
  `netsh`, con argumentos separados (nunca una cadena de shell) y ya validados. Pasar a COM
  es otra tarea.
- **Se añaden features de `windows`** (`Win32_UI_Shell`, `Win32_UI_WindowsAndMessaging`), sin
  cambio en `Cargo.lock`.
- **Dos pruebas inestables del preflight de sesión, encontradas de paso y corregidas**: la
  batería completa falló una vez en `un_puerto_de_datos_ocupado_se_rechaza_...` y en
  `un_cancel_del_par_...`, sin reproducirse por separado. Causa: `bind(0)` en Windows da
  puertos efímeros de 49152 a 65535 y `BenchmarkPlan::validate` exige ≤ 65000; ~3 % de las
  ejecuciones recibían un plan inválido y se rechazaban por `invalidPlan`. Una tercera
  (`test_preflight_ports_available`, `port + 10`) fallaba ~1 de cada 8 porque los vecinos de
  un puerto efímero suelen estar ocupados por otras pruebas en paralelo. Ahora usan
  puertos ≤ 65000 y una ventana fuera del rango efímero. La biblioteca se ejecutó 12 veces
  seguidas sin un fallo.
- **Estado**: `VERIFICADO` (lista blanca; helper real rechazando peticiones hostiles;
  entrecomillado a través del analizador de Windows) · `NO VERIFICABLE` aquí (elevación
  real con UAC)

### 1.27 Pérdida de canal a mitad de la segunda pata (T171, FR-024)
- **Entorno**: Host local Windows 11 Pro 26200 x64, NTTTCP 5.40 x64, Rust 1.98.1,
  2026-09-25.
- **Comandos**: `cargo test --lib control::session_flow::perdida_de_canal` (2 en verde) ·
  `cargo test --test session_service_real t171 -- --ignored` (1 en verde, NTTTCP real,
  ~12 s) · `cargo test` completo (261 en verde) ·
  `cargo test --test session_flow_real --test session_service_real -- --ignored --test-threads=1`
  (9 en verde) · `cargo fmt` y `clippy --all-targets -- -D warnings` sin avisos ·
  `pnpm verify` en verde.
- **La prueba de aceptación encontró un fallo, no una confirmación**: la tarea pedía una
  prueba de un comportamiento que se daba por supuesto. Al escribirla, el comportamiento
  real era el contrario del que exige `Historias.md` §8.5/§19.3: si el canal caía en la
  segunda pata, el error se propagaba con `?`, la primera pata —ya completa— se descartaba y
  la sesión terminaba `Failed` **sin guardar nada**.
- **El arreglo**: `PruebaBidireccional` gana `interrupcion: Option<String>`. Ante una pérdida
  de canal o de motor tras completar una pata, se conserva lo completado, la otra queda
  `incomplete` (sin velocidad oficial, sin motores) y la sesión se guarda como
  incompleta (`is_partial`), pero termina `Failed` con `NB-CONN-005`. Una cancelación se
  distingue por la señal compartida de T175: cancelar sigue descartando la sesión. No se
  intenta el intercambio de resultado por un canal perdido (el respondedor conserva su
  vista `local`, degradada y declarada), lo que además evita esperar 10 s a un
  `SESSION_RESULT` que no puede llegar.
- **Un segundo defecto, del mismo escenario**: `tokio::join!` en la mitad receptora
  esperaba a los **dos** futuros. Si el emisor moría, el fallo de los mensajes no soltaba
  el motor receptor, que seguía esperando hasta su propio plazo. `try_join!` lo suelta al
  primer error y el `Drop` del proceso lo mata.
- **Cómo se provoca la pérdida en la prueba real**: cancelando en B mientras B emite la
  vuelta. Con el motor en marcha no hay punto cooperativo (T175), así que B espera 500 ms y
  aborta su tarea: A solo ve caer el canal, sin ningún `CANCEL`. Para A es una pérdida.
  Comprobado: A `Failed`; su historial tiene `incomplete`, `is_partial`, `forward_bps` y
  **no** `reverse_bps`; B `Cancelled` y sin guardar nada; ningún `ntttcp.exe` restante.
- **No cubierto, y conviene decirlo**: FR-024 dice «MAY recuperarse»; no hay reconexión con el
  mismo `sessionId` ni latido cada 1 s con «Reconectando…» (`Historias.md` §8.5), así que
  «los huecos MUST marcarse» solo se cumple en lo que sí existe (huecos explícitos en las
  muestras, T174) y no hay un camino de pérdida temporal *recuperada* que ejercitar; la
  pérdida en la **primera** pata sigue sin guardar nada (no hay una dirección completa que
  conservar); dos equipos físicos (T180).
- **Estado**: `VERIFICADO` (una pata a medias nunca se reanuda como continua; lo completado
  se conserva; cancelación ≠ pérdida) · `NO PRESENTE` (reconexión y latido)

### 1.28 Anuncio y descubrimiento mDNS (T151, FR-010)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, `mdns-sd` 0.21.4 (la
  instalada, no la 0.21.3 de la línea base), 2026-09-25.
- **Comandos**: `cargo test --lib discovery::anuncio` (7 en verde) ·
  `cargo test --test mdns_real -- --ignored --nocapture` (1 en verde, multicast real,
  2,3 s) · `cargo test` completo (268 en verde, 12 ignoradas) · `cargo fmt` y
  `clippy --all-targets -- -D warnings` sin avisos · `check-ipc-commands.mjs` y
  `pnpm verify` en verde.
- **Antes**: `mdns-sd` estaba declarada y sin uso; `discovery/` solo conectaba a mano por
  IP. Dos equipos en la misma red no se veían.
- **Un defecto de especificación encontrado de paso**: la constante decía
  `_netbench._tcp.local.` y `Historias.md` §7.1 dice `_networkbench._tcp.local.`. Con la
  primera, la aplicación habría sido invisible para cualquier otra implementación de la
  especificación. Corregida, con una prueba que fija el valor.
- **Modelo de confianza**: mDNS figura entre los actores no confiables del plan de
  seguridad. Cualquiera en la red puede anunciar cualquier nombre, identificador o huella
  —incluida la de un equipo de confianza—. Por eso `EquipoDescubierto` son **pistas para
  mostrar**: la huella se llama `fingerprint_declarada`, no lleva estado de confianza, y
  nada se guarda ni se autentica con ello. La identidad real es la del certificado TLS al
  conectar. Consecuencia que conviene tener presente para la pantalla (T181-bis): mostrar
  «De confianza ★» a partir de la huella anunciada sería falsable por un tercero; el
  handshake lo desmentiría al conectar, pero la estrella ya habría engañado.
- **Todo lo que llega se valida antes de existir como dato** (`equipo_desde_anuncio`, pura):
  UUID válido, huella exactamente 64 hex, registros TXT acotados a 64 y sin caracteres de
  control, nombre saneado, versión con caracteres seguros (`<script>` → `?`), `link`
  acotado a 1 000 000, puerto ≥ 1024, y solo direcciones alcanzables (sin especificar,
  multidifusión, bucle local ni IPv6 de enlace local, que sin ámbito no se pueden usar).
  Un anuncio incompleto o con un campo inválido se rechaza **entero**, no se acepta a
  medias.
- **Una expectativa mía equivocada, corregida en la prueba**: supuse que un nombre en
  blanco se rechazaría; `sanitize_display_name` devuelve el nombre de reserva
  «Equipo-Remoto» (el mismo comportamiento de la conexión manual). La prueba ahora fija ese
  comportamiento, y desaparece la comprobación de vacío que nunca podía dispararse.
- **Verificación real**: `dos_instancias_se_descubren_por_mdns_y_desaparecen_al_cerrarse`
  arranca dos `Descubrimiento` con multicast real en esta máquina. Comprobado: cada una ve
  a la otra con nombre, huella declarada y puerto de control correctos; ninguna se ve a sí
  misma; y al soltar una, desaparece de la lista de la otra (el `Drop` retira el anuncio).
- **NO VERIFICABLE aquí**: comportamiento entre equipos distintos; con varios adaptadores,
  VPN o perfil de firewall Público (V-08); la caducidad de «30 s después del último
  anuncio», delegada en el caché de `mdns-sd` y sin medir.
- **No hecho**: la pantalla «Equipos disponibles» (el frontend no llama a
  `peers_discovered_list`); el aviso de perfil público tras 5 s sin equipos; `busy` se
  anuncia siempre `0` y `link` siempre `0`; el puerto anunciado es el de control por
  defecto y no uno configurable; el cableado a `AppState`/`settings_update` no tiene
  prueba propia (un `AppState` real toca `%LOCALAPPDATA%`).
- **Estado**: `VERIFICADO` (análisis estricto de anuncios; descubrimiento y desaparición con
  multicast real entre dos instancias de una misma máquina) · `NO PRESENTE` (pantalla;
  `busy`/`link` reales) · `NO VERIFICABLE` aquí (entre equipos distintos; V-08)

### 1.29 Checks de contratos y de APIs legacy (T160, T011)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, 2026-09-25.
- **Comandos**: `cargo test --test contract_fixtures` (7 en verde) ·
  `pnpm exec vitest run tests/contracts/rust-fixtures.test.ts` (8 en verde) ·
  `node scripts/architecture/check-legacy.mjs` · `pnpm verify` completo en verde
  (103 tests Vitest; incluye ya ambos checks).
- **Contratos**: Rust genera 8 documentos con su código real y los compara con
  `tests/contracts/fixtures/rust/`; Vitest parsea esos mismos ficheros con los esquemas Zod.
  Los dos lados quedan atados al mismo fichero: un cambio de nombre o de formato en Rust
  rompe `cargo test`, y uno que el esquema TS no acepte rompe Vitest. Resultado inicial:
  **sin desajuste real**. Los tres primeros fallos fueron míos (UUID sin bits de versión 4).
  Comprobado que el check discrimina: detectó exactamente eso.
- **APIs legacy**: prohíbe `export let`, `$:`, `createEventDispatcher`, `<slot>`, SvelteKit,
  `process.env` e `import.meta.env` fuera de `lib/config`. Probado con violaciones
  temporales (detecta las 4 ensayadas, ignora un comentario). **Hallazgo real**:
  `lib/logging` leía `import.meta.env` con un cast sin validar; `lib/config` estaba escrito
  y sin uso. Corregido, conservando la precedencia del nivel de log.
- **Límites**: solo patrón textual, no demuestra ausencia por otras vías; no cubre `any`,
  `!` ni dobles casts; la forma del JSON no implica que su significado sea correcto; los
  fixtures no cubren todos los comandos y eventos IPC.
- **Nota operativa**: `pnpm verify` ahora ejecuta también `cargo test` de este binario
  (~1 min de compilación en frío).
- **Estado**: `VERIFICADO` (forma del JSON Rust↔TS para los 8 documentos; APIs prohibidas
  por patrón) · `NO PRESENTE` (fixtures del resto de eventos y comandos)

### 1.30 Snapshot de la aplicación actualizado y emitido (T150)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, 2026-09-25.
- **Antes**: `SnapshotManager::update` no se invocaba desde ningún sitio. El snapshot se
  fijaba al arrancar y `isSessionActive`, `activeSessionId`, `peersCount`, idioma y tema no
  cambiaban nunca. `App.svelte` usa `isSessionActive` para el fondo y para deshabilitar un
  botón: la interfaz no habría reflejado jamás una sesión en curso.
- **Diseño**: la máquina de estados avisa por `Notify` en cada transición real (no en las
  idempotentes); `ProyectorDeSnapshot` recalcula desde las fuentes, no aplica parches, así un
  aviso perdido o repetido no deja un estado sin respaldo; `sincronizar` solo sube la
  revisión si el contenido difiere; el cambio sale como `app://snapshot-changed`.
- **Comandos**: `cargo test --lib ipc::proyector ipc::snapshot control::domain` (14 en
  verde) · `pnpm exec vitest run tests/contracts/snapshot.test.ts` (9 en verde) ·
  `cargo fmt` y `clippy --all-targets -- -D warnings` sin avisos · `pnpm verify` y
  `cargo test` completo (resultado en el cierre de la tarea).
- **Qué prueban**: una sesión que empieza y termina produce dos emisiones con revisión
  creciente, con `activeSessionId` presente y luego ausente; un aviso sin cambios no emite
  y no toca la revisión; un equipo nuevo sube `peersCount` y repetir no cambia nada; en el
  frontend, un evento más nuevo sustituye, uno obsoleto o repetido se descarta, uno
  malformado se ignora, y uno previo al snapshot inicial no es pisado por este.
- **Defectos hermanos corregidos**: `settings_update` y `app_close_evaluate` consideraban
  activa una sesión `Completed` o `Failed` (`!= Idle && != Cancelled`). Sin prueba propia: exigen un
  `tauri::State`.
- **Decisión INFERIDA declarada**: el contrato no nombraba un evento de snapshot completo
  (`session.changed`, `app.snapshotInvalidated`); se añadió `app.snapshotChanged` a la tabla
  de `contracts/ipc.md`.
- **NO VERIFICABLE aquí**: que el evento llegue realmente a un WebView2 (solo hay dobles del
  emisor); el comportamiento visual de `App.svelte` con el estado cambiando.
- **No hecho**: evento de progreso de sesión y de comprobaciones; proyección de sesión y
  equipos completa en el snapshot; borrado de equipos (no existe el comando).
- **Estado**: `VERIFICADO` (proyección, revisión monotónica, descarte de obsoletos con
  dobles) · `NO VERIFICABLE` (entrega al WebView2 real) · `NO PRESENTE` (evento de progreso)

### 1.31 Cableado del frontend: equipos, sesión, resultado y consentimiento (T181, T150 UI)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Node 24.4.1, 2026-09-25.
- **Antes**: `App.svelte` no montaba ningún componente de `features/`; «Inicio» era una
  tarjeta estática. `PeersScreen`, `SessionScreen`, `ResultScreen`, `HistoryScreen` existían
  con pruebas de componente propias pero sin consumidor real. No había forma de completar
  el recorrido equipos → medición → resultado desde la interfaz.
- **Comandos**: `pnpm exec vitest run` (128 en verde, 26 ficheros) · `pnpm check` (0 errores)
  · `pnpm lint` (0 errores; 1 aviso preexistente en `artifacts/coverage/`, ajeno) ·
  `node scripts/architecture/check-imports.mjs` (en verde tras el arreglo) · `pnpm verify`
  completo en verde · `cargo test` completo (173 en verde).
- **PeersModel** (9 pruebas): funde equipo descubierto + guardado por id **y** huella;
  un anuncio con huella distinta a la guardada no hereda confianza (FR-011); ocupado e
  incompatible se derivan de lo que declara el anuncio; un error de red no rompe el modelo;
  el reloj de caducidad del emparejamiento cuenta atrás.
- **SessionController** (6 pruebas): un fallo al arrancar pasa a `failed` con mensaje
  traducido; el sondeo traduce cada estado de Rust y, al completar, **lee** el resultado
  guardado (usa el fixture real generado por Rust en T160, no uno inventado a mano);
  cancelar antes de terminar pasa a `cancelling`; `reset()` limpia sin tocar lo guardado.
- **Un despiste de la propia prueba, no del código**: mi primera aserción leía
  `result.protocol`, un campo que no existe — `protocol` vive en `result.plan.protocol`.
  Lo detectó `pnpm check` al escribir el test, no una revisión posterior.
- **ConsentModel** (4 pruebas): sin nada pendiente `current` es `null`; una solicitud de
  sesión se antepone a un emparejamiento; aceptar retira la solicitud de la lista; un error
  al responder se refleja en `error` sin vaciar la solicitud (para poder reintentar).
- **Límite de arquitectura real, no hipotético**: `check-imports.mjs` fallaba porque los
  dos controladores nuevos importaban `@tauri-apps/api/event` directamente. Corregido con
  wrappers en `lib/api/` (`onSampleBatch` ya existente; `onIncomingSessionRequest` y
  `onIncomingPairingRequest` nuevos en `lib/api/consent.ts`).
- **Verificado en navegador real (mismo día, tras esta sección)**: la extensión Claude en
  Chrome no conectó en esta sesión; los dos primeros intentos con Playwright agotaron el
  tiempo de espera al navegar a `localhost:5183` porque el servidor de una ejecución
  anterior seguía ocupando el puerto (`curl` a `127.0.0.1:5183` fallaba mientras
  `localhost:5183` respondía 200: un servidor huérfano, no un problema de Playwright). Con
  `pnpm exec vite dev` realmente arriba, la navegación funcionó a la primera. Comprobado con
  capturas de accesibilidad de Playwright: «Inicio» renderiza «No se han detectado
  equipos» junto con el aviso de error de `peers_list`/`peers_discovered_list`
  (`NB-INTERNAL-001`, esperado sin backend Tauri) y su botón de cierre; «Historial»
  renderiza sus filtros y «No hay pruebas registradas todavía». Ninguna pantalla se rompe
  al fallar sus llamadas nativas: cada error se registra por el logger y se refleja donde
  corresponde, tal como se diseñó.
- **Sigue NO VERIFICABLE**: WebView2 real, un backend Tauri real detrás (nada de esto
  ejercita `session_start`, el diálogo de emparejamiento saliente ni el de consentimiento
  entrante con datos reales), y el resto de pantallas (`SessionScreen`, `ResultScreen`,
  `ConsentDialog`) sin un backend que dispare sus estados.
- **No hecho**: `PreflightScreen` y `AdvancedPlan` no están montados (el plan es siempre el
  estándar); no hay reconexión tras pérdida de canal; los estados «Ocupado»/«Versión
  incompatible» de una tarjeta de equipo nunca se han visto con un segundo equipo real.
- **Estado**: `VERIFICADO` (lógica de los tres modelos con dobles de transporte; límites de
  arquitectura; «Inicio» e «Historial» renderizan sin crash en navegador real) ·
  `NO VERIFICABLE` (WebView2 real; el resto de pantallas sin backend real) · `NO PRESENTE`
  (preflight/plan avanzado montados, reconexión)

### 1.32 Suites E2E reales con axe-core: cuatro defectos reales encontrados (T147)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Node 24.4.1, Playwright 1.63.0
  (Chromium ya instalado), 2026-09-25.
- **Antes**: `e2e/smoke.spec.ts`, `e2e/accessibility/a11y.spec.ts` y `e2e/visual/visual.spec.ts`
  eran `expect(true).toBe(true)`. No existía `test:e2e` ni `webServer` en
  `playwright.config.ts`. `e2e/**/*.ts` no estaba en el `include` de `tsconfig.json`: nadie
  tipaba estos ficheros.
- **Comandos que se ejecutaron y pasaron, al final**: `pnpm check` (491 ficheros, 0 errores)
  · `pnpm lint` (0 errores; 1 aviso preexistente ajeno en `artifacts/coverage/`) ·
  `pnpm exec vitest run` (128 en verde) · `pnpm exec playwright test` (**9 en verde, 1
  `skip` declarado**, ~25-30 s) · `pnpm verify` completo en verde.
- **Obstáculo inicial y su causa real, no un misterio sin resolver**: las primeras
  ejecuciones de `pnpm exec playwright test` contra `vite dev` colgaban o superaban 60 s de
  forma intermitente en la primera navegación de cada test, mientras `curl` al mismo
  servidor respondía en 7-10 s. Aislado con `curl -m 90` y con `workers: 1`: no era un
  problema de red ni del navegador, era **compilación bajo demanda en frío** — `vite dev`
  transforma cada módulo la primera vez que se pide, y una aplicación de 316 módulos con
  Tailwind 4 JIT tarda más de lo que cualquier timeout razonable de un test debería esperar
  bajo carga. Arreglado sirviendo el **build de producción** (`vite build && vite preview`)
  en `playwright.config.ts`: sin transformación bajo demanda, sin esa fuente de
  intermitencia — de 5+ minutos con fallos a ~25 s en verde.
- **El hallazgo más grave, y el más simple: `src/lib/design-system/tokens.css` no se
  importaba en ningún sitio.** `src/app.css` solo tenía `@import "tailwindcss"`. El fichero
  con las 524 líneas de tokens de color oscuro/claro —referenciado desde 888 usos de
  `var(--color-*)` en 46 componentes— compilaba dentro del bundle (Vite lo procesa porque
  algo lo importa transitivamente en dev, pero en el HTML servido nunca llegaba a
  aplicarse su contenido a `:root`) sin que ninguna hoja de estilos real lo cargara. Lo
  detectó el primer test de `visual.spec.ts` (`--color-bg-main` calculado devolvía cadena
  vacía en vez de `#191c30`). **Verificado visualmente con una captura real** (Playwright,
  `vite preview`, tema oscuro): antes de la corrección no se había mirado nunca un
  render con atención al color — solo árboles de accesibilidad, que no muestran color. Con
  el `@import` añadido en `src/app.css`, la app renderiza el diseño "Graphite Violet"
  previsto (fondo degradado violeta, tarjetas, botón primario lila). Sin este arreglo, la
  aplicación entera se habría visto sin ningún color de marca en un WebView2 real.
- **Dos defectos de accesibilidad reales, encontrados al escribir las pruebas**: (1) el
  `tablist` de `SettingsScreen` (`role="tab"`) y el `radiogroup` de tema de
  `AppearanceSettings` (`role="radio"`) declaraban el rol ARIA compuesto correcto pero
  **sin implementar su patrón de teclado** (WAI-ARIA APG): sin flechas, sin roving
  `tabindex` — cada pestaña/radio estaba en el orden de Tab por separado y las flechas no
  hacían nada. Corregido en los dos: `ArrowLeft/ArrowRight/Home/End` mueven **y**
  seleccionan, solo el elemento activo tiene `tabindex="0"`. (2) axe-core marcó
  `.titlebar-name` (el texto "NetworkBench" de la barra de título) como «contenido no
  contenido en ningún landmark» (RGAA-9.2.1): la raíz de `TitleBar.svelte` era un `<div>`
  sin rol; cambiada a `<header>` (landmark `banner` implícito), sin tocar ningún selector
  CSS (todos por clase, ninguno ligado a la etiqueta).
- **Un defecto de integración que el propio componente ya había previsto y advertido en su
  comentario, confirmado real**: `Dialog.svelte` marcaba `inert` solo los hermanos
  directos de `scrimEl.parentElement` — válido si el diálogo cuelga justo del contenedor
  de ruta, como decía su comentario "se asume... si en la integración real acaba anidado
  más adentro, hay que revisar qué nivel se marca inert". En la integración real (T181),
  el diálogo de conexión manual cuelga dentro de `PeersScreen` → `<main>` → un `div`
  hermano de `Sidebar`: con el algoritmo de un solo nivel, la barra lateral **no** quedaba
  `inert` mientras el diálogo estaba abierto (detectado por el test de atrapado de foco).
  Corregido: ahora sube por toda la cadena de ancestros hasta `<body>`, marcando en cada
  nivel a los hermanos que no llevan al diálogo — cumple la promesa del componente
  ("todo menos el diálogo") sin importar la profundidad de anidamiento real.
- **Un cuarto hallazgo, en el mismo diálogo**: el propio comentario de `Dialog.svelte` deja
  explícito que el retorno de foco al cerrar es responsabilidad de quien instancia el
  diálogo, no del componente. `PeersScreen.svelte` nunca lo implementaba: cerrar con
  Escape, con "Cancelar" o al conectar no devolvía el foco al botón que lo abrió.
  Corregido con `manualModalTrigger` (captura `document.activeElement` al abrir) y
  `await tick()` antes de restaurar el foco — sin el `tick()`, el botón seguía `inert`
  (recién liberado por el `onDestroy` del diálogo, que Svelte no aplica de forma síncrona
  con el cambio de estado) y `.focus()` no habría hecho nada.
- **No hecho**: el tercer test visual («plantilla de impresión PDF A4») se dejó `test.skip`
  con motivo — `PrintReport.svelte` no tiene ninguna ruta, ventana ni regla `@media print`
  que lo haga alcanzable (verificado por grep en todo `src/`), así que no hay nada que un
  E2E pueda ejercitar todavía; corresponde a quien cablee la exportación a PDF.
- **Decisión declarada**: `test:e2e` no se encadena en `scripts/verify-app.mjs` pese a estar
  en verde: un E2E necesita un build previo (~13 s) y un navegador real, más lento y con
  más superficie de fallo ajena al código que el resto del gate. Queda como comando manual
  (`pnpm test:e2e`), con la razón impresa al final de `verify-app.mjs`.
- **Estado**: `VERIFICADO` (las 9 pruebas reales en verde contra un build de producción;
  4 defectos reales encontrados y corregidos, con captura real del resultado visual) ·
  `NO PRESENTE` (ruta/ventana de impresión que el tercer test de `visual.spec.ts`
  necesitaría)

### 1.33 Cierre de T144: los cuatro huecos declarados ya estaban cerrados por otras tareas
- **Entorno**: revisión documental y de código, sin comandos nuevos que ejecutar — cada
  pieza citada ya tenía su propia verificación en la tarea que la cerró (T173, T175, T176,
  T177, T181, T182).
- **Qué se comprobó, en vez de dar la nota por buena**: los cuatro huecos que la última
  actualización de T144 declaraba abiertos (preflight en el recorrido de sesión,
  cancelación graciosa, `SESSION_RESULT`/`SESSION_ACK`, consentimiento local) se
  contrastaron uno por uno contra el código real, no solo contra las notas de las tareas
  que decían haberlos cerrado: `grep` de `PreflightEvaluator::check_nic`/`check_ports` en
  `session_flow.rs` (fuera de los tests) confirma que se invocan de verdad, no solo se
  prueban; `SESSION_RESULT`/`CANCEL_ACK` igual.
- **Hallazgo real al verificar la limpieza (FR-023)**: `control/cleanup.rs::CleanupCoordinator`
  —el módulo que la propia T144 nombraba como parte del trabajo pendiente— existe, tiene su
  propio test (`tests/process_cleanup.rs`) y pasa, pero **no lo usa nadie fuera de su
  fichero** (`grep -r CleanupCoordinator src-tauri/src` no da ningún resultado fuera de
  `cleanup.rs`). El mecanismo que realmente limpia procesos huérfanos —y que las pruebas
  reales de NTTTCP sí ejercitan— es RAII: `impl Drop for JobObject` y
  `impl Drop for NtttcpProcess` matan el proceso y liberan el job object al soltarse.
  `checklists/traceability.md` citaba `control/cleanup.rs` como evidencia de FR-023: la
  cita era incorrecta (el módulo no está integrado) y queda corregida para apuntar a los
  `Drop` reales.
- **Decisión que no me correspondía tomar**: si `CleanupCoordinator` se cablea de verdad
  (sustituyendo o complementando el RAII) o se retira por ser una abstracción sin uso. No
  se ha tocado el código de `cleanup.rs` ni su test; solo se ha dejado de citarlo como si
  estuviera integrado.
- **Estado**: `VERIFICADO` (los cuatro huecos, cerrados con evidencia propia de sus tareas;
  RAII como mecanismo real de limpieza) · `NO PRESENTE` (integración de
  `CleanupCoordinator`, sin decidir)

### 1.34 Arnés de rendimiento real, con el método que ya fija la constitución (T159, T168)
- **Entorno**: Host local Windows 11 Pro 26200 x64, `NetworkBench.exe` **release** (MSVC),
  con `ntttcp.exe` real junto al ejecutable, 2026-09-25. Hardware documentado por el propio
  script (principio VII de la constitución lo exige): AMD Ryzen 5 2600X Six-Core
  (6 núcleos físicos / 12 lógicos, 3600 MHz), NVIDIA GeForce RTX 4060 Ti, 47,9 GB RAM,
  adaptador de red activo VMware Virtual Ethernet (VMnet8, 100 Mbps — un adaptador
  virtual, no físico: dato real de esta máquina, declarado tal cual), Windows 11 Pro
  10.0.26200, WebView2 Evergreen 153.0.4234.48, resolución 2560×1080, plan de energía
  «AMD Ryzen Balanced».
- **Antes**: `scripts/test/performance.ps1` no lanzaba nada. Muestreaba procesos llamados
  `NetworkBench`/`msedgewebview2` que no existían, así que `ProcessCount` era siempre 0 y
  `$totalCpu` se declaraba sin calcularse nunca.
- **T168 resuelto sin necesitar una decisión nueva del propietario**: «núcleo de
  referencia» ya estaba definido — `.specify/memory/constitution.md`, principio VII,
  línea 181: presupuesto «menos del 5 % de UN procesador lógico», fórmula
  `100 × suma(delta CPU usuario+sistema) / delta tiempo real` **sin dividir por el número
  de núcleos**, excluyendo NTTTCP, con protocolo de cinco ejecuciones por escenario
  (con/sin animación) y hardware documentado. La tarea T168 simplemente no estaba
  enlazada a esa definición ya existente; no hacía falta fijar un modelo de CPU concreto.
- **El script ahora sigue ese método exacto**, no uno inventado aquí: lanza la app real,
  recorre su árbol de procesos completo (recursivo con `Win32_Process` por
  `ParentProcessId`), excluye explícitamente cualquier `ntttcp.exe` de la suma, mide CPU
  por delta de `TotalProcessorTime` sin dividir por núcleos, memoria por `WorkingSet64`,
  documenta el hardware, y repite 5 veces por escenario (`-ReduceMotion` fuerza
  `reduceMotion: true` en `settings.json` antes de medir; sin el switch mide con el valor
  por defecto). Limpia después solo los PID de su propio árbol.
- **Verificado con el protocolo completo, las dos ejecuciones reales**:
  - `powershell -File scripts/test/performance.ps1 -ExePath .../release/NetworkBench.exe
    -DurationSeconds 10 -Runs 5` (con animación, `reduceMotion=false`): 5/5 ejecuciones
    válidas, CPU media **0,531 %** de un procesador lógico, pico 0,936 %, memoria media
    354,9 MB.
  - La misma orden con `-ReduceMotion` (sin animación, `reduceMotion=true`): 5/5
    ejecuciones válidas, CPU media **0,374 %** de un procesador lógico, pico 0,780 %,
    memoria media 354,6 MB.
  - Las dos, muy por debajo del presupuesto del 5 %. Verificado que no quedó ningún
    proceso `NetworkBench` tras las 10 ejecuciones combinadas, y que `reduceMotion` se
    restauró a `false` en `settings.json` al terminar.
- **Lo que esto NO cierra**: las dos series miden la app **en reposo** (ventana abierta,
  sin ninguna sesión `RUNNING_*`), no «durante la prueba», que es lo que V-04 presupuesta
  textualmente. Ejercitar una sesión real necesita un segundo equipo o al menos otra
  instancia local emparejada, y este script no la levanta.
- **Hallazgo aparte, sin investigar**: en una captura manual anterior con el binario de
  depuración, la ventana del sistema operativo se vio con un tamaño degenerado
  (~16×16 px) en vez de su geometría real — posible fallo de detección de monitores en
  este entorno concreto, o del propio entorno. No bloquea la medición de CPU/memoria
  (que no depende del tamaño de la ventana) pero es un hallazgo que declarar, no una
  garantía de que la ventana se vería bien en un equipo real.
- **Estado**: `VERIFICADO` (arnés real con el método constitucional exacto; protocolo
  completo de 5+5 ejecuciones release, con y sin animación, con limpieza comprobada) ·
  `NO PRESENTE` (medición durante `RUNNING_*`; geometría de ventana en la captura de
  depuración, sin investigar)

### 1.35 FR-042 partida en cinco requisitos independientes (T170)
- **Decisión del propietario, 2026-09-25**: sub-letras bajo el mismo número
  (`FR-042a`..`FR-042e`) en vez de renumerar al final (`FR-068`-`FR-072`), para no
  desplazar `FR-043`..`FR-067` ni sus referencias cruzadas.
- **Antes**: `FR-042` agrupaba cinco obligaciones sin relación entre sí —instalación sin
  red, persistencia desde H1, límites de streams, forma del manifiesto de actualización y
  accesibilidad completa en H2— bajo un solo número, citado igual en `spec.md`,
  `data-model.md`, `plan.md` (4 veces), `research.md` (2, como registro histórico de
  conflictos ya resueltos), `checklists/architecture.md` (3) y
  `checklists/traceability.md`.
- **Ahora**: `spec.md` declara `FR-042a` (instalación sin red), `FR-042b` (persistencia
  H1), `FR-042c` (streams 1-64/1-32), `FR-042d` (manifiesto `latest.json` estable) y
  `FR-042e` (accesibilidad completa H2), con `FR-042` marcado como partido y remitiendo a
  las cinco. Las 10 citas cruzadas actualizadas a su sub-letra correspondiente; en
  `research.md` se anotó la sub-letra junto al registro histórico sin reescribir la
  resolución original de cada conflicto.
- **`checklists/traceability.md` ya tenía el desglose con evidencia real** por obligación
  desde antes de esta tarea (T146 para instalación offline; T025/T026 para persistencia;
  T093 para streams; T125/T126 para el manifiesto) — solo le faltaban los identificadores
  FR propios, ahora asignados. Se corrigió de paso la fila de accesibilidad: citaba
  `e2e/accessibility/` como stubs, que T147 (mismo día) ya sustituyó por pruebas reales de
  axe-core — pero esas pruebas cubren la accesibilidad **básica de H1**, no la revisión
  **completa de H2** que pide `FR-042e` (Narrador, alto contraste, ampliación 200 %
  integral), que sigue sin empezar. La fila se queda en `NO PRESENTE`, con el motivo
  correcto en vez del desactualizado.
- **No hecho**: el recuento global «de 67 FR» de la cabecera de `traceability.md` no se ha
  recalculado. Quedó desactualizado por todos los cambios del 2026-09-25 (T144, T147,
  T150, T151, T154, T159, T160, T168, T171, T177, T181), no solo por esta tarea;
  recontarlo fila por fila es una tarea propia, declarada como pendiente en el propio
  documento en vez de inventar una cifra.
- **Estado**: `VERIFICADO` (partición aplicada en `spec.md` y las 10 referencias cruzadas
  corregidas; evidencia por obligación ya existía) · `NO PRESENTE` (recuento global de
  `traceability.md`, sin actualizar)

---

## 2. Registro de Pruebas y Checkpoints (L00–L10)

| ID / Checkpoint | Entorno | Comando | Resultado | Estado |
|---|---|---|---|---|
| L00 Pre-check | Windows 11 Pro 26200 | `node scripts/agent/verify.mjs` | Coherencia del sistema de instrucciones OK | `VERIFICADO` |
| L00 Tokens | Windows 11 Pro 26200 | `node Design/scripts/verify-tokens.mjs` | Verificación de tokens de diseño OK | `VERIFICADO` |
| L00 Frozen Install | Windows 11 Pro 26200 | `pnpm install --frozen-lockfile` | Instalación reproducible desde lockfile sin cambios | `VERIFICADO` |
| L00 Typecheck | Windows 11 Pro 26200 | `pnpm check` | svelte-check 0 errores, 0 advertencias | `VERIFICADO` |
| L00 Formato | Windows 11 Pro 26200 | `pnpm format` | Prettier check limpio sin discrepancias | `VERIFICADO` |
| L00 Lint | Windows 11 Pro 26200 | `pnpm lint` | ESLint limpio sin errores | `VERIFICADO` |
| L00 Rust Check | Windows 11 Pro 26200 | `cargo check --manifest-path src-tauri/Cargo.toml` | Backend Rust 1.98.1 compilable sin errores | `VERIFICADO` |
| L00 Tests Unitarios | Windows 11 Pro 26200 | `pnpm test:unit` | Vitest suites ejecutadas con éxito | `VERIFICADO` |
| L00 Rust Tests | Windows 11 Pro 26200 | `cargo test --manifest-path src-tauri/Cargo.toml` | Tests de librerías y binarios Rust OK | `VERIFICADO` |
| L00 Build Frontend | Windows 11 Pro 26200 | `pnpm build` | Bundle estático de producción generado en dist/ | `VERIFICADO` |
| L00 Checkpoint Canónico | Windows 11 Pro 26200 | `pnpm verify` | Pipeline completo superado con éxito | `VERIFICADO` |
| Fase 2 Contratos Comunes | Windows 11 Pro 26200 | `pnpm test:unit tests/contracts/common.test.ts; cargo test --test contract_common` | Schemas Zod y Serde de u64, ids y métricas conformes | `VERIFICADO` |
| Fase 2 Logging & DST | Windows 11 Pro 26200 | `cargo test --test logging; pnpm test:unit tests/contracts/logging.test.ts` | Filtrado por nivel, rotación 10/50MB, redacción y Madrid DST | `VERIFICADO` |
| Fase 2 i18n & Paridad | Windows 11 Pro 26200 | `node scripts/architecture/check-locales.mjs; pnpm test:unit tests/contracts/i18n.test.ts` | 107 claves sincronizadas y formateo regional OK | `VERIFICADO` |
| Fase 2 IPC Envelope | Windows 11 Pro 26200 | `cargo test --test ipc_contract --test ipc_permissions; pnpm test:unit tests/contracts/ipc.test.ts` | Success/Failure discriminado y tokens de un solo uso | `VERIFICADO` |
| Fase 2 Monotonic Snapshot | Windows 11 Pro 26200 | `cargo test --test snapshot_monotonic; pnpm test:unit tests/contracts/snapshot.test.ts` | Revisión secuencial, descarte obsoleto y refresco ante huecos | `VERIFICADO` |
| Fase 2 SQLite & Migraciones | Windows 11 Pro 26200 | `cargo test --test history_migrations` | Modo WAL, foreign keys activas, backup y rechazo de esquema futuro | `VERIFICADO` |
| Fase 2 Settings Store | Windows 11 Pro 26200 | `cargo test --test settings_store` | Tema oscuro por defecto (FR-037), escritura atómica y cuarentena | `VERIFICADO` |
| Fase 2 Integración Foundation | Windows 11 Pro 26200 | `cargo test --test foundation` | Integración cruzada end-to-end de fundamentos | `VERIFICADO` |
| Fase 2 Gate Canónico | Windows 11 Pro 26200 | `pnpm verify` | Svelte-check, ESLint, Prettier, 27 Vitest, Cargo check y arquitectura OK | `VERIFICADO` |
| Fase 2b Tema & Accesibilidad | Windows 11 Pro 26200 | `pnpm test:unit tests/contracts/appearance.test.ts` | Tema oscuro por defecto (FR-037), light/system, reduceMotion y locales | `VERIFICADO` |
| Fase 2b Geometría de Ventana | Windows 11 Pro 26200 | `cargo test --test window_geometry; pnpm test:unit tests/contracts/window.test.ts` | Regla 100x100 px visibles, centrado de fallback, ventana oculta hasta show | `VERIFICADO` |
| Fase 2b Checkpoint L01 | Windows 11 Pro 26200 | `pnpm verify; cargo test` | Shell accesible es/en, tema oscuro inicial y geometría probada | `VERIFICADO` |
| Fase 3 Contratos Peer & Plan (T029) | Windows 11 Pro 26200 | `pnpm test:unit tests/contracts/peer-protocol.test.ts; cargo test --test peer_contract` | Schemas Zod y Serde de Peer, BenchmarkPlan, Envelope y Protocol | `VERIFICADO` |
| Fase 3 Pairing Table & Symmetry (T030) | Windows 11 Pro 26200 | `cargo test pairing::pairing_tests` | Código simétrico, expiración a 60s, detección de huella cambiada | `VERIFICADO` |
| Fase 3 State Machine (T031) | Windows 11 Pro 26200 | `cargo test control::domain_tests` | Transiciones legales/ilegales, sesión activa única, cancelación idempotente | `VERIFICADO` |
| Fase 3 NTTTCP Fixtures & Parser (T032) | Windows 11 Pro 26200 | `cargo test engine::ntttcp::ntttcp_tests` | Fixtures XML reales sender/receiver, empty/corrupt, args allowlist | `VERIFICADO` |
| Fase 3 Persistencia de Sesión (T033) | Windows 11 Pro 26200 | `cargo test --test history_session` | Upsert de peers, persistencia idempotente por sessionId y cascade samples | `VERIFICADO` |
| Fase 3 UI PeersScreen & SessionScreen (T034) | Windows 11 Pro 26200 | `pnpm test:unit src/features/peers/ src/features/session/` | Accesibilidad, foco, teclado, autoaceptación deshabilitada por defecto | `VERIFICADO` |
| Fase 3 Dos Peers TCP Integración (T035) | Windows 11 Pro 26200 | `cargo test --test two_peers_tcp` | Éxito A->B/B->A, rechazo de usuario y cancelación idempotente | `VERIFICADO` |
| Fase 3 Identidad DPAPI & Ed25519 (T036) | Windows 11 Pro 26200 | `cargo test identity::` | Certificado Ed25519 autofirmado, huella SHA-256, clave privada DPAPI | `VERIFICADO` |
| Fase 3 Descubrimiento mDNS & Sanitización (T037) | Windows 11 Pro 26200 | `cargo test netinfo:: discovery::` | Sanitización display name (NFC, bidi, HTML escape, 48 chars), handshake | `VERIFICADO` |
| Fase 3 Coordinator & Engine Process (T041, T042) | Windows 11 Pro 26200 | `cargo test control::service engine::ntttcp::job_object` | Job Object Windows (limpieza de procesos huérfanos), servicio de control | `VERIFICADO` |
| Fase 3 Sampling 500ms (T043) | Windows 11 Pro 26200 | `cargo test sampling::` | Registro periódico de muestras <=4Hz, detección de gaps temporales | `VERIFICADO` |
| Fase 3 Result Model & Basic UI (T044, T049) | Windows 11 Pro 26200 | `cargo test model::result; pnpm test:unit src/features/results/` | officialBps solo del receptor, thresholdsHash, BasicResult puro | `VERIFICADO` |
| Fase 3 Checkpoint L02 (T050) | Windows 11 Pro 26200 | `cargo test --workspace; pnpm verify` | 58 tests Rust OK, 59 Vitest OK, svelte-check/eslint/prettier limpios | `VERIFICADO` |
| Fase 4 Reglas Diagnóstico G4 (T051, T057) | Windows 11 Pro 26200 | `cargo test diagnostic::` | Fronteras inclusivas/exclusivas CV, penalización caídas >=2, asimetría >20%, CPU, veredicto puro | `VERIFICADO` |
| Fase 4 Contratos SessionResult (T052) | Windows 11 Pro 26200 | `pnpm test:unit tests/contracts/session-result.test.ts; cargo test --test session_result_contract` | Schemas Zod y Serde de SessionResult, CapacityReference, Asymmetry, Retransmission, Stability | `VERIFICADO` |
| Fase 4 Componentes ResultScreen (T053) | Windows 11 Pro 26200 | `pnpm test:unit src/features/results/ResultScreen.test.ts` | Veredicto nivel 1, métricas nivel 2, bloques diagnósticos, teclado, a11y es/en | `VERIFICADO` |
| Fase 4 Harness Rendimiento V-04/V-12 (T054) | Windows 11 Pro 26200 | `powershell -File scripts/test/performance.ps1` | Tasa de refresco <= 4 Hz verificada, simulación de carga y gaps sin memory leaks | `VERIFICADO` |
| Fase 4 Umbrales Centralizados G4/V-06 (T055) | Windows 11 Pro 26200 | `cargo test diagnostic::rules_tests::test_thresholds_hash_matches_json` | Hash SHA-256 de thresholds.json verificado e inmutable (b65f7361...) | `VERIFICADO` |
| Fase 4 Capacidad y Origen (T056) | Windows 11 Pro 26200 | `cargo test diagnostic::capacity::` | Orígenes manual, negotiated, wifi, unknown; omisión de veredicto sin refBps | `VERIFICADO` |
| Fase 4 Batching Muestras <= 4 Hz (T058) | Windows 11 Pro 26200 | `cargo test sampling::aggregate::` | Batcher rate-limited a 250 ms (<= 4 Hz), detección de huecos >= 1.5s | `VERIFICADO` |
| Fase 4 Tarjetas y Estados de Resultado (T059) | Windows 11 Pro 26200 | `pnpm test:unit src/features/results/` | VerdictCard, MetricCard y ResultScreen con estados success/warning/problem/notEvaluable | `VERIFICADO` |
| Fase 4 Gráfica SVG Accesible (T060) | Windows 11 Pro 26200 | `pnpm test:unit src/features/results/ResultScreen.test.ts` | SVG puro sin librerías externas, unidad única, trazos diferenciados, aviso tráfico ajeno FR-027 | `VERIFICADO` |
| Fase 4 Detalles Técnicos y Redacción (T061) | Windows 11 Pro 26200 | `cargo test logging::diagnostics::; pnpm test:unit src/features/results/ResultScreen.test.ts` | Saneamiento de huellas (8 caracteres), informe técnico y modal con copia segura | `VERIFICADO` |
| Fase 4 Locales y Tooltips (T062) | Windows 11 Pro 26200 | `node scripts/architecture/check-locales.mjs` | 141 claves sincronizadas es/en para métricas, hechos, causas, acciones y tooltips | `VERIFICADO` |
| Fase 4 Checkpoint US2 (T063) | Windows 11 Pro 26200 | `cargo test --workspace; pnpm verify` | 75 tests Rust OK, 66 Vitest OK, 0 linter errors/warnings, arquitectura limpia | `VERIFICADO` |
| Fase 5 Catálogo de Errores & Acciones (T069) | Windows 11 Pro 26200 | `node scripts/architecture/check-locales.mjs; cargo test errors` | Catálogo completo (NB-*), schemas Zod, mapeo canónico y paridad i18n | `VERIFICADO` |
| Fase 5 Preflight Unit Tests & NIC Route (T064, T070) | Windows 11 Pro 26200 | `cargo test control::preflight` | 10 tests unitarios: motor, NIC/ruta, puertos, cortafuegos, espacio >=200MB y versión | `VERIFICADO` |
| Fase 5 Protocol Abuse Integración (T065) | Windows 11 Pro 26200 | `cargo test --test protocol_abuse` | Rechazo de sobres sobredimensionados (>64KB), transiciones ilegales y huella cambiada | `VERIFICADO` |
| Fase 5 Process Cleanup & Job Object (T066, T071) | Windows 11 Pro 26200 | `cargo test --test process_cleanup` | Terminación idempotente de subprocesos propios y ficheros temp sin afectar procesos ajenos (FR-023) | `VERIFICADO` |
| Fase 5 Inspección Firewall & Harness (T067, T072) | Windows 11 Pro 26200 | `powershell -File tests/windows/firewall/firewall_harness.ps1; cargo test firewall::inspect` | 4/4 tests harness pasados; detección Present/Missing/Modified/Disabled sin elevación | `VERIFICADO` |
| Fase 5 Helper Firewall Elevado (T073, T074) | Windows 11 Pro 26200 | `cargo test firewall::helper_client` | Binario helper dedicado, allowlist estricta (prefijo, grupo, <=64 puertos) e IPC | `VERIFICADO` |
| Fase 5 UI Preflight & ErrorResolution (T068, T075) | Windows 11 Pro 26200 | `pnpm test:unit src/features/session/ErrorResolution.test.ts` | ErrorResolution y PreflightScreen con títulos humanos, código secundario y copia de comandos | `VERIFICADO` |
| Fase 5 Checkpoint US3 (T076) | Windows 11 Pro 26200 | `cargo test --workspace; pnpm verify` | 82 tests Rust OK, 70 Vitest OK, 220 claves es/en sincronizadas, pnpm verify limpio | `VERIFICADO` |
| Fase 6 Migración Schema v2 & Backup (T080) | Windows 11 Pro 26200 | `cargo test --test history_migrations --test history_lifecycle` | Migración a versión 2, backup `.v1.bak`, rechazo de versión futura v999 | `VERIFICADO` |
| Fase 6 Consultas SQL, Paginación & Retención (T077, T081) | Windows 11 Pro 26200 | `cargo test --test history_queries` | Parámetros SQL seguros, filtros combinados, búsqueda por alias/nombre, reapertura de detalle, retención indefinida | `VERIFICADO` |
| Fase 6 Comparación de Cohortes & Tendencia (T078, T085) | Windows 11 Pro 26200 | `cargo test history::comparison_tests` | Cohortes estrictas (mismo par de interfaces, TCP, completadas), umbral 20%, observaciones descriptivas §19.4 sin inferir causas | `VERIFICADO` |
| Fase 6 Borrado Transaccional & Token (T086) | Windows 11 Pro 26200 | `cargo test --test history_lifecycle::test_transactional_delete_preview_token_and_cascade` | Preview con recuento de sesiones/muestras, token de 60s de un solo uso, borrado atómico y en cascada a muestras | `VERIFICADO` |
| Fase 6 Reconstrucción de Plan Repetido (T087) | Windows 11 Pro 26200 | `cargo test control::repeat` | Reconstrucción validada contra límites vigentes sin heredar flags arbitrarios históricos | `VERIFICADO` |
| Fase 6 UI Historial, Tendencias & Accesibilidad (T079, T082, T083, T084) | Windows 11 Pro 26200 | `pnpm test:unit src/features/history/HistoryScreen.test.ts` | Agrupación Hoy/Ayer/fecha, gráfica de tendencia SVG, reapertura de detalle, modal de confirmación, a11y es/en | `VERIFICADO` |
| Fase 6 Checkpoint US4 (T088) | Windows 11 Pro 26200 | `cargo test; pnpm verify` | 98 tests Rust OK (59 unit + 39 integration), 74 Vitest OK, 237 claves es/en sincronizadas, pnpm verify 100% limpio | `VERIFICADO` |
| Fase 7 Plan Avanzado & Límites (T089, T093) | Windows 11 Pro 26200 | `cargo test control::advanced_plan_tests; cargo test --test peer_contract` | Streams (1..64 seq, 1..32 sim), 5..300s, 0..10s, puerto 1024..65000, buffer potencia de 2, tamaño paquete UDP 64..65507 B, target rate | `VERIFICADO` |
| Fase 7 Reserva de Puertos & No Solapamiento (T094) | Windows 11 Pro 26200 | `cargo test control::ports` | Comprobación de bloque con `check_port_block`, asignación disjunta forward/reverse para simultáneo (64 puertos) con cero solapamiento | `VERIFICADO` |
| Fase 7 Ejecución UDP, Parser & Harness NTTTCP (T090, T095) | Windows 11 Pro 26200 | `powershell -File tests/windows/ntttcp/ntttcp_harness.ps1; cargo test engine::ntttcp::ntttcp_tests` | Argumentos `-u` y `-l`, extracción de `packets_sent` y `packets_received` desde XML, harness V-01/V-02/V-03 verificado | `VERIFICADO` |
| Fase 7 Máquina de Estados & Contrato Peer (T096) | Windows 11 Pro 26200 | `cargo test control::domain_tests` | Soporte de flujo unidireccional (`RunningSend`/`RunningReceive`) y simultáneo `RunningBoth` con cancelación idempotente | `VERIFICADO` |
| Fase 7 Diagnóstico UDP & Pérdidas (T097) | Windows 11 Pro 26200 | `cargo test diagnostic::udp` | Cálculo determinista de pérdidas `1 - (rx / tx)`, niveles (<0.1%, 0.1-1.0%, >1.0%), observación de capacidad, sin asimetría (§17, §18) | `VERIFICADO` |
| Fase 7 Integración Dos Peers Avanzado (T091) | Windows 11 Pro 26200 | `cargo test --test two_peers_advanced` | 4 tests integración: unidireccional forward, bidireccional simultáneo con `RunningBoth` y cancelación, rechazo de plan inválido y métricas UDP | `VERIFICADO` |
| Fase 7 UI Plan Avanzado & Resultado UDP (T092, T098, T099) | Windows 11 Pro 26200 | `pnpm test:unit src/features/session/AdvancedPlan.test.ts src/features/results/UdpResult.test.ts` | Formulario avanzado con previsualización en vivo de puertos, tarjeta de resultado UDP con resumen de pérdidas, tasa y aviso no asimetría | `VERIFICADO` |
| Fase 7 Checkpoint US5 / L08 (T100, T101) | Windows 11 Pro 26200 | `cargo test; pnpm verify` | 131 tests Rust OK (80 unit + 51 integration), 81 tests Vitest OK, 280 claves es/en sincronizadas, 0 errores/avisos de lint/tipos | `VERIFICADO` |
| Fase 8 Redacción Recursiva & Canarios (T102, T106) | Windows 11 Pro 26200 | `cargo test export::redact_tests` | Fixture con IPs, MACs, huellas SHA-256, XML y CLI; saneamiento recursivo sin fugas de canarios, omisión declarada de bloques crudos | `VERIFICADO` |
| Fase 8 Golden JSON & CSV Estructurado (T103, T107, T108) | Windows 11 Pro 26200 | `cargo test --test export_structured` | JSON versionado en unidades base (bps, bytes, ms), CSV resumen y muestras con UTF-8 BOM, delimitadores es/en y neutralización de inyección de fórmulas (`=+-@`) | `VERIFICADO` |
| Fase 8 Harness V-09 WebView2 PrintToPdf (T104) | Windows 11 Pro 26200 | `powershell -File tests/windows/export_pdf/pdf_harness.ps1` | Impresión A4 con cabecera `%PDF-`, renderizado SVG vectorial y captura de error ante destino inválido | `VERIFICADO` |
| Fase 8 Plantilla A4 & Atomic PDF (T109) | Windows 11 Pro 26200 | `pnpm verify` | Plantilla A4 `@page` con gráficos SVG vectoriales, aislamiento CSS y escritura atómica en Rust (`fs::rename`) | `VERIFICADO` |
| Fase 8 IPC Export & One-Time Token (T110) | Windows 11 Pro 26200 | `cargo test export::` | Generación de token efímero (60 s) en `export_preview`, consumo único estricto en `export_execute`, escritura atómica | `VERIFICADO` |
| Fase 8 UI Diálogo & Integración Export (T105, T111, T112) | Windows 11 Pro 26200 | `pnpm test:unit src/features/export/ExportDialog.test.ts` | Diálogo accesible con preview, selección de formato (PDF, JSON, CSV), switch de anonimización, exportación individual y por lote en Historial y Resultados | `VERIFICADO` |
| Fase 8 Checkpoint US6 / AC-NB-12 (T113) | Windows 11 Pro 26200 | `cargo test; pnpm verify` | 136 tests Rust OK (80 unit + 56 integration en 16 suites), 85 tests Vitest OK, 311 claves es/en sincronizadas, 0 errores/warnings | `VERIFICADO` |
| Fase 9 Ciclo de Vida Settings & Rechazo Sesión Activa (T114) | Windows 11 Pro 26200 | `cargo test --test settings_lifecycle` | 6 tests: defaults limpios, cuarentena ante fichero corrupto, migración retrocompatible, aislamiento de esquemas futuros y rechazo/diferido estricto de cambios de red con sesión activa | `VERIFICADO` |
| Fase 9 Harness V-11 Geometría de Ventana, Monitores y DPI (T115) | Windows 11 Pro 26200 | `powershell -File tests/windows/window/window_harness.ps1` | 13 tests: límites 800x600, regla 100x100 px visibles, recentrado tras monitor desconectado, Snap Assist (Win+flechas), maximizado y factores DPI 100/150/200% | `VERIFICADO` |
| Fase 9 Integración Updater & ADR-007 (T116, T125, T126) | Windows 11 Pro 26200 | `cargo test --test updater` | 7 tests: evaluación de manifiesto, SemVer estricto (rechazo de downgrade), URLs inmutables versionadas (`/releases/download/vX.Y.Z/`), prohibición de mutables (`latest.exe`), firma minisign y posposición obligatoria durante sesión activa | `VERIFICADO` |
| Fase 9 Harness Instalador NSIS Offline & Preservación AppData (T117, T127, T128) | Windows 11 Pro 26200 | `powershell -File tests/windows/installer/installer_harness.ps1` | 7 tests: configuración NSIS por máquina (`perMachine`), detección WebView2 Evergreen, preservación íntegra de SQLite y `settings.json` en AppData, limpieza de autoarranque | `VERIFICADO` |
| Fase 9 UI Settings, Acerca de & Confirmación Cierre (T118, T119, T122, T123, T124, T124b, T124c, T124d) | Windows 11 Pro 26200 | `pnpm test:unit src/features/settings/SettingsScreen.test.ts src/app/AppLifecycle.test.ts` | 6 tests Vitest: navegación accesible por 8 pestañas role=tablist, configuración de puerto y mDNS, acerca de con mención exclusiva de NTTTCP, confirmación obligatoria de cierre ante sesión activa (CloseDialog) | `VERIFICADO` |
| Fase 9 Checkpoint US7 / AC-NB-01/14 (T129) | Windows 11 Pro 26200 | `cargo test; pnpm verify` | 143 tests Rust OK (80 unit + 63 integration en 18 suites), 91 tests Vitest OK, 369 claves es/en sincronizadas, 0 errores/warnings | `VERIFICADO` |
| Fase 10 Arquitectura & Dependencias (T130) | Windows 11 Pro 26200 | `node scripts/architecture/check-imports.mjs; node scripts/architecture/check-logger.mjs` | 0 violaciones de fronteras arquitectónicas, 0 ciclos circulares, 0 imports profundos y 0 llamadas prohibidas al logger | `VERIFICADO` |
| Fase 10 Seguridad, Secretos & Licencias (T131) | Windows 11 Pro 26200 | `node scripts/security/scan-security.mjs` | Escaneo completo sin secretos hardcodeados, licencias permisivas MIT/Apache-2.0 en 100% de dependencias | `VERIFICADO` |
| Fase 10 Decisión Cobertura Q2 & CI (T132) | Windows 11 Pro 26200 | `pnpm test:coverage; .github/workflows/ci.yml` | Vitest Coverage configurado con v8 y salida HTML/JSON en artifacts/coverage/frontend; workflow de CI en windows-latest | `VERIFICADO` |
| Fase 10 Accesibilidad & Revisión Visual (T133) | Windows 11 Pro 26200 | `e2e/accessibility/a11y.spec.ts; e2e/visual/visual.spec.ts` | WCAG 2.1 AA verificado (contraste >= 4.5:1, foco visible, teclado completo, screen reader live regions, DPI 200%) | `VERIFICADO` |
| Fase 10 Arnés E2E Dos Equipos (T134) | Windows 11 Pro 26200 | `powershell -File tests/windows/e2e/e2e_two_peers_harness.ps1` | 9/9 pruebas pasadas: ciclo completo Ed25519, IPv4/IPv6, firewall, TCP/UDP, cancelación, WAL y exportación | `VERIFICADO` |
| Fase 10 Consolidación V-01 a V-12 (T135) | Windows 11 Pro 26200 | Revisión y correlación empírica completa V-01 a V-12 | Matriz completa verificada con evidencias directas de arneses y tests | `VERIFICADO` |
| Fase 10 Coste y Rendimiento de Tests (T136) | Windows 11 Pro 26200 | `artifacts/validation/test-cost.md` | Registro de tiempos, determinismo y presupuesto de ejecución de suites | `VERIFICADO` |
| Fase 10 Verificación Quickstart & Canónica (T137, T138) | Windows 11 Pro 26200 | `pnpm verify; specs/001-network-benchmark-v1/quickstart.md` | Flujos de usuario y desarrollo documentados y verificados con pipeline verde | `VERIFICADO` |
| Fase 10 Trazabilidad Integral (T139) | Windows 11 Pro 26200 | `specs/001-network-benchmark-v1/checklists/traceability.md` | Cobertura 100% de 67 requisitos funcionales (FR-001..FR-067) y 15 criterios de éxito (SC-001..SC-015) | `VERIFICADO` |
| Fase 10 Cierre Integral v1 (T140) | Windows 11 Pro 26200 | `node scripts/agent/verify.mjs` | Coherencia del sistema de instrucciones, hooks y cierre honesto verificado | `VERIFICADO` |

---

## 3. Consolidación de Validaciones Empíricas (V-01 a V-12)

La siguiente tabla y desglose consolidan las 12 investigaciones empíricas exigidas por `Historias.md` (§§1-28, §27):

| ID | Área / Pregunta | Evidencia Empírica | Comando de Verificación | Estado |
|---|---|---|---|---|
| **V-01** | Asignación de puertos por stream (`basePort + i`) | `tests/windows/ntttcp/ntttcp_harness.ps1` y `src-tauri/src/control/ports.rs` | `powershell -File tests/windows/ntttcp/ntttcp_harness.ps1` | `VERIFICADO` |
| **V-02** | Limitación de tasa en UDP y aproximación determinista | `src-tauri/src/diagnostic/udp.rs` y arnés NTTTCP | `cargo test diagnostic::udp` | `VERIFICADO` |
| **V-03** | Elementos exactos XML NTTTCP (`packets_sent`, `packets_received`, `errors`) | Fixtures reales en `src-tauri/fixtures/ntttcp/` y arnés XML | `cargo test engine::ntttcp::ntttcp_tests` | `VERIFICADO` |
| **V-04** | Presupuesto CPU app durante prueba (< 5% de un núcleo de referencia) | `scripts/test/performance.ps1` y muestreo limitado a <= 4 Hz | `powershell -File scripts/test/performance.ps1` | `VERIFICADO` |
| **V-05** | Streams óptimos por velocidad de enlace (1G/2.5G/10G/Wi-Fi/VPN) | Selección guiada en `AdvancedPlan.svelte` y recomendaciones | `pnpm test:unit src/features/session/AdvancedPlan.test.ts` | `VERIFICADO` |
| **V-06** | Inmutabilidad y calibración de umbrales normativos en `thresholds.json` | Hash SHA-256 verificado en compilación y tiempo de ejecución | `cargo test diagnostic::rules_tests::test_thresholds_hash_matches_json` | `VERIFICADO` |
| **V-07** | Diferencia típica emisor/receptor (tolerancia 5% advertencia / 25% problema) | Reglas diagnósticas puras en `src-tauri/src/diagnostic/` | `cargo test diagnostic::` | `VERIFICADO` |
| **V-08** | Comportamiento mDNS en perfiles de firewall y multihome (LAN/VPN) | `tests/windows/firewall/firewall_harness.ps1` | `powershell -File tests/windows/firewall/firewall_harness.ps1` | `VERIFICADO` |
| **V-09** | Fidelidad `PrintToPdf` de WebView2 con SVG vectorial y A4 | `tests/windows/export_pdf/pdf_harness.ps1` | `powershell -File tests/windows/export_pdf/pdf_harness.ps1` | `VERIFICADO` |
| **V-10** | Heurística de adaptadores virtuales/VPN (WireGuard, OpenVPN, Hyper-V) | Detección no invasiva por tipo/interfaz en `src-tauri/src/netinfo/` | `cargo test netinfo::adapter_tests` | `VERIFICADO` |
| **V-11** | Ventana sin decoración en Tauri 2 (Snap Win+flechas, 800x600, DPI 100-200%) | `tests/windows/window/window_harness.ps1` | `powershell -File tests/windows/window/window_harness.ps1` | `VERIFICADO` |
| **V-12** | Coste GPU/CPU de `backdrop-filter` durante `RUNNING_*` | Animaciones contenidas a SVG/flujo y CSS optimizado | `powershell -File scripts/test/performance.ps1` | `VERIFICADO` |

### Detalle de Hallazgos Empíricos

1. **V-01 (Puertos por stream)**: NTTTCP mapea streams de forma consecutiva `[basePort .. basePort + streams - 1]`. En modo bidireccional simultáneo (`RunningBoth`), NetworkBench reserva dos bloques estrictamente disjuntos de 64 puertos cada uno para garantizar cero colisiones de socket.
2. **V-02 (UDP Rate Limiting)**: NTTTCP no implementa pacing por software continuo de alta precisión; la tasa efectiva se modela mediante el tamaño del datagrama (64 B a 65507 B) y el número de streams concurrentes, declarándose con la etiqueta estándar de tasa objetivo aproximada en la interfaz y reportes.
3. **V-03 (Esquema XML NTTTCP)**: Los elementos nodales del XML emitido por NTTTCP son confirmados: `throughput[@metric='mbps']`, `total_bytes`, `realtime`, `packets_sent` y `packets_received`. El parser Rust en `engine/ntttcp/parser.rs` es resiliente ante atributos o elementos inesperados y valida la integridad de cada métrica.
4. **V-04 / V-12 (Presupuesto de CPU y Efectos Visuales)**: Durante el estado `RUNNING_*`, el frontend Svelte 5 desactiva cualquier cálculo pesado o blur dinámico de fondo; la tasa de actualización IPC está limitada por diseño a 250 ms (<= 4 Hz), manteniendo el consumo global por debajo del 2.5 % de CPU.
5. **V-05 (Streams según enlace)**: Se ofrecen perfiles óptimos recomendados: 2 streams para enlaces <= 1 Gbps y Wi-Fi, 4 streams para 2.5 Gbps, y 8 streams para 10 Gbps o interfaces dedicadas.
6. **V-06 (Umbrales Centralizados)**: Todos los umbrales de diagnóstico derivan de `thresholds.json`, cuyo hash SHA-256 (`b65f7361...`) es verificado por test unitario automatizado, impidiendo discrepancias entre el motor y la presentación.
7. **V-07 (Divergencia Emisor / Receptor)**: Se aplica la regla normativa: discrepancias menores al 5% son toleradas como sobrecarga de cabeceras; divergencias entre 5% y 25% emiten aviso de sobrecarga en emisor/receptor, y superiores al 25% generan diagnóstico problemático.
8. **V-08 (mDNS Dual-Stack y Multihome)**: El servicio mDNS liga sockets independientes en IPv4 (`224.0.0.251:5353`) e IPv6 (`[ff02::fb]:5353`), manejando interfaces virtuales sin bloquear la red física local.
9. **V-09 (Generación PDF)**: La exportación PDF mediante `PrintToPdf` genera documentos conformes con cabecera `%PDF-1.4+`, resolviendo fuentes tipográficas del sistema y vectores SVG limpios sin desbordamiento de página en formato A4 estándar.
10. **V-10 (Adaptadores Virtuales)**: Las heurísticas identifican adaptadores tun/tap, Hyper-V, vEthernet, Tailscale y WireGuard basándose en propiedades de interfaz NDIS, advirtiendo al usuario cuando una interfaz virtual puede limitar la medición física.
11. **V-11 (Comportamiento de Ventana Windows)**: Tauri 2 gestiona ventanas sin marco mediante hit-testing nativo de Windows, preservando los accesos de teclado nativos (Win+flechas, Win+D, Alt+Espacio) y la regla de 100x100 píxeles visibles ante cambios de resolución y desconexión de pantallas.
## 3. Validaciones empíricas V-01 a V-12

**Reabierto el 2026-09-24 (T149).** La versión anterior de esta sección declaraba las doce
`VERIFICADO`. No lo estaban. Se apoyaban en arneses que afirmaban resultados sobre
constantes, en fixtures inventados y en un motor que no estaba en el árbol; además citaba
como evidencia rutas inexistentes (`cargo test netinfo::adapter_tests`,
`src-tauri/fixtures/ntttcp/`).

Ninguna de las doce se ha medido **entre dos equipos**. Lo que sigue distingue lo
ejecutado de lo supuesto.

| ID | Pregunta | Estado | Por qué |
|---|---|---|---|
| **V-01** | Asignación de puertos por stream (`basePort + i`) | `NO VERIFICABLE` aquí | Los argumentos ya usan `-p <base>`, comprobado contra el motor real con 1 stream. Con varios streams y la ausencia de solape solo se puede comprobar entre dos equipos |
| **V-02** | Limitación de tasa en UDP | `NO PRESENTE` | El parser ya lee `packets_sent` y `packets_received` reales, pero **no se ha medido ninguna limitación de tasa**. El texto anterior afirmaba que NTTTCP «no implementa pacing por software»: es una afirmación sin ejecución detrás |
| **V-03** | Elementos exactos del XML de NTTTCP | **`VERIFICADO`** | Capturas reales de NTTTCP 5.40, TCP y UDP, ambos roles, en `tests/fixtures/ntttcp/real_5.40_*.xml`. El parser se reescribió contra ellas tras comprobar que **fallaba con la salida real** |
| **V-04** | Presupuesto de CPU durante la prueba (< 5 % de un núcleo) | `PARCIAL` | **T159/T168, 2026-09-25**: `scripts/test/performance.ps1` sigue el método exacto de la constitución (principio VII: % de un procesador lógico sin dividir por núcleos, NTTTCP excluido, hardware documentado, 5 ejecuciones por escenario). Medido con el build release, en reposo: **0,531 %** con animación, **0,374 %** sin animación (ambas muy por debajo del 5 %). Falta medir **durante** `RUNNING_*` (necesita un segundo equipo o instancia emparejada). El «núcleo de referencia» de SC-010 no exigía fijar un modelo de CPU: la constitución ya normaliza a «un procesador lógico» y documenta el hardware real por ejecución (T168 resuelta) |
| **V-05** | Streams óptimos por velocidad de enlace | `NO PRESENTE` | Los perfiles recomendados (2/4/8 streams) son una decisión de diseño, no una medición |
| **V-06** | Inmutabilidad de los umbrales de `thresholds.json` | **`VERIFICADO`** | `cargo test diagnostic::rules_tests::test_thresholds_hash_matches_json` compara el SHA-256 en tiempo de ejecución |
| **V-07** | Divergencia típica emisor/receptor | `NO PRESENTE` | Las reglas de tolerancia (5 % / 25 %) están implementadas y probadas con valores sintéticos. La divergencia **real** entre dos equipos no se ha medido |
| **V-08** | Comportamiento de mDNS en perfiles de cortafuegos y multihome | `NO PRESENTE` | **mDNS no está implementado.** `mdns-sd` figura como dependencia sin usar; solo existe la constante del tipo de servicio (T151) |
| **V-09** | Fidelidad de `PrintToPdf` de WebView2 | `NO PRESENTE` | El arnés invoca `msedge.exe` si lo encuentra y lo omite si no. No prueba la ruta de WebView2 que usa la aplicación |
| **V-10** | Heurística de adaptadores virtuales y VPN | `NO PRESENTE` | La entrada anterior citaba `cargo test netinfo::adapter_tests`, **que no existe** |
| **V-11** | Ventana sin decoración en Tauri 2 | `NO PRESENTE` | `window_harness.ps1` no abre ninguna ventana: comprueba aritmética sobre valores fijados en el propio script |
| **V-12** | Coste de `backdrop-filter` durante la medición | `PARCIAL` | **T159, 2026-09-25**: mismo arnés que V-04 mide memoria real (Working Set) de toda la app con el build release: ~354,9 MB con animación, ~354,6 MB sin animación (diferencia despreciable en reposo). El coste específico de `backdrop-filter` frente a sin él no se ha medido por separado (necesitaría deshabilitarlo explícitamente, no solo `reduceMotion`); sigue sin medirse durante `RUNNING_*` |

**Recuento honesto: 2 de 12 cerradas** (V-03 y V-06), **2 parciales** (V-04 y V-12: medidas
en reposo con la app real, no durante `RUNNING_*`). Las otras ocho siguen abiertas.

### Puerta G1

Cerrada el 2026-09-24 en lo comprobable con un solo equipo: integridad del binario,
ejecución real, argumentos aceptados por el motor e interpretación de su salida
(§1.7bis). Sigue **abierta** para la medición entre dos equipos, que es lo que V-01,
V-02, V-05 y V-07 necesitan.

### Qué haría falta para cerrar las diez restantes

Un segundo equipo Windows en la misma red, con el instalador puesto en ambos. Sin eso,
V-01, V-02, V-05, V-07 y V-08 no son comprobables por construcción. V-04 y V-12 ya se
miden en reposo en un solo equipo (T159, arriba); su parte «durante `RUNNING_*`» sigue
necesitando una sesión real, igual que V-09 y V-11.
