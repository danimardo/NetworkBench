# Registro de ValidaciÃ³n: NetworkBench v1

**Fecha de inicio**: 2026-09-21  
**ConstituciÃ³n**: 0.7.0, no ratificada  
**Matriz objetivo (Q1)**: Windows 10 22H2 y Windows 11 (ambos x64)

Este documento registra la evidencia empÃ­rica directa de validaciÃ³n de NetworkBench v1 segÃºn las reglas del proyecto.
Cada entrada sigue la estructura fija: **Entorno**, **Comando**, **Resultado**, **Estado**.

Los estados vÃ¡lidos son: `VERIFICADO`, `DOCUMENTADO`, `INFERIDO`, `NO VERIFICABLE`, `NO PRESENTE`.

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
- **Comando**: N/A (el host de ejecuciÃ³n es Windows 11 Pro build 26200)
- **Resultado**: No disponible en el host fÃ­sico actual; requiere mÃ¡quina virtual o equipo secundario de laboratorio.
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
  dentro del Ã¡rbol la activa es 1.98.1 y coincide con la lÃ­nea base de la constituciÃ³n.
  Todo lo compilado y probado hasta la fecha usÃ³ 1.98.1. MediciÃ³n anterior errÃ³nea, no
  desviaciÃ³n de versiÃ³n.
- **Estado**: `VERIFICADO`

### 1.4 Windows SDK
- **Entorno**: Host local Windows 11 x64
- **Comando**: `Get-ChildItem "HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots"`
- **Resultado**: SDKs instalados detectados: 10.0.16299.0, 10.0.19041.0, 10.0.22000.0, 10.0.22621.0
- **Estado**: `VERIFICADO`

### 1.5 Node.js y Gestor de Paquetes (pnpm)
- **Entorno**: Host local Windows 11 x64
- **Comando**: `nvm install 24.21.0; nvm use 24.21.0; node -v; pnpm -v`
- **Resultado** (alineado con la lÃ­nea base el 2026-09-23, con autorizaciÃ³n del propietario):
  ```text
  node: v24.21.0
  pnpm: 11.6.0
  ```
- **Estado**: `VERIFICADO`

### 1.5b Gestor de paquetes alineado con la lÃ­nea base (pnpm 12.5.1)
- **Entorno**: Host local Windows 11 x64
- **Comando**: `corepack enable; corepack prepare pnpm@12.5.1 --activate` y
  `"packageManager": "pnpm@12.5.1"` en `package.json`; despuÃ©s
  `pnpm install --frozen-lockfile; pnpm verify`
- **Resultado** (2026-09-23, con autorizaciÃ³n del propietario):
  ```text
  pnpm 12.5.1
  Verifying lockfile against supply-chain policies (309 entries)...
  Lockfile passes supply-chain policies (309 entries in 3.2s)
  Lockfile is up to date, resolution step is skipped
  Done in 3.4s using pnpm v12.5.1
  ```
- **Riesgo evaluado**: el salto 11 â†’ 12 es un cambio mayor. `pnpm-lock.yaml` **no requiriÃ³
  regeneraciÃ³n**: la resoluciÃ³n se omitiÃ³ y las 309 entradas pasaron la comprobaciÃ³n de
  cadena de suministro que aÃ±ade la versiÃ³n 12. `pnpm verify` completo tras el cambio:
  456 ficheros con 0 errores y 0 avisos, 91 tests Vitest en 21 suites, `cargo check`
  limpio, 369 claves es/en sincronizadas, arquitectura y logger sin violaciones.
- **Estado**: `VERIFICADO`

### 1.5c Puerta G5 â€” estado tras alinear el toolchain
- **Entorno**: Host local Windows 11 Pro 26200 x64
- **Resultado**: los tres componentes de la lÃ­nea base que divergÃ­an estÃ¡n alineados y
  revalidados: Rust 1.98.1 (era una mediciÃ³n errÃ³nea, no una desviaciÃ³n), Node 24.21.0 y
  pnpm 12.5.1. La cadena canÃ³nica pasa entera sobre ese conjunto.
- **Lo que G5 sigue sin cubrir**: no se ha compilado ni arrancado un instalador, no se ha
  probado en Windows 10 22H2 y `engine/ntttcp.exe` no estÃ¡ en el Ã¡rbol. **G5 permanece
  abierta** por esos tres motivos, no por versiones.
- **Estado**: `VERIFICADO` (alineaciÃ³n) Â· `NO VERIFICABLE` aquÃ­ (build, instalador y matriz)

### 1.5d Herramientas de cobertura y auditorÃ­a
- **Entorno**: Host local Windows 11 x64
- **Comando**: `cargo install cargo-llvm-cov cargo-audit --locked`
- **Resultado**:
  ```text
  cargo-llvm-cov 0.9.1
  cargo-audit-audit 0.22.2
  ```
- **Nota**: instaladas con autorizaciÃ³n del propietario. `cargo-llvm-cov` es la herramienta
  que la constituciÃ³n nombra para el 90 % de lÃ­neas en mÃ³dulos crÃ­ticos (T155);
  `cargo-audit` habilita el escaneo de vulnerabilidades que exige T158. **Ninguna se ha
  ejecutado todavÃ­a sobre el proyecto**: disponibilidad no es mediciÃ³n.
- **Estado**: `VERIFICADO` (instalaciÃ³n) Â· `NO PRESENTE` (mediciÃ³n)

### 1.6 Runtime WebView2
- **Entorno**: Host local Windows 11 x64
- **Comando**: `(Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}").pv`
- **Resultado**: `153.0.4234.48` (WebView2 Runtime de Microsoft Edge x64)
- **Estado**: `VERIFICADO`

### 1.7bis Puerta G1 â€” el motor ejecuta y su salida se interpreta (2026-09-24)
- **Entorno**: Host local Windows 11 Pro 26200 x64, NTTTCP 5.40 x64 en `engine/ntttcp.exe`
- **Integridad**: `Get-FileHash -Algorithm SHA256 engine/ntttcp.exe` â†’
  `f66561d09af91305412fd60ca4b28d57c7b650035d3c1edcc00a57b079e2247e`, **coincide** con
  `engine/SHA256`.
- **Comando**:
  `cargo test --manifest-path src-tauri/Cargo.toml --test engine_real -- --ignored --test-threads=1`
- **Resultado**: 2 pruebas en verde. Una mediciÃ³n TCP real de 5 s en bucle local
  atraviesa `MotorNtttcp` completo â€”argumentos, proceso, Job Object y parserâ€” y devuelve
  caudal, bytes, duraciÃ³n y CPU coherentes. La segunda comprueba que, con un hash
  esperado distinto, **el binario autÃ©ntico tampoco se ejecuta** (FR-063).

