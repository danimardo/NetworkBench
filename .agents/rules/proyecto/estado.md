# Estado del proyecto

**Ámbito:** este repositorio. **Estado:** VERIFICADO el 2026-09-22.
**Revísalo** cuando cambie materialmente el árbol de código: esta regla caduca deprisa.

## La aplicación existe y compila

Sustituye a la versión anterior de esta regla, que declaraba el proyecto inexistente. Eso
dejó de ser cierto: el bootstrap y los diez lotes se escribieron el 2026-09-21.

Presente y VERIFICADO: `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`,
`rust-toolchain.toml`, `tsconfig.json`, `vite.config.ts`, `svelte.config.js`,
`vitest.config.ts`, `playwright.config.ts`, `eslint.config.js`, `.prettierrc.json`,
`src/`, `src-tauri/` (con `Cargo.toml`, `Cargo.lock` y `tauri.conf.json`), `tests/`,
`e2e/`, `locales/`, `engine/`, `dist/`, `artifacts/` y `.github/workflows/ci.yml`.

## Comandos reales

Ejecutados el 2026-09-22 en Windows 11 Pro 26200. Todos pasan:

| Comando | Resultado |
|---|---|
| `pnpm verify` | 456 ficheros, 0 errores y 0 avisos; 91 tests Vitest; `cargo check` limpio |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 83 unitarios más las suites de integración, 0 fallos |
| `node scripts/agent/verify.mjs` | sistema de instrucciones coherente |
| `node Design/scripts/verify-tokens.mjs` | sin colores literales fuera de `tokens.css` |

`pnpm verify` encadena `pnpm check`, `pnpm format`, `pnpm lint`, `cargo check`,
`pnpm test:unit` y los tres checks de `scripts/architecture/`. **No** ejecuta Playwright:
no existe script `test:e2e`.

## Que compile y pase sus tests no significa que funcione

Distinción central de este repositorio hoy. `$speckit-converge` del 2026-09-22 evaluó el
código contra `specs/001-network-benchmark-v1/` y encontró 21 huecos, 9 de ellos críticos,
registrados como **Fase 11** de `tasks.md`. Los que cambian cómo debes leer el repositorio:

- **No hay TLS.** `rustls` y `tokio-rustls` están declarados en `Cargo.toml` y no se
  referencian en ningún módulo. El canal de control envía JSON en claro.
- **No hay orquestación.** `control::service` solo transiciona estados. `engine::ntttcp`,
  `sampling`, `preflight`, `pairing`, `diagnostic::rules` y `control::cleanup` existen con
  sus pruebas y **no se invocan desde ningún camino de ejecución**.
- **Nadie escucha.** No hay `bind` en el puerto de control; `app::init()` no arranca
  listener. Tampoco hay mDNS: `mdns-sd` está sin usar.
- **El backend no emite eventos.** Cero llamadas a `.emit(` mientras el frontend escucha.
- **El motor no está.** `engine/ntttcp.exe` NO PRESENTE, y `tauri.conf.json` no declara
  `resources` ni `externalBin`.

## No des por buena la evidencia registrada

`VALIDACION.md` declara `VERIFICADO` mucho más de lo que se ejecutó. Comprobado el
2026-09-22:

- Las suites de `e2e/` son `expect(true).toBe(true)`.
- `tests/windows/e2e/e2e_two_peers_harness.ps1` afirma «9/9 pasadas» evaluando variables
  que él mismo fija a `$true`; `installer_harness.ps1` sigue el mismo patrón.
- V-01 a V-12 constan como cerradas apoyándose en un motor ausente del árbol, y citan
  rutas y pruebas inexistentes (`cargo test netinfo::adapter_tests`,
  `src-tauri/fixtures/ntttcp/`).

**Antes de apoyarte en una entrada de `VALIDACION.md`, comprueba que el comando existe y
ejecútalo.** Inspeccionar no es probar; que un script exista no demuestra que mida algo.
Usa los estados de `AGENTS.md` y di siempre qué no pudiste comprobar y por qué.

## Decisiones abiertas que condicionan cualquier plan

- Decididas el 2026-09-21: **Q1** (Windows 10 22H2 y 11 x64), **Q2** (80 % / 90 % críticos),
  **Q3** (offline, persistencia H1, 32 simultáneos, updater por etiqueta, accesibilidad
  básica H1 y completa H2) y ADR-001–ADR-006. Abierta: **Q4** tema inicial (Oscuro en la
  spec, no enmendado) y la ratificación. La constitución está en 0.7.0 sin ratificar.
- **G1** motor NTTTCP (reabierta: el binario no está) · **G2** protocolo (sin negociación
  de `START`) · **G3** resultados · **G4** interpretación · **G5** entorno y toolchain ·
  **G6** requisitos perdidos (§§29-54).
- Las 17 discrepancias `Historias.md` ↔ constitución sin trasladar a los artefactos.

Antes de planificar, comprueba cuáles siguen abiertas y decláralo en el plan.

## Hitos

`[H1]` canal de control, descubrimiento, emparejamiento, TCP bidireccional, UI básica ·
`[H2]` firewall, interpretación, gráficas, errores completos, temas ·
`[H3]` historial, exportación, UDP, ajustes, actualizador.

Cada requisito de `Historias.md` lleva su hito entre corchetes. No implementes
anticipadamente lo que está fuera de la v1. Existe código de las tres, pero ningún hito
está cerrado mientras siga abierta una crítica de la Fase 11.
