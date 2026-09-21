# Investigación: NetworkBench v1

**Fecha**: 2026-09-21  
**Constitución**: 0.6.0, no ratificada  
**Estado**: decisiones de diseño resueltas; gates empíricos y normativos conservados

Este documento resuelve las decisiones técnicas necesarias para diseñar la feature. No presenta
como validadas las versiones, el motor, Windows ni el laboratorio: G1–G6 y V-01–V-12 siguen
requiriendo ejecución real.

## R01 — Forma del sistema

**Decisión**: monolito modular de escritorio, con un crate Rust principal, frontend Svelte y
helper elevado separado únicamente para firewall.

**Motivo**: el producto se instala y ejecuta en un equipo, usa recursos locales y solo necesita
comunicarse con un peer. Los límites modulares reducen acoplamiento y permiten trabajo paralelo
sin añadir despliegues o protocolos internos.

**Alternativas consideradas**: microservicios, servicio Windows permanente, varios crates por
feature, Clean Architecture completa. Añaden operación, procesos o capas sin problema actual.

## R02 — Autoridad y estado

**Decisión**: Rust es autoridad de identidad, consentimiento, sesión, plan, procesos, resultado y
preferencias persistidas. Svelte conserva únicamente estado presentacional y formularios aún no
aceptados. Un coordinador serializa los mandatos de la única sesión activa.

**Motivo**: impide que una condición visual autorice efectos y reduce carreras entre red,
usuario, procesos y eventos. Canales acotados separan control/cancelación de muestras descartables.

**Alternativas consideradas**: store global TypeScript como autoridad, locks globales alrededor de
objetos compartidos y framework de actores. Duplican autoridad o añaden abstracción innecesaria.

## R03 — Frontera Tauri

**Decisión**: comandos estrechos por caso de uso, capabilities explícitas por ventana, scopes
mínimos y validación Zod/Serde en ambos lados. `src/lib/api` es el único consumidor directo del
SDK Tauri en frontend.

**Motivo**: la documentación de Tauri define WebView/Rust como frontera de confianza y explica
que las capabilities y la runtime authority deciden si una invocación llega al comando. El propio
comando mantiene autorización de negocio y validación; ACL no las sustituye.

**Alternativas consideradas**: un comando genérico, acceso a shell/filesystem desde la WebView o
confiar solo en el registro de comandos. Amplían la superficie y mezclan permisos con negocio.

**Referencias primarias**:

- https://v2.tauri.app/security/
- https://v2.tauri.app/security/runtime-authority/
- https://v2.tauri.app/reference/config/

## R04 — Contratos TypeScript/Rust

**Decisión**: contratos mantenidos explícitamente: Serde y validadores autoritativos en Rust,
schemas Zod e inferencia de tipos en TypeScript, más fixtures compartidos válidos e inválidos. No
se introduce generación automática en v1.

**Motivo**: existen pocos contratos públicos y la validación runtime es obligatoria. Los
fixtures detectan divergencias sin incorporar un generador, pipeline y formato intermedio.

**Alternativas consideradas**: tipos TypeScript sin validación, JSON Schema generado o generador
Rust→TS. El primero es inseguro; los otros se reconsiderarán si la divergencia repetida lo justifica.

## R05 — Protocolo peer

**Decisión**: TLS 1.3 mutuo, framing length-prefixed, envelope versionado y máquina de estados
explícita. Cada mensaje tiene límites, estados permitidos e idempotencia definida. G2 debe cerrar
el orden exacto de primer contacto, plan y aceptación antes de L02/L03.

**Motivo**: una conexión persistente por sesión cubre coordinación, heartbeat, cancelación y
resultado sin servidor central. Un protocolo explícito hace comprobables replay, duplicados,
mensajes fuera de orden y compatibilidad.

**Alternativas consideradas**: HTTP local, WebSocket, confianza por IP/nombre y descubrimiento como
autenticación. Añaden capas o no aportan identidad suficiente.

## R06 — Motor de benchmark