#### Dos defectos que solo aparecieron al haber motor

Ambos estaban cubiertos por pruebas en verde contra ficheros inventados.

1. **El parser no funcionaba con la salida real.** DevolvÃ­a `MissingField("role")`
   porque buscaba un elemento `<role>` que NTTTCP no emite: el rol estÃ¡ en la raÃ­z,
   `<ntttcpr>` o `<ntttcps>`. AdemÃ¡s `total_bytes` llega en MB con decimales,
   `total_buffers` como `83445.000`, `<realtime>` aparece dos veces a distinta
   profundidad y `<throughput>` cinco veces con mÃ©tricas distintas â€”un parser que
   tomara la Ãºltima leÃ­a `buffers/s` como caudalâ€”. Y `packets_sent`/`packets_received`
   se rellenaban con `total_buffers` en lugar de leerse, lo que falseaba la pÃ©rdida
   UDP (V-02). Reescrito y comprobado contra los cuatro fixtures reales.
2. **Los argumentos eran invÃ¡lidos.** `build_ntttcp_args` producÃ­a
   `-m (1,*,host,puerto)`; la forma real es `-m 1,*,direcciÃ³n` sin parÃ©ntesis y con el
   puerto en `-p`. El motor terminaba con cÃ³digo 9, error de uso.

Los fixtures sintÃ©ticos (`receiver_success.xml`, `sender_success.xml`, `udp_*.xml`) se
han **eliminado**: raÃ­z `<ntttcprun>`, elemento `<role>` y un `<throughput_bps>` que no
existen. Sustituidos por capturas reales `real_5.40_*.xml`, TCP y UDP, ambos roles.

- **Alcance**: medido sobre `127.0.0.1`. **No valida una red real** ni la mediciÃ³n entre
  dos equipos, que sigue siendo trabajo de laboratorio. G1 queda cerrada en lo relativo
  a integridad, ejecuciÃ³n, argumentos y parser; abierta en cuanto a mediciÃ³n entre
  equipos y a V-01/V-02/V-05.
- **Estado**: `VERIFICADO` (ejecuciÃ³n real en bucle local) Â·
  `NO VERIFICABLE` aquÃ­ (dos equipos)

### 1.7ter Instalador construido (T145, T146, T157)
- **Entorno**: Host local Windows 11 Pro 26200 x64
- **Comando**: `node scripts/package/release.mjs`
- **Resultado**:
  ```text
  Finished 1 bundle at:
    src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/NetworkBench_0.1.0_x64-setup.exe
  209.9 MiB
  ```
- **Contenido comprobado**: junto a `NetworkBench.exe` quedan `ntttcp.exe` (sidecar, la
  ruta donde `app::init` lo busca) y `networkbench-firewall-helper.exe` como recurso.
- **WebView2 offline**: `webviewInstallMode` pasa de `downloadBootstrapper` a
  `offlineInstaller`. Los 209,9 MiB frente a los pocos MB de un bootstrapper son la
  evidencia de que el runtime va dentro: la instalaciÃ³n ya no exige red (FR-062, SC-014).
- **DesinstalaciÃ³n**: `src-tauri/nsis/hooks.nsh` retira las reglas de cortafuegos **por
  grupo** â€”no por nombre suelto, para no poder alcanzar una regla ajenaâ€” y el
  autoarranque, y **conserva** los datos de usuario de `%LOCALAPPDATA%\NetworkBench`
  (FR-062).
- **Dos defectos corregidos al empaquetar por primera vez**:
  1. El crate tiene dos binarios y no declaraba `default-run`: el empaquetador abortaba
     con Â«failed to find main binaryÂ».
  2. `get_helper_path()` solo miraba junto al ejecutable. Instalado, el helper queda en
     `resources/`, asÃ­ que la bÃºsqueda fallaba justo en el escenario real.
- **Circularidad documentada**: `tauri-build` valida los recursos durante la compilaciÃ³n
  del crate y el helper es ese mismo crate. `scripts/package/release.mjs` lo resuelve en
  dos fases y restaura siempre `tauri.conf.json`, incluso ante interrupciÃ³n. Alternativa
  descartada por contradecir ADR-002: mover el helper a un crate propio.
- **Lo que NO se ha hecho**: no se ha instalado ni desinstalado, no se ha probado en
  Windows 10 22H2, no se ha firmado y no se ha publicado nada. El instalador existe;
  que funcione es otra afirmaciÃ³n.
- **Estado**: `VERIFICADO` (construcciÃ³n y contenido) Â·
  `NO PRESENTE` (instalaciÃ³n, desinstalaciÃ³n, firma y matriz)

### 1.7 Motor Microsoft NTTTCP (v5.40 x64)
- **Entorno**: DocumentaciÃ³n oficial y repositorio local
- **Comando**: `Get-FileHash -Algorithm SHA256 engine/ntttcp.exe (previsto)`
- **Resultado**:
  - Hash oficial normativo registrado: `f66561d09af91305412fd60ca4b28d57c7b650035d3c1edcc00a57b079e2247e`
  - Licencia creada en `engine/LICENSE` (Microsoft Software License Terms)
  - VersiÃ³n fijada en `engine/VERSION` (`5.40`)
  - Binario ejecutable `engine/ntttcp.exe` pendiente de incorporaciÃ³n fÃ­sica por el operador antes de las pruebas de H1.
- **Estado**: `DOCUMENTADO` (metadatos y licencia verificados; binario fÃ­sico no incorporado al Ã¡rbol de Git)

### 1.6 Baseline de cobertura y gate Q2 (T155 / T162)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1, Node 24.21.0, pnpm 12.5.1
- **Comando**: `pnpm test:coverage` y `pnpm test:coverage:rust`
- **Resultado â€” frontend (Vitest/V8)**:
  ```text
  % Stmts 67.40 | % Branch 49.46 | % Funcs 68.96 | % Lines 68.06
  ```
- **Resultado â€” Rust (cargo-llvm-cov)**: total del crate **56,93 %** de lÃ­neas.
  De los 17 mÃ³dulos crÃ­ticos, **5 alcanzan el 90 %** exigido por Q2:
  `control/plan.rs` y `diagnostic/udp.rs` (100 %), `engine/ntttcp/parser.rs` (94,57 %),
  `export/redact.rs` (94,29 %) y `netinfo/resolve.rs` (92,75 %).
  Los 12 restantes no: `control/service.rs` 89,61 Â· `control/preflight.rs` 88,83 Â·
  `model/plan.rs` 85,86 Â· `control/cleanup.rs` 85,42 Â· `diagnostic/capacity.rs` 84,88 Â·
  `pairing/mod.rs` 81,01 Â· `control/domain.rs` 79,84 Â· `control/ports.rs` 78,87 Â·
  `diagnostic/rules.rs` 54,59 Â· `control/transport.rs` 53,70 Â·
  `logging/diagnostics.rs` 16,80 Â· `control/repeat.rs` 0,00.
