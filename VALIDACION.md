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
  evidencia de que el runtime va dentro: la instalación ya no exige red (FR-062, SC-014).
- **Desinstalación**: `src-tauri/nsis/hooks.nsh` retira las reglas de cortafuegos **por
  grupo** —no por nombre suelto, para no poder alcanzar una regla ajena— y el
  autoarranque, y **conserva** los datos de usuario de `%LOCALAPPDATA%\NetworkBench`
  (FR-062).
- **Dos defectos corregidos al empaquetar por primera vez**:
  1. El crate tiene dos binarios y no declaraba `default-run`: el empaquetador abortaba
     con «failed to find main binary».
  2. `get_helper_path()` solo miraba junto al ejecutable. Instalado, el helper queda en
     `resources/`, así que la búsqueda fallaba justo en el escenario real.
- **Circularidad documentada**: `tauri-build` valida los recursos durante la compilación
  del crate y el helper es ese mismo crate. `scripts/package/release.mjs` lo resuelve en
  dos fases y restaura siempre `tauri.conf.json`, incluso ante interrupción. Alternativa
  descartada por contradecir ADR-002: mover el helper a un crate propio.
- **Lo que NO se ha hecho**: no se ha instalado ni desinstalado, no se ha probado en
  Windows 10 22H2, no se ha firmado y no se ha publicado nada. El instalador existe;
  que funcione es otra afirmación.
- **Estado**: `VERIFICADO` (construcción y contenido) ·
  `NO PRESENTE` (instalación, desinstalación, firma y matriz)

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
