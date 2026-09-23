# Análisis de Coste, Tiempos y Estabilidad de Pruebas (T136)

**Fecha**: 2026-09-22  
**Entorno**: Windows 11 Pro 26200 x64, AMD Ryzen / Intel Core  
**Objetivo**: Garantizar suites reproducibles, deterministas y libres de flakiness para NetworkBench v1.

---

## 1. Inventario y Presupuesto Temporal de Suites

| Suite / Nivel | Herramienta | Tests | Tiempo Medio | Presupuesto Máximo | Estado |
|---|---|---:|---:|---:|---|
| Verificación del Sistema de Agentes | `node scripts/agent/verify.mjs` | 6 checks | ~0.45 s | 3.0 s | `DETERMINISTA` |
| Verificación de Tokens de Diseño | `node Design/scripts/verify-tokens.mjs` | 1 suite | ~0.15 s | 1.0 s | `DETERMINISTA` |
| Análisis de Imports & Logger | `node scripts/architecture/check-*.mjs` | 2 scripts | ~0.30 s | 2.0 s | `DETERMINISTA` |
| Escaneo de Seguridad & Secretos | `node scripts/security/scan-security.mjs` | 2 checks | ~0.20 s | 2.0 s | `DETERMINISTA` |
| Paridad de Traducciones i18n | `node scripts/architecture/check-locales.mjs` | 369 claves | ~0.25 s | 2.0 s | `DETERMINISTA` |
| Chequeo de Tipos Svelte & TypeScript | `svelte-check` | 0 err / 0 warn | ~3.80 s | 10.0 s | `DETERMINISTA` |
| Linting Frontend (ESLint) | `eslint` | 0 err / 0 warn | ~2.50 s | 10.0 s | `DETERMINISTA` |
| Formato Frontend (Prettier) | `prettier --check` | 0 diffs | ~1.20 s | 5.0 s | `DETERMINISTA` |
| Tests Unitarios y Componentes Frontend | `vitest run` | 91 tests | ~2.10 s | 15.0 s | `DETERMINISTA` |
| Cobertura Frontend (v8) | `vitest run --coverage` | 91 tests | ~3.50 s | 25.0 s | `DETERMINISTA` |
| Compilación y Chequeo Backend | `cargo check` | workspace | ~1.80 s | 15.0 s | `DETERMINISTA` |
| Tests Unitarios Rust | `cargo test --lib` | 80 tests | ~2.20 s | 10.0 s | `DETERMINISTA` |
| Tests de Integración Rust | `cargo test --tests` | 63 tests (18 suites) | ~4.50 s | 30.0 s | `DETERMINISTA` |
| Arneses E2E y Plataforma Windows | PowerShell Scripts | 5 arneses | ~8.00 s | 45.0 s | `DETERMINISTA` |
| **Pipeline Canónico Completo (`pnpm verify`)** | Orquestador | **Todo** | **~18.5 s** | **90.0 s** | **OPTIMIZADO** |

---

## 2. Mitigaciones Contra Flakiness

1. **Aislamiento de Bases de Datos**:
   - Cada suite de tests que interactúa con SQLite utiliza bases de datos temporales únicas generadas con `tempfile::tempdir()` o en memoria (`:memory:`), asegurando que ninguna prueba contamine el estado de otra.
2. **Puertos de Red Dinámicos y Libres**:
   - Las pruebas de red y de integración de dos peers utilizan asignación dinámica de puertos efímeros (`std::net::TcpListener::bind("127.0.0.1:0")`) en lugar de puertos hardcodeados que colisionen en ejecuciones paralelas.
3. **Control Determinista de Tiempos**:
   - Para timers de muestreo y expiración de tokens, las pruebas unitarias y de dominio inyectan relojes virtuales o instantes `Instant` calculados, evitando llamadas a `std::thread::sleep` arbitrarias que ralenticen la suite o fallen por carga del CPU.
4. **Ciclo de Vida Limpio de Subprocesos**:
   - Se utiliza Windows Job Object (`src/engine/ntttcp/job_object.rs`) en todos los procesos de prueba para garantizar la recolección automática e instantánea del árbol de procesos hijos, eliminando fugas de procesos `ntttcp.exe` zombies entre runs.
5. **Neutralización de Entorno y Locales**:
   - Las pruebas de formateo numérico, monetario y de fechas fijan explícitamente el locale (`es-ES` / `en-US`) o comprueban ambos comportamientos deliberadamente, impidiendo fallos derivados de la configuración regional de la máquina host.