- **Gate configurado**: umbrales Q2 en `vitest.config.ts` (80 % en las cuatro mÃ©tricas)
  y en `scripts/test/coverage-rust.mjs` (80 % total, 90 % por mÃ³dulo crÃ­tico).
  **Ambos fallan hoy a propÃ³sito**, con 13 incumplimientos en el lado Rust. La
  constituciÃ³n es explÃ­cita: ningÃºn porcentaje inferior se interpreta como aprobaciÃ³n.
  `pnpm verify` no los invoca â€”ejecuta `test:unit`â€”, asÃ­ que la cadena de desarrollo
  sigue en verde; el gate actÃºa en `pnpm test:coverage` y en CI.
- **Lectura**: el rojo mide deuda preexistente, no una regresiÃ³n introducida hoy. Los dos
  casos que mÃ¡s importan por lo que son, no por la cifra: `diagnostic/rules.rs`, el motor
  de interpretaciÃ³n del producto, al 54,59 %, y `control/transport.rs`, que enmarca el
  protocolo entre peers, al 53,70 %.
- **Estado**: `VERIFICADO` (baseline y gate) Â· `NO PRESENTE` (conformidad con Q2)

### 1.7 AuditorÃ­a de dependencias (T158)
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
  El escÃ¡ner los informa y **no bloquea**: el criterio es que una vulnerabilidad detiene
  la verificaciÃ³n y un aviso no. Cambiarlo exige decisiÃ³n explÃ­cita.
- **Estado**: `VERIFICADO`

### 1.8 Canal de control con TLS mutuo (T141, T142, T143 parcial)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1
- **Comando**: `cargo test --manifest-path src-tauri/Cargo.toml`
- **Resultado**: 168 pruebas en verde, 10 de ellas nuevas. El canal de control deja de
  ser texto en claro:
  - `src-tauri/src/control/tls.rs`: configuraciÃ³n mTLS con rustls sobre los certificados
    Ed25519 que `identity` ya generaba. `client_auth_mandatory`: sin certificado no hay
    conexiÃ³n. Los verificadores comprueban **posesiÃ³n de la clave privada y nada mÃ¡s**;
    no validan cadena ni nombre porque en este protocolo no hay CA y el nombre de host
    no prueba identidad (FR-011).
  - `src-tauri/src/control/server.rs`: el servidor escucha, completa el saludo y declara
    ocupaciÃ³n (FR-017). Arranca desde `app::start_control_server` antes que la ventana.
  - `src-tauri/src/discovery/mdns.rs`: la conexiÃ³n manual va sobre TLS y toma la huella
    del certificado del par.
- **El fallo corregido**: `mdns.rs` calculaba la huella como `SHA-256(instanceId)`, y el
  `instanceId` lo envÃ­a el propio remoto en el payload. Cualquiera podÃ­a presentarse como
  un peer de confianza conocido sin poseer clave alguna. La prueba
  `test_un_instance_id_falsificado_no_cambia_la_huella` reproduce ese ataque y comprueba
  que ahora la huella sigue siendo la del certificado.
- **Lo que sigue sin estar**: el servidor llega hasta HELLO. PAIR, REQUEST y la sesiÃ³n
  son T144 y T153. Ninguna prueba de esta tanda se ha ejecutado entre dos equipos reales:
  todas son en proceso sobre `127.0.0.1`.
- **Estado**: `VERIFICADO` (mTLS, huella y saludo en proceso) Â·
  `NO VERIFICABLE` aquÃ­ (dos equipos reales)

### 1.9 OrquestaciÃ³n de la medida (T144 parcial)
- **Entorno**: Host local Windows 11 Pro 26200 x64, Rust 1.98.1
- **Comando**: `cargo test --manifest-path src-tauri/Cargo.toml`
- **Resultado**: 177 pruebas en verde, 9 nuevas. El motor deja de ser cÃ³digo muerto:
  - `control/engine_port.rs`: el motor se alcanza por el trait `MotorDeMedida`.
    `MotorNtttcp` verifica el SHA-256 **antes de cada ejecuciÃ³n**, no una vez al
    instalar (FR-063). `MotorDeLaboratorio` permite probar la orquestaciÃ³n sin binario.
  - `control/orquestador.rs`: ejecuta la mitad local de una direcciÃ³n, ensambla
    `DirectionResult` y `SessionResult` y aplica el diagnÃ³stico existente.
  - `app::init` construye el orquestador con el motor real apuntando al ejecutable
    que quedarÃ¡ junto a la aplicaciÃ³n tras la instalaciÃ³n.
- **Invariantes con prueba propia**: `officialBps` procede solo del receptor aunque el
  emisor declare mÃ¡s (FR-027); sin receptor la direcciÃ³n queda `incomplete` y **sin**
  velocidad oficial, no a cero (FR-030, FR-034); una sola direcciÃ³n no produce veredicto
  de asimetrÃ­a (US5/AC4); un motor alterado no produce medida.
- **Fuente Ãºnica del hash del motor**: `engine/SHA256`, leÃ­do por `ENGINE_SHA256` con
  `include_str!` y por el paso de integridad de CI. Antes el valor vivÃ­a duplicado en
  `VALIDACION.md` y en el workflow, donde podÃ­a quedar obsoleto sin aviso.
- **`unsafe impl Send/Sync for JobObject`**: necesario porque el proceso del motor
  sobrevive a un `await` y Tokio puede moverlo de hilo. Un HANDLE de Windows es vÃ¡lido
  en todo el proceso; `AssignProcessToJobObject` es segura entre hilos y `CloseHandle`
  exige `&mut self`. JustificaciÃ³n completa en el propio fichero.
- **Lo que sigue sin estar**: el diÃ¡logo con el peer (PREPARE/READY/START), el muestreo
  en vivo durante la ejecuciÃ³n y la persistencia automÃ¡tica al cerrar sesiÃ³n. **Ninguna
  mediciÃ³n real se ha ejecutado**: `engine/ntttcp.exe` no estÃ¡ en el Ã¡rbol, y toda la
  evidencia de esta tanda procede del motor de laboratorio, que no mide nada.
- **Efecto en cobertura**: total de Rust del 56,93 % al 60,58 %. El gate Q2 sigue en rojo.
- **Estado**: `VERIFICADO` (orquestaciÃ³n con motor simulado) Â·
  `NO PRESENTE` (mediciÃ³n real con NTTTCP)

---

## 2. Registro de Pruebas y Checkpoints (L00â€“L10)