**Decisión**: `BenchmarkEngine` es un puerto estrecho y NTTTCP 5.40 x64 su única implementación.
Solo `engine/ntttcp` verifica hash, construye argumentos, posee procesos y analiza XML. G1/V-01 a
V-03 se ejecutan antes de congelar puertos, tasa UDP o campos del parser.

**Motivo**: aísla entrada no confiable y permite probar control/diagnóstico con un fake sin
fingir que el fake valida NTTTCP real.

**Alternativas consideradas**: exponer argumentos libres, repartir parsing entre módulos o crear
plugins/múltiples motores. Violan seguridad o preparan extensiones no solicitadas.

## R07 — Concurrencia y async

**Decisión**: Tokio para red, timeouts y coordinación; tareas bloqueantes/FFI en workers limitados;
ningún lock se mantiene durante `.await`. La cancelación es idempotente y tiene prioridad sobre
muestras/logging. NTTTCP vive en un Job Object con cleanup de procesos propios.

**Motivo**: el sistema combina control asíncrono con procesos, SQLite y APIs Windows que pueden
bloquear. Separar estas clases evita bloquear el runtime y hace explícito su lifecycle.

**Alternativas consideradas**: un thread por operación sin coordinador, acceso SQLite dentro del
runtime async o matar procesos por nombre. Dificultan límites y pueden afectar procesos ajenos.

## R08 — Persistencia

**Decisión**: rusqlite con SQLite bundled, WAL, claves foráneas, transacciones y un escritor en
worker dedicado. Ajustes en JSON versionado con escritura atómica; identidad secreta protegida
con DPAPI. Sin ORM ni pool inicial.

**Motivo**: las sesiones tienen relaciones, filtros y migraciones; SQLite cubre atomicidad y
consulta local. Un propietario único reduce carreras. Pool y ORM no resuelven una carga demostrada.

**Alternativas consideradas**: localStorage/IndexedDB, JSON para todo, base externa y ORM.

## R09 — Resultado e interpretación

**Decisión**: `SessionResult` es un snapshot versionado, calculado por reglas puras Rust y guardado
sin reinterpretación histórica. El receptor aporta throughput oficial; ausencias permanecen
ausentes; hechos, observaciones, posibles causas y acciones son categorías separadas. G3/G4 y
V-05 a V-07 deben cerrar tamaños, reconciliación y fronteras antes de congelar schemaVersion 1.

**Motivo**: mantiene auditabilidad y evita que una actualización cambie el significado de una
sesión pasada o convierta datos ausentes en éxito.

**Alternativas consideradas**: recalcular al leer, guardar solo métricas finales o transportar texto
localizado. Pierden evidencia, reproducibilidad o independencia del idioma.

## R10 — Frontend moderno

**Decisión**: Svelte 5 en runes mode (`$state`, `$derived`, `$props`), callbacks y snippets;
TypeScript estricto; Tailwind CSS 4 mediante `@tailwindcss/vite` y `@import "tailwindcss"`.
Design sigue siendo la fuente de tokens y componentes; Tailwind no crea una paleta paralela.

**Motivo**: evita APIs legacy frecuentes en código generado por LLM. La documentación oficial
de Tailwind 4 recomienda el plugin Vite y reemplaza directivas/configuración v3. El proyecto no
necesita SvelteKit, SSR ni rutas de servidor.

**Alternativas consideradas**: Svelte legacy (`export let`, `$:`, dispatcher/slots), Tailwind v3,
PostCSS adicional, SvelteKit o una librería de componentes general. No aportan una responsabilidad
nueva y pueden contradecir el sistema visual existente.

**Referencias primarias**:

- https://svelte.dev/docs/svelte/$state
- https://svelte.dev/docs/svelte/$derived
- https://svelte.dev/docs/svelte/$props
- https://tailwindcss.com/docs/upgrade-guide

## R11 — Testing

**Decisión**: pirámide por riesgo: unitarios puros, componentes/DOM, integración con recursos
reales aislados, UI en navegador y pruebas Windows/laboratorio. Tauri mock no se denomina E2E de
producto. Se hará un piloto de WebDriver Tauri en Windows; Playwright conserva UI web con bridge
explícito si el piloto no cubre el riesgo.

