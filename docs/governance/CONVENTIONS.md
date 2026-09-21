# Convenciones tecnológicas

**Estado:** propuesta. Las versiones exactas se consultan en la constitución 0.4.0 no
ratificada. Ninguna está resuelta en manifiestos del proyecto ni validada como conjunto.

## Svelte 5

**USE:** runes (`$state`, `$derived`, `$props`), callbacks tipados, snippets, componentes
pequeños y cleanup explícito de suscripciones. Derivar antes que sincronizar con `$effect`.

**DO NOT INTRODUCE:** `export let`, `$:`, `createEventDispatcher`, `<slot>` o componentes
tratados como clases en código nuevo por copiar patrones Svelte 3/4. SvelteKit, SSR,
hidratación, `+page`, `+layout`, `load` y APIs `$env` no pertenecen a esta arquitectura.

Los stores no están prohibidos por ser legacy; se introducen solo si un problema real de
streams externos o suscripciones manuales lo justifica. No se usa un store global como
segunda autoridad de sesión.

## TypeScript y Zod

**USE:** `strict`, `noUncheckedIndexedAccess`, uniones discriminadas, tipos de dominio,
`unknown` en fronteras, narrowing exhaustivo y tipos inferidos de schemas Zod.

**DO NOT INTRODUCE:** `any` evasivo, doble cast, `!`, enum numérico o default silencioso
para convertir datos inválidos en operaciones autorizadas. `as`, `satisfies` y tipos genéricos
no validan datos en runtime.

Un schema vive con su contrato. Formularios, IPC, configuración, archivos e importaciones
parsean entrada y salida. Las coerciones son explícitas: la cadena `false` no se interpreta
como booleano verdadero por existir.

## Vite y configuración

**USE:** ESM, `import.meta.env` únicamente en `lib/config`, allowlist de claves públicas y
validación Zod al arrancar. Los recursos del producto se empaquetan localmente.

**DO NOT INTRODUCE:** `process.env` en la aplicación, polyfill de `process`, exposición del
entorno completo, `envPrefix` amplio o secretos `VITE_*`. Un valor incluido en el bundle es
público. `.env` no persiste preferencias de la aplicación instalada.

## Tailwind CSS 4 y diseño

**USE:** plugin oficial de Vite, integración CSS de Tailwind 4, `@theme`/`@theme inline`
cuando corresponda y tokens de Design como única escala visual. Clases completas y detectables
durante build. CSS propio permanece válido para materiales, animación, impresión y controles.

**DO NOT INTRODUCE:** configuración v3 copiada, paleta paralela, CDN, Sass/Less, clases
construidas dinámicamente o valores arbitrarios no vinculados a tokens/excepciones documentadas.

Revisar Preflight, cascada y soporte del WebView2 mínimo. `Design/maqueta-navegable.html`
orienta; no es contrato ni código de producción.

## Tauri 2 e IPC

**USE:** `@tauri-apps/api` actual, comandos estrechos organizados por área, clientes IPC
tipados, errores serializables, permisos/capabilities mínimos y CSP local restrictiva.
Comandos para acciones; eventos/snapshots para cambios asíncronos.

**DO NOT INTRODUCE:** allowlists o imports de Tauri 1, `window.__TAURI__` global, shell/SQL/fs
genérico desde UI, payloads sin límite o nombres de comando dispersos por componentes.

Registrar un comando propio puede dejarlo accesible a las webviews registradas si el manifiesto
no se restringe expresamente. Cada permiso se verifica con un caso permitido y otro denegado.

## Rust 2024

**USE:** `Result`, enums y newtypes para estados/identificadores/unidades; match exhaustivo;
tipos Serde más validadores; ownership explícito y APIs públicas mínimas. Clippy con warnings
como errores según la configuración finalmente adoptada.

**DO NOT INTRODUCE:** `unwrap`/`expect` sobre entrada o I/O recuperable, `panic!` como flujo,
strings como máquina de estados, FFI en dominio o `unsafe` sin invariante documentada y prueba
del wrapper seguro. `pub(crate)` no sustituye el control del grafo de dependencias.

## Async y concurrencia

**USE:** Tokio para I/O, timeouts explícitos, cancelación idempotente, canales acotados y
trabajo bloqueante aislado. Los procesos tienen propietario, plazo y cleanup.

**DO NOT INTRODUCE:** I/O bloqueante en workers async, mutex durante `.await`, tareas detached
sin responsable, colas infinitas, sleeps para coordinar o supuesto de que abortar un
`spawn_blocking` detendrá trabajo ya iniciado.

Duraciones usan reloj monotónico; UTC para persistencia. El orden causal entre equipos no se
deduce de timestamps de pared.

## SQLite y archivos

**USE:** rusqlite, SQL parametrizado, WAL, claves foráneas comprobadas por conexión,
transacciones, consultas específicas y migrations numeradas. Backup compatible con WAL y
restauración ensayada. Archivos JSON versionados con escritura temporal + reemplazo atómico.

**DO NOT INTRODUCE:** ORM sin problema demostrado, SQL fuera de `history`, una conexión SQLite
bloqueando Tokio, copia simple del `.db` abierto como backup o reinicialización silenciosa.

## Logging y errores

**USE:** fachadas propias sobre `tracing` y `loglevel`, contexto tipado, allowlist de campos,
redacción antes de cada sink, buffers/rotación acotados y errores NB estructurados.

**DO NOT INTRODUCE:** `console.*`, `println!`, `eprintln!`, `dbg!` o macros `tracing` directas
fuera de los adaptadores; serialización completa de objetos/error/payload; secretos, huellas,
IPs, rutas personales o `sessionId` en logs.

Un fallo se registra en la frontera responsable; propagarlo no justifica duplicarlo.

## UI y accesibilidad

**USE:** elementos HTML nativos, nombres accesibles, `:focus-visible`, teclado, retorno de
foco, estados con texto/icono además de color, i18n es/en y reducción de movimiento.

**DO NOT INTRODUCE:** `<select>` nativo si contradice el sistema aprobado, diálogos del
navegador, `title` como ayuda, emojis como iconos, `{@html}` para datos, fuentes remotas o
interacción solo con ratón.

La precisión del benchmark tiene prioridad sobre animaciones. Durante medición se usan
`transform`/`opacity`, buffers limitados y refresco de datos máximo acordado.

## Tests

**USE:** tablas de casos, fixtures auténticos saneados, builders con defaults válidos, reloj/IDs
inyectados, SQLite/TLS/procesos reales donde sean la garantía y localizadores accesibles.

**DO NOT INTRODUCE:** tests por archivo, snapshots de objetos enormes, implementación duplicada
en el expected, `waitForTimeout`, dependencia del orden, acceso a Internet en suites rápidas o
llamar E2E del producto a Playwright con IPC simulado.