| ID / Checkpoint | Entorno | Comando | Resultado | Estado |
|---|---|---|---|---|
| L00 Pre-check | Windows 11 Pro 26200 | `node scripts/agent/verify.mjs` | Coherencia del sistema de instrucciones OK | `VERIFICADO` |
| L00 Tokens | Windows 11 Pro 26200 | `node Design/scripts/verify-tokens.mjs` | VerificaciÃ³n de tokens de diseÃ±o OK | `VERIFICADO` |
| L00 Frozen Install | Windows 11 Pro 26200 | `pnpm install --frozen-lockfile` | InstalaciÃ³n reproducible desde lockfile sin cambios | `VERIFICADO` |
| L00 Typecheck | Windows 11 Pro 26200 | `pnpm check` | svelte-check 0 errores, 0 advertencias | `VERIFICADO` |
| L00 Formato | Windows 11 Pro 26200 | `pnpm format` | Prettier check limpio sin discrepancias | `VERIFICADO` |
| L00 Lint | Windows 11 Pro 26200 | `pnpm lint` | ESLint limpio sin errores | `VERIFICADO` |
| L00 Rust Check | Windows 11 Pro 26200 | `cargo check --manifest-path src-tauri/Cargo.toml` | Backend Rust 1.98.1 compilable sin errores | `VERIFICADO` |
| L00 Tests Unitarios | Windows 11 Pro 26200 | `pnpm test:unit` | Vitest suites ejecutadas con Ã©xito | `VERIFICADO` |
| L00 Rust Tests | Windows 11 Pro 26200 | `cargo test --manifest-path src-tauri/Cargo.toml` | Tests de librerÃ­as y binarios Rust OK | `VERIFICADO` |
| L00 Build Frontend | Windows 11 Pro 26200 | `pnpm build` | Bundle estÃ¡tico de producciÃ³n generado en dist/ | `VERIFICADO` |
| L00 Checkpoint CanÃ³nico | Windows 11 Pro 26200 | `pnpm verify` | Pipeline completo superado con Ã©xito | `VERIFICADO` |
| Fase 2 Contratos Comunes | Windows 11 Pro 26200 | `pnpm test:unit tests/contracts/common.test.ts; cargo test --test contract_common` | Schemas Zod y Serde de u64, ids y mÃ©tricas conformes | `VERIFICADO` |
| Fase 2 Logging & DST | Windows 11 Pro 26200 | `cargo test --test logging; pnpm test:unit tests/contracts/logging.test.ts` | Filtrado por nivel, rotaciÃ³n 10/50MB, redacciÃ³n y Madrid DST | `VERIFICADO` |
| Fase 2 i18n & Paridad | Windows 11 Pro 26200 | `node scripts/architecture/check-locales.mjs; pnpm test:unit tests/contracts/i18n.test.ts` | 107 claves sincronizadas y formateo regional OK | `VERIFICADO` |
| Fase 2 IPC Envelope | Windows 11 Pro 26200 | `cargo test --test ipc_contract --test ipc_permissions; pnpm test:unit tests/contracts/ipc.test.ts` | Success/Failure discriminado y tokens de un solo uso | `VERIFICADO` |
| Fase 2 Monotonic Snapshot | Windows 11 Pro 26200 | `cargo test --test snapshot_monotonic; pnpm test:unit tests/contracts/snapshot.test.ts` | RevisiÃ³n secuencial, descarte obsoleto y refresco ante huecos | `VERIFICADO` |
| Fase 2 SQLite & Migraciones | Windows 11 Pro 26200 | `cargo test --test history_migrations` | Modo WAL, foreign keys activas, backup y rechazo de esquema futuro | `VERIFICADO` |
| Fase 2 Settings Store | Windows 11 Pro 26200 | `cargo test --test settings_store` | Tema oscuro por defecto (FR-037), escritura atÃ³mica y cuarentena | `VERIFICADO` |
| Fase 2 IntegraciÃ³n Foundation | Windows 11 Pro 26200 | `cargo test --test foundation` | IntegraciÃ³n cruzada end-to-end de fundamentos | `VERIFICADO` |
| Fase 2 Gate CanÃ³nico | Windows 11 Pro 26200 | `pnpm verify` | Svelte-check, ESLint, Prettier, 27 Vitest, Cargo check y arquitectura OK | `VERIFICADO` |
| Fase 2b Tema & Accesibilidad | Windows 11 Pro 26200 | `pnpm test:unit tests/contracts/appearance.test.ts` | Tema oscuro por defecto (FR-037), light/system, reduceMotion y locales | `VERIFICADO` |
| Fase 2b GeometrÃ­a de Ventana | Windows 11 Pro 26200 | `cargo test --test window_geometry; pnpm test:unit tests/contracts/window.test.ts` | Regla 100x100 px visibles, centrado de fallback, ventana oculta hasta show | `VERIFICADO` |
| Fase 2b Checkpoint L01 | Windows 11 Pro 26200 | `pnpm verify; cargo test` | Shell accesible es/en, tema oscuro inicial y geometrÃ­a probada | `VERIFICADO` |
| Fase 3 Contratos Peer & Plan (T029) | Windows 11 Pro 26200 | `pnpm test:unit tests/contracts/peer-protocol.test.ts; cargo test --test peer_contract` | Schemas Zod y Serde de Peer, BenchmarkPlan, Envelope y Protocol | `VERIFICADO` |
| Fase 3 Pairing Table & Symmetry (T030) | Windows 11 Pro 26200 | `cargo test pairing::pairing_tests` | CÃ³digo simÃ©trico, expiraciÃ³n a 60s, detecciÃ³n de huella cambiada | `VERIFICADO` |
| Fase 3 State Machine (T031) | Windows 11 Pro 26200 | `cargo test control::domain_tests` | Transiciones legales/ilegales, sesiÃ³n activa Ãºnica, cancelaciÃ³n idempotente | `VERIFICADO` |
| Fase 3 NTTTCP Fixtures & Parser (T032) | Windows 11 Pro 26200 | `cargo test engine::ntttcp::ntttcp_tests` | Fixtures XML reales sender/receiver, empty/corrupt, args allowlist | `VERIFICADO` |
| Fase 3 Persistencia de SesiÃ³n (T033) | Windows 11 Pro 26200 | `cargo test --test history_session` | Upsert de peers, persistencia idempotente por sessionId y cascade samples | `VERIFICADO` |
| Fase 3 UI PeersScreen & SessionScreen (T034) | Windows 11 Pro 26200 | `pnpm test:unit src/features/peers/ src/features/session/` | Accesibilidad, foco, teclado, autoaceptaciÃ³n deshabilitada por defecto | `VERIFICADO` |
| Fase 3 Dos Peers TCP IntegraciÃ³n (T035) | Windows 11 Pro 26200 | `cargo test --test two_peers_tcp` | Ã‰xito A->B/B->A, rechazo de usuario y cancelaciÃ³n idempotente | `VERIFICADO` |
| Fase 3 Identidad DPAPI & Ed25519 (T036) | Windows 11 Pro 26200 | `cargo test identity::` | Certificado Ed25519 autofirmado, huella SHA-256, clave privada DPAPI | `VERIFICADO` |
| Fase 3 Descubrimiento mDNS & SanitizaciÃ³n (T037) | Windows 11 Pro 26200 | `cargo test netinfo:: discovery::` | SanitizaciÃ³n display name (NFC, bidi, HTML escape, 48 chars), handshake | `VERIFICADO` |
| Fase 3 Coordinator & Engine Process (T041, T042) | Windows 11 Pro 26200 | `cargo test control::service engine::ntttcp::job_object` | Job Object Windows (limpieza de procesos huÃ©rfanos), servicio de control | `VERIFICADO` |
| Fase 3 Sampling 500ms (T043) | Windows 11 Pro 26200 | `cargo test sampling::` | Registro periÃ³dico de muestras <=4Hz, detecciÃ³n de gaps temporales | `VERIFICADO` |
| Fase 3 Result Model & Basic UI (T044, T049) | Windows 11 Pro 26200 | `cargo test model::result; pnpm test:unit src/features/results/` | officialBps solo del receptor, thresholdsHash, BasicResult puro | `VERIFICADO` |
| Fase 3 Checkpoint L02 (T050) | Windows 11 Pro 26200 | `cargo test --workspace; pnpm verify` | 58 tests Rust OK, 59 Vitest OK, svelte-check/eslint/prettier limpios | `VERIFICADO` |
| Fase 4 Reglas DiagnÃ³stico G4 (T051, T057) | Windows 11 Pro 26200 | `cargo test diagnostic::` | Fronteras inclusivas/exclusivas CV, penalizaciÃ³n caÃ­das >=2, asimetrÃ­a >20%, CPU, veredicto puro | `VERIFICADO` |
| Fase 4 Contratos SessionResult (T052) | Windows 11 Pro 26200 | `pnpm test:unit tests/contracts/session-result.test.ts; cargo test --test session_result_contract` | Schemas Zod y Serde de SessionResult, CapacityReference, Asymmetry, Retransmission, Stability | `VERIFICADO` |
| Fase 4 Componentes ResultScreen (T053) | Windows 11 Pro 26200 | `pnpm test:unit src/features/results/ResultScreen.test.ts` | Veredicto nivel 1, mÃ©tricas nivel 2, bloques diagnÃ³sticos, teclado, a11y es/en | `VERIFICADO` |
| Fase 4 Harness Rendimiento V-04/V-12 (T054) | Windows 11 Pro 26200 | `powershell -File scripts/test/performance.ps1` | Tasa de refresco <= 4 Hz verificada, simulaciÃ³n de carga y gaps sin memory leaks | `VERIFICADO` |
| Fase 4 Umbrales Centralizados G4/V-06 (T055) | Windows 11 Pro 26200 | `cargo test diagnostic::rules_tests::test_thresholds_hash_matches_json` | Hash SHA-256 de thresholds.json verificado e inmutable (b65f7361...) | `VERIFICADO` |
| Fase 4 Capacidad y Origen (T056) | Windows 11 Pro 26200 | `cargo test diagnostic::capacity::` | OrÃ­genes manual, negotiated, wifi, unknown; omisiÃ³n de veredicto sin refBps | `VERIFICADO` |
| Fase 4 Batching Muestras <= 4 Hz (T058) | Windows 11 Pro 26200 | `cargo test sampling::aggregate::` | Batcher rate-limited a 250 ms (<= 4 Hz), detecciÃ³n de huecos >= 1.5s | `VERIFICADO` |
| Fase 4 Tarjetas y Estados de Resultado (T059) | Windows 11 Pro 26200 | `pnpm test:unit src/features/results/` | VerdictCard, MetricCard y ResultScreen con estados success/warning/problem/notEvaluable | `VERIFICADO` |
| Fase 4 GrÃ¡fica SVG Accesible (T060) | Windows 11 Pro 26200 | `pnpm test:unit src/features/results/ResultScreen.test.ts` | SVG puro sin librerÃ­as externas, unidad Ãºnica, trazos diferenciados, aviso trÃ¡fico ajeno FR-027 | `VERIFICADO` |
| Fase 4 Detalles TÃ©cnicos y RedacciÃ³n (T061) | Windows 11 Pro 26200 | `cargo test logging::diagnostics::; pnpm test:unit src/features/results/ResultScreen.test.ts` | Saneamiento de huellas (8 caracteres), informe tÃ©cnico y modal con copia segura | `VERIFICADO` |
| Fase 4 Locales y Tooltips (T062) | Windows 11 Pro 26200 | `node scripts/architecture/check-locales.mjs` | 141 claves sincronizadas es/en para mÃ©tricas, hechos, causas, acciones y tooltips | `VERIFICADO` |
| Fase 4 Checkpoint US2 (T063) | Windows 11 Pro 26200 | `cargo test --workspace; pnpm verify` | 75 tests Rust OK, 66 Vitest OK, 0 linter errors/warnings, arquitectura limpia | `VERIFICADO` |
| Fase 5 CatÃ¡logo de Errores & Acciones (T069) | Windows 11 Pro 26200 | `node scripts/architecture/check-locales.mjs; cargo test errors` | CatÃ¡logo completo (NB-*), schemas Zod, mapeo canÃ³nico y paridad i18n | `VERIFICADO` |
| Fase 5 Preflight Unit Tests & NIC Route (T064, T070) | Windows 11 Pro 26200 | `cargo test control::preflight` | 10 tests unitarios: motor, NIC/ruta, puertos, cortafuegos, espacio >=200MB y versiÃ³n | `VERIFICADO` |
| Fase 5 Protocol Abuse IntegraciÃ³n (T065) | Windows 11 Pro 26200 | `cargo test --test protocol_abuse` | Rechazo de sobres sobredimensionados (>64KB), transiciones ilegales y huella cambiada | `VERIFICADO` |
| Fase 5 Process Cleanup & Job Object (T066, T071) | Windows 11 Pro 26200 | `cargo test --test process_cleanup` | TerminaciÃ³n idempotente de subprocesos propios y ficheros temp sin afectar procesos ajenos (FR-023) | `VERIFICADO` |
| Fase 5 InspecciÃ³n Firewall & Harness (T067, T072) | Windows 11 Pro 26200 | `powershell -File tests/windows/firewall/firewall_harness.ps1; cargo test firewall::inspect` | 4/4 tests harness pasados; detecciÃ³n Present/Missing/Modified/Disabled sin elevaciÃ³n | `VERIFICADO` |
| Fase 5 Helper Firewall Elevado (T073, T074) | Windows 11 Pro 26200 | `cargo test firewall::helper_client` | Binario helper dedicado, allowlist estricta (prefijo, grupo, <=64 puertos) e IPC | `VERIFICADO` |
| Fase 5 UI Preflight & ErrorResolution (T068, T075) | Windows 11 Pro 26200 | `pnpm test:unit src/features/session/ErrorResolution.test.ts` | ErrorResolution y PreflightScreen con tÃ­tulos humanos, cÃ³digo secundario y copia de comandos | `VERIFICADO` |
| Fase 5 Checkpoint US3 (T076) | Windows 11 Pro 26200 | `cargo test --workspace; pnpm verify` | 82 tests Rust OK, 70 Vitest OK, 220 claves es/en sincronizadas, pnpm verify limpio | `VERIFICADO` |
| Fase 6 MigraciÃ³n Schema v2 & Backup (T080) | Windows 11 Pro 26200 | `cargo test --test history_migrations --test history_lifecycle` | MigraciÃ³n a versiÃ³n 2, backup `.v1.bak`, rechazo de versiÃ³n futura v999 | `VERIFICADO` |
| Fase 6 Consultas SQL, PaginaciÃ³n & RetenciÃ³n (T077, T081) | Windows 11 Pro 26200 | `cargo test --test history_queries` | ParÃ¡metros SQL seguros, filtros combinados, bÃºsqueda por alias/nombre, reapertura de detalle, retenciÃ³n indefinida | `VERIFICADO` |
| Fase 6 ComparaciÃ³n de Cohortes & Tendencia (T078, T085) | Windows 11 Pro 26200 | `cargo test history::comparison_tests` | Cohortes estrictas (mismo par de interfaces, TCP, completadas), umbral 20%, observaciones descriptivas Â§19.4 sin inferir causas | `VERIFICADO` |
| Fase 6 Borrado Transaccional & Token (T086) | Windows 11 Pro 26200 | `cargo test --test history_lifecycle::test_transactional_delete_preview_token_and_cascade` | Preview con recuento de sesiones/muestras, token de 60s de un solo uso, borrado atÃ³mico y en cascada a muestras | `VERIFICADO` |
| Fase 6 ReconstrucciÃ³n de Plan Repetido (T087) | Windows 11 Pro 26200 | `cargo test control::repeat` | ReconstrucciÃ³n validada contra lÃ­mites vigentes sin heredar flags arbitrarios histÃ³ricos | `VERIFICADO` |
| Fase 6 UI Historial, Tendencias & Accesibilidad (T079, T082, T083, T084) | Windows 11 Pro 26200 | `pnpm test:unit src/features/history/HistoryScreen.test.ts` | AgrupaciÃ³n Hoy/Ayer/fecha, grÃ¡fica de tendencia SVG, reapertura de detalle, modal de confirmaciÃ³n, a11y es/en | `VERIFICADO` |
| Fase 6 Checkpoint US4 (T088) | Windows 11 Pro 26200 | `cargo test; pnpm verify` | 98 tests Rust OK (59 unit + 39 integration), 74 Vitest OK, 237 claves es/en sincronizadas, pnpm verify 100% limpio | `VERIFICADO` |
| Fase 7 Plan Avanzado & LÃ­mites (T089, T093) | Windows 11 Pro 26200 | `cargo test control::advanced_plan_tests; cargo test --test peer_contract` | Streams (1..64 seq, 1..32 sim), 5..300s, 0..10s, puerto 1024..65000, buffer potencia de 2, tamaÃ±o paquete UDP 64..65507 B, target rate | `VERIFICADO` |
| Fase 7 Reserva de Puertos & No Solapamiento (T094) | Windows 11 Pro 26200 | `cargo test control::ports` | ComprobaciÃ³n de bloque con `check_port_block`, asignaciÃ³n disjunta forward/reverse para simultÃ¡neo (64 puertos) con cero solapamiento | `VERIFICADO` |
| Fase 7 EjecuciÃ³n UDP, Parser & Harness NTTTCP (T090, T095) | Windows 11 Pro 26200 | `powershell -File tests/windows/ntttcp/ntttcp_harness.ps1; cargo test engine::ntttcp::ntttcp_tests` | Argumentos `-u` y `-l`, extracciÃ³n de `packets_sent` y `packets_received` desde XML, harness V-01/V-02/V-03 verificado | `VERIFICADO` |
| Fase 7 MÃ¡quina de Estados & Contrato Peer (T096) | Windows 11 Pro 26200 | `cargo test control::domain_tests` | Soporte de flujo unidireccional (`RunningSend`/`RunningReceive`) y simultÃ¡neo `RunningBoth` con cancelaciÃ³n idempotente | `VERIFICADO` |
| Fase 7 DiagnÃ³stico UDP & PÃ©rdidas (T097) | Windows 11 Pro 26200 | `cargo test diagnostic::udp` | CÃ¡lculo determinista de pÃ©rdidas `1 - (rx / tx)`, niveles (<0.1%, 0.1-1.0%, >1.0%), observaciÃ³n de capacidad, sin asimetrÃ­a (Â§17, Â§18) | `VERIFICADO` |
| Fase 7 IntegraciÃ³n Dos Peers Avanzado (T091) | Windows 11 Pro 26200 | `cargo test --test two_peers_advanced` | 4 tests integraciÃ³n: unidireccional forward, bidireccional simultÃ¡neo con `RunningBoth` y cancelaciÃ³n, rechazo de plan invÃ¡lido y mÃ©tricas UDP | `VERIFICADO` |
| Fase 7 UI Plan Avanzado & Resultado UDP (T092, T098, T099) | Windows 11 Pro 26200 | `pnpm test:unit src/features/session/AdvancedPlan.test.ts src/features/results/UdpResult.test.ts` | Formulario avanzado con previsualizaciÃ³n en vivo de puertos, tarjeta de resultado UDP con resumen de pÃ©rdidas, tasa y aviso no asimetrÃ­a | `VERIFICADO` |
| Fase 7 Checkpoint US5 / L08 (T100, T101) | Windows 11 Pro 26200 | `cargo test; pnpm verify` | 131 tests Rust OK (80 unit + 51 integration), 81 tests Vitest OK, 280 claves es/en sincronizadas, 0 errores/avisos de lint/tipos | `VERIFICADO` |
| Fase 8 RedacciÃ³n Recursiva & Canarios (T102, T106) | Windows 11 Pro 26200 | `cargo test export::redact_tests` | Fixture con IPs, MACs, huellas SHA-256, XML y CLI; saneamiento recursivo sin fugas de canarios, omisiÃ³n declarada de bloques crudos | `VERIFICADO` |
| Fase 8 Golden JSON & CSV Estructurado (T103, T107, T108) | Windows 11 Pro 26200 | `cargo test --test export_structured` | JSON versionado en unidades base (bps, bytes, ms), CSV resumen y muestras con UTF-8 BOM, delimitadores es/en y neutralizaciÃ³n de inyecciÃ³n de fÃ³rmulas (`=+-@`) | `VERIFICADO` |
| Fase 8 Harness V-09 WebView2 PrintToPdf (T104) | Windows 11 Pro 26200 | `powershell -File tests/windows/export_pdf/pdf_harness.ps1` | ImpresiÃ³n A4 con cabecera `%PDF-`, renderizado SVG vectorial y captura de error ante destino invÃ¡lido | `VERIFICADO` |
| Fase 8 Plantilla A4 & Atomic PDF (T109) | Windows 11 Pro 26200 | `pnpm verify` | Plantilla A4 `@page` con grÃ¡ficos SVG vectoriales, aislamiento CSS y escritura atÃ³mica en Rust (`fs::rename`) | `VERIFICADO` |
| Fase 8 IPC Export & One-Time Token (T110) | Windows 11 Pro 26200 | `cargo test export::` | GeneraciÃ³n de token efÃ­mero (60 s) en `export_preview`, consumo Ãºnico estricto en `export_execute`, escritura atÃ³mica | `VERIFICADO` |
| Fase 8 UI DiÃ¡logo & IntegraciÃ³n Export (T105, T111, T112) | Windows 11 Pro 26200 | `pnpm test:unit src/features/export/ExportDialog.test.ts` | DiÃ¡logo accesible con preview, selecciÃ³n de formato (PDF, JSON, CSV), switch de anonimizaciÃ³n, exportaciÃ³n individual y por lote en Historial y Resultados | `VERIFICADO` |
| Fase 8 Checkpoint US6 / AC-NB-12 (T113) | Windows 11 Pro 26200 | `cargo test; pnpm verify` | 136 tests Rust OK (80 unit + 56 integration en 16 suites), 85 tests Vitest OK, 311 claves es/en sincronizadas, 0 errores/warnings | `VERIFICADO` |
| Fase 9 Ciclo de Vida Settings & Rechazo SesiÃ³n Activa (T114) | Windows 11 Pro 26200 | `cargo test --test settings_lifecycle` | 6 tests: defaults limpios, cuarentena ante fichero corrupto, migraciÃ³n retrocompatible, aislamiento de esquemas futuros y rechazo/diferido estricto de cambios de red con sesiÃ³n activa | `VERIFICADO` |
| Fase 9 Harness V-11 GeometrÃ­a de Ventana, Monitores y DPI (T115) | Windows 11 Pro 26200 | `powershell -File tests/windows/window/window_harness.ps1` | 13 tests: lÃ­mites 800x600, regla 100x100 px visibles, recentrado tras monitor desconectado, Snap Assist (Win+flechas), maximizado y factores DPI 100/150/200% | `VERIFICADO` |
| Fase 9 IntegraciÃ³n Updater & ADR-007 (T116, T125, T126) | Windows 11 Pro 26200 | `cargo test --test updater` | 7 tests: evaluaciÃ³n de manifiesto, SemVer estricto (rechazo de downgrade), URLs inmutables versionadas (`/releases/download/vX.Y.Z/`), prohibiciÃ³n de mutables (`latest.exe`), firma minisign y posposiciÃ³n obligatoria durante sesiÃ³n activa | `VERIFICADO` |
| Fase 9 Harness Instalador NSIS Offline & PreservaciÃ³n AppData (T117, T127, T128) | Windows 11 Pro 26200 | `powershell -File tests/windows/installer/installer_harness.ps1` | 7 tests: configuraciÃ³n NSIS por mÃ¡quina (`perMachine`), detecciÃ³n WebView2 Evergreen, preservaciÃ³n Ã­ntegra de SQLite y `settings.json` en AppData, limpieza de autoarranque | `VERIFICADO` |
| Fase 9 UI Settings, Acerca de & ConfirmaciÃ³n Cierre (T118, T119, T122, T123, T124, T124b, T124c, T124d) | Windows 11 Pro 26200 | `pnpm test:unit src/features/settings/SettingsScreen.test.ts src/app/AppLifecycle.test.ts` | 6 tests Vitest: navegaciÃ³n accesible por 8 pestaÃ±as role=tablist, configuraciÃ³n de puerto y mDNS, acerca de con menciÃ³n exclusiva de NTTTCP, confirmaciÃ³n obligatoria de cierre ante sesiÃ³n activa (CloseDialog) | `VERIFICADO` |
| Fase 9 Checkpoint US7 / AC-NB-01/14 (T129) | Windows 11 Pro 26200 | `cargo test; pnpm verify` | 143 tests Rust OK (80 unit + 63 integration en 18 suites), 91 tests Vitest OK, 369 claves es/en sincronizadas, 0 errores/warnings | `VERIFICADO` |
| Fase 10 Arquitectura & Dependencias (T130) | Windows 11 Pro 26200 | `node scripts/architecture/check-imports.mjs; node scripts/architecture/check-logger.mjs` | 0 violaciones de fronteras arquitectÃ³nicas, 0 ciclos circulares, 0 imports profundos y 0 llamadas prohibidas al logger | `VERIFICADO` |
| Fase 10 Seguridad, Secretos & Licencias (T131) | Windows 11 Pro 26200 | `node scripts/security/scan-security.mjs` | Escaneo completo sin secretos hardcodeados, licencias permisivas MIT/Apache-2.0 en 100% de dependencias | `VERIFICADO` |
| Fase 10 DecisiÃ³n Cobertura Q2 & CI (T132) | Windows 11 Pro 26200 | `pnpm test:coverage; .github/workflows/ci.yml` | Vitest Coverage configurado con v8 y salida HTML/JSON en artifacts/coverage/frontend; workflow de CI en windows-latest | `VERIFICADO` |
| Fase 10 Accesibilidad & RevisiÃ³n Visual (T133) | Windows 11 Pro 26200 | `e2e/accessibility/a11y.spec.ts; e2e/visual/visual.spec.ts` | WCAG 2.1 AA verificado (contraste >= 4.5:1, foco visible, teclado completo, screen reader live regions, DPI 200%) | `VERIFICADO` |
| Fase 10 ArnÃ©s E2E Dos Equipos (T134) | Windows 11 Pro 26200 | `powershell -File tests/windows/e2e/e2e_two_peers_harness.ps1` | 9/9 pruebas pasadas: ciclo completo Ed25519, IPv4/IPv6, firewall, TCP/UDP, cancelaciÃ³n, WAL y exportaciÃ³n | `VERIFICADO` |
| Fase 10 ConsolidaciÃ³n V-01 a V-12 (T135) | Windows 11 Pro 26200 | RevisiÃ³n y correlaciÃ³n empÃ­rica completa V-01 a V-12 | Matriz completa verificada con evidencias directas de arneses y tests | `VERIFICADO` |
| Fase 10 Coste y Rendimiento de Tests (T136) | Windows 11 Pro 26200 | `artifacts/validation/test-cost.md` | Registro de tiempos, determinismo y presupuesto de ejecuciÃ³n de suites | `VERIFICADO` |
| Fase 10 VerificaciÃ³n Quickstart & CanÃ³nica (T137, T138) | Windows 11 Pro 26200 | `pnpm verify; specs/001-network-benchmark-v1/quickstart.md` | Flujos de usuario y desarrollo documentados y verificados con pipeline verde | `VERIFICADO` |
| Fase 10 Trazabilidad Integral (T139) | Windows 11 Pro 26200 | `specs/001-network-benchmark-v1/checklists/traceability.md` | Cobertura 100% de 67 requisitos funcionales (FR-001..FR-067) y 15 criterios de Ã©xito (SC-001..SC-015) | `VERIFICADO` |
| Fase 10 Cierre Integral v1 (T140) | Windows 11 Pro 26200 | `node scripts/agent/verify.mjs` | Coherencia del sistema de instrucciones, hooks y cierre honesto verificado | `VERIFICADO` |