**Motivo**: Tauri documenta que su mock runtime no ejecuta bibliotecas nativas de WebView y que
WebDriver puede operar en Windows. UAC, firewall, WebView2, instalador y dos equipos requieren
evidencia nativa.

**Alternativas consideradas**: todo E2E, todo mock, sleeps y snapshots masivos. Son costosos,
frágiles o no prueban el comportamiento prometido.

**Referencias primarias**:

- https://v2.tauri.app/develop/tests/
- https://v2.tauri.app/develop/tests/webdriver/

## R12 — Cobertura Q2

**Decisión**: medir baseline completa desde L00 y dejar el gate numérico sin ratificar hasta que
el propietario responda Q2. La propuesta documentada es 80 % general por lenguaje/métrica y 90 %
de líneas en módulos críticos Rust; no se configura como aprobada por omisión.

**Motivo**: el porcentaje es una decisión de gobernanza, no un resultado de investigación. La
calidad de aserciones, aceptación, seguridad y laboratorio sigue siendo obligatoria con cualquier
umbral.

**Alternativas consideradas**: adoptar automáticamente la propuesta, inventar un umbral inferior o
omitir medición. Todas convierten una decisión abierta en política tácita.

## R13 — Plataforma y distribución

**Decisión**: el plan usa Windows 10 22H2 y Windows 11 x64, instalador NSIS por máquina y WebView2
offline, conforme a `spec.md`. G5 debe compilar, instalar y arrancar el conjunto real antes de
afirmar compatibilidad.

**Motivo**: es la elección expresa del propietario para esta feature. WebView2 Evergreen no se
declara fijado; cada evidencia registra su versión y el instalador offline exacto.

**Alternativas consideradas**: Windows 10 1809+, solo Windows 11, bootstrapper online, portable y
runtime WebView embebido.

## R14 — Actualizaciones: conflicto resuelto el 2026-09-21

**Resolución**: `spec.md` FR-042 se alineó con la Constitución: `latest.json` estable cuyo artefacto apunta a la URL inmutable de la etiqueta `vX.Y.Z`. El texto siguiente se conserva como registro del conflicto original.

**Decisión**: diseñar H3 contra un único manifiesto estable de última versión porque así quedó
resuelta la spec, pero bloquear implementación/publicación hasta alinear la Constitución, cuya
propuesta exige artefactos/URLs por etiqueta y reserva `latest` para estables.

**Motivo**: no hay una interpretación compatible que preserve simultáneamente ambos contratos.
El conflicto necesita una decisión normativa explícita, no un adaptador que oculte la diferencia.

**Alternativas consideradas**: implementar ambas rutas, decidir en código o ignorar una fuente. Las
tres duplican comportamiento o rompen la jerarquía documental.

## R15 — Accesibilidad por hito: conflicto resuelto el 2026-09-21

**Resolución**: la Constitución 0.5.0 (VI) fija accesibilidad básica desde H1 y revisión completa en H2, coincidiendo con FR-042. El texto siguiente se conserva como registro del conflicto original.

**Decisión**: mantener el alcance de la spec —accesibilidad completa en H2— y no declarar H1
cerrado contra la propuesta constitucional de accesibilidad desde H1 hasta que se alineen ambas.
Los controles H1 se diseñan desde el principio con semántica, teclado, foco y tokens para evitar
una reconstrucción, pero eso no reetiqueta el hito aprobado.

**Motivo**: reduce deuda técnica sin convertir una propuesta pendiente en alcance ratificado.

**Alternativas consideradas**: ignorar accesibilidad hasta H2 o adelantarla silenciosamente a H1.

## R16 — Decisiones pospuestas

**Decisión**: posponer codegen de contratos, segundo motor, múltiples crates, bus genérico,
cache, servicio Windows, nube, otras plataformas, mutation gate global y pool SQLite.

**Motivo**: ninguna tiene necesidad demostrada. Se reconsideran solo ante requisitos aprobados,
consumidores independientes, consultas medidas o fallos repetidos que el diseño actual no resuelva.

**Alternativas consideradas**: preparar extensibilidad hipotética. Aumentaría superficie, builds,
ownership y formas de resolver el mismo problema.