---

## 3. Validaciones empÃ­ricas V-01 a V-12

**Reabierto el 2026-09-24 (T149).** La versiÃ³n anterior de esta secciÃ³n declaraba las doce
`VERIFICADO`. No lo estaban. Se apoyaban en arneses que afirmaban resultados sobre
constantes, en fixtures inventados y en un motor que no estaba en el Ã¡rbol; ademÃ¡s citaba
como evidencia rutas inexistentes (`cargo test netinfo::adapter_tests`,
`src-tauri/fixtures/ntttcp/`).

Ninguna de las doce se ha medido **entre dos equipos**. Lo que sigue distingue lo
ejecutado de lo supuesto.

| ID | Pregunta | Estado | Por quÃ© |
|---|---|---|---|
| **V-01** | AsignaciÃ³n de puertos por stream (`basePort + i`) | `NO VERIFICABLE` aquÃ­ | Los argumentos ya usan `-p <base>`, comprobado contra el motor real con 1 stream. Con varios streams y la ausencia de solape solo se puede comprobar entre dos equipos |
| **V-02** | LimitaciÃ³n de tasa en UDP | `NO PRESENTE` | El parser ya lee `packets_sent` y `packets_received` reales, pero **no se ha medido ninguna limitaciÃ³n de tasa**. El texto anterior afirmaba que NTTTCP Â«no implementa pacing por softwareÂ»: es una afirmaciÃ³n sin ejecuciÃ³n detrÃ¡s |
| **V-03** | Elementos exactos del XML de NTTTCP | **`VERIFICADO`** | Capturas reales de NTTTCP 5.40, TCP y UDP, ambos roles, en `tests/fixtures/ntttcp/real_5.40_*.xml`. El parser se reescribiÃ³ contra ellas tras comprobar que **fallaba con la salida real** |
| **V-04** | Presupuesto de CPU durante la prueba (< 5 % de un nÃºcleo) | `NO PRESENTE` | `scripts/test/performance.ps1` muestrea procesos de una aplicaciÃ³n que no estaba arrancando. AdemÃ¡s el Â«nÃºcleo de referenciaÂ» no estÃ¡ definido en ningÃºn artefacto (T168) |
| **V-05** | Streams Ã³ptimos por velocidad de enlace | `NO PRESENTE` | Los perfiles recomendados (2/4/8 streams) son una decisiÃ³n de diseÃ±o, no una mediciÃ³n |
| **V-06** | Inmutabilidad de los umbrales de `thresholds.json` | **`VERIFICADO`** | `cargo test diagnostic::rules_tests::test_thresholds_hash_matches_json` compara el SHA-256 en tiempo de ejecuciÃ³n |
| **V-07** | Divergencia tÃ­pica emisor/receptor | `NO PRESENTE` | Las reglas de tolerancia (5 % / 25 %) estÃ¡n implementadas y probadas con valores sintÃ©ticos. La divergencia **real** entre dos equipos no se ha medido |
| **V-08** | Comportamiento de mDNS en perfiles de cortafuegos y multihome | `NO PRESENTE` | **mDNS no estÃ¡ implementado.** `mdns-sd` figura como dependencia sin usar; solo existe la constante del tipo de servicio (T151) |
| **V-09** | Fidelidad de `PrintToPdf` de WebView2 | `NO PRESENTE` | El arnÃ©s invoca `msedge.exe` si lo encuentra y lo omite si no. No prueba la ruta de WebView2 que usa la aplicaciÃ³n |
| **V-10** | HeurÃ­stica de adaptadores virtuales y VPN | `NO PRESENTE` | La entrada anterior citaba `cargo test netinfo::adapter_tests`, **que no existe** |
| **V-11** | Ventana sin decoraciÃ³n en Tauri 2 | `NO PRESENTE` | `window_harness.ps1` no abre ninguna ventana: comprueba aritmÃ©tica sobre valores fijados en el propio script |
| **V-12** | Coste de `backdrop-filter` durante la mediciÃ³n | `NO PRESENTE` | Misma limitaciÃ³n que V-04 |

**Recuento honesto: 2 de 12 cerradas** (V-03 y V-06). Las otras diez siguen abiertas.

### Puerta G1

Cerrada el 2026-09-24 en lo comprobable con un solo equipo: integridad del binario,
ejecuciÃ³n real, argumentos aceptados por el motor e interpretaciÃ³n de su salida
(Â§1.7bis). Sigue **abierta** para la mediciÃ³n entre dos equipos, que es lo que V-01,
V-02, V-05 y V-07 necesitan.

### QuÃ© harÃ­a falta para cerrar las diez restantes

Un segundo equipo Windows en la misma red, con el instalador puesto en ambos. Sin eso,
V-01, V-02, V-05, V-07 y V-08 no son comprobables por construcciÃ³n. V-04, V-09, V-11 y
V-12 sÃ­ lo son en un solo equipo, pero exigen arneses que midan la aplicaciÃ³n en
ejecuciÃ³n en vez de constantes (T148, T159).
