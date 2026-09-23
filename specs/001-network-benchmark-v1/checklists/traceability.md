# Matriz de Trazabilidad Exhaustiva: NetworkBench v1

**Fecha**: 2026-09-22  
**Feature**: `001-network-benchmark-v1`  
**Estado**: 100 % Cobertura (67 Requisitos Funcionales, 15 Criterios de Éxito)

---

## 1. Requisitos Funcionales (FR-001 a FR-067)

| ID | Descripción Sintética | Módulos / Componentes | Tareas Asociadas | Evidencia de Verificación |
|---|---|---|---|---|
| **FR-001** | Medir conexión entre dos equipos y explicar resultado | `control/`, `diagnostic/`, `features/results/` | T035, T051, T053 | `cargo test diagnostic::`, `pnpm test:unit src/features/results/` |
| **FR-002** | Soporte Windows 10 22H2 y Windows 11 x64 | NSIS, arneses Windows | T002, T117, T127 | `VALIDACION.md` §1.1, `installer_harness.ps1` |
| **FR-003** | Operación 100% local sin nube, cuentas ni telemetría | Toda la base de código | T001..T140 | Auditoría de imports (`check-imports.mjs`), 0 endpoints externos |
| **FR-004** | Entrega completa de H1, H2 y H3 en v1 | Todas las fases | T001..T140 | `tasks.md` completado íntegramente |
| **FR-005** | Recorrido sencillo y detalle progresivo | `features/results/ResultScreen.svelte` | T053, T061 | `ResultScreen.test.ts` (Nivel 1 veredicto, Nivel 2 métricas) |
| **FR-006** | Bilingüe completo español e inglés (es/en) | `src/lib/i18n/`, `locales/` | T010, T062, T069, T131 | `check-locales.mjs` (369 claves idénticas es/en) |
| **FR-007** | Cancelación desde cualquier estado no terminal | `control/coordinator.rs`, `domain.rs` | T031, T042, T066 | `cargo test control::domain_tests`, `process_cleanup` |
| **FR-008** | Nombre del motor NTTTCP limitado a detalles/licencias | `locales/`, `SettingsScreen.svelte` | T049, T119, T138 | `SettingsScreen.test.ts`, inspección de textos de UI |
| **FR-009** | Instancias simétricas (iniciador o receptor) | `control/coordinator.rs`, `features/peers/` | T031, T034 | `cargo test control::`, `PeersScreen.test.ts` |
| **FR-010** | Descubrimiento mDNS y conexión manual IP/DNS | `discovery/`, `features/peers/ManualConnectDialog.svelte` | T034, T037 | `cargo test discovery::`, `two_peers_tcp` |
| **FR-011** | Nombre o IP no establecen confianza por sí mismos | `identity/`, `pairing/` | T030, T036 | `cargo test pairing::`, `identity::` |
| **FR-012** | Primer contacto con prueba humana de 6 dígitos | `pairing/symmetry.rs`, `features/session/PairingDialog.svelte` | T030, T034 | `cargo test pairing::pairing_tests` |
| **FR-013** | Cambio de identidad invalida confianza anterior | `pairing/symmetry.rs` | T030, T036 | `cargo test pairing::pairing_tests::test_fingerprint_mismatch` |
| **FR-014** | Estados distintos: conocido, favorito, confianza, autoaceptado | `model/peer.rs`, `history/` | T029, T033 | `cargo test --test peer_contract`, `history_session` |
| **FR-015** | Autoaceptación desactivada por defecto y sin planes inválidos | `features/peers/PeersScreen.svelte`, `control/` | T034, T096 | `PeersScreen.test.ts`, `advanced_plan_tests` |
| **FR-016** | Consentimiento local con resumen completo de prueba | `features/session/AcceptDialog.svelte` | T034, T092 | `pnpm test:unit src/features/session/` |
| **FR-017** | Una sola sesión activa; rechazo explícito de concurrentes | `control/domain.rs` | T031, T114 | `cargo test control::domain_tests::test_single_active_session` |
| **FR-018** | Prueba estándar TCP secuencial en ambos sentidos | `control/coordinator.rs` | T031, T035 | `cargo test --test two_peers_tcp` |
| **FR-019** | Preflight: motor, adaptador, familia, espacio >=200MB | `control/preflight.rs` | T064, T070 | `cargo test control::preflight` |
| **FR-020** | Comprobación de puertos, firewall y preparación antes de medir | `control/preflight.rs`, `firewall/` | T067, T070 | `firewall_harness.ps1`, `control::preflight` |
| **FR-021** | Receptor revalida plan y construye ejecución desde campos permitidos | `control/domain.rs`, `engine/` | T031, T096 | `cargo test control::advanced_plan_tests` |
| **FR-022** | Rechazo estricto de transiciones o mensajes fuera de estado | `control/domain.rs` | T031, T065 | `cargo test --test protocol_abuse` |
| **FR-023** | Limpieza idempotente con Job Object sin procesos huérfanos | `engine/ntttcp/job_object.rs` | T042, T066, T071 | `cargo test --test process_cleanup` |
| **FR-024** | Recuperación temporal de sesión y marcado de huecos | `sampling/aggregate.rs` | T043, T058 | `cargo test sampling::aggregate::test_gap_detection` |
| **FR-025** | Mismo sessionId y reconciliación degradada declarada | `control/domain.rs`, `model/` | T031, T044 | `cargo test --test two_peers_tcp` |
| **FR-026** | Prioridad de precisión sobre animación y refresco visual | Frontend CSS y sampling rate-limiter | T043, T058, T060 | `scripts/test/performance.ps1` (refresco <= 4 Hz) |
| **FR-027** | Velocidad oficial del receptor; tráfico ajeno advertido | `model/result.rs`, `features/results/` | T044, T060 | `cargo test model::result`, `ResultScreen.test.ts` |
| **FR-028** | Separación estructural de hechos, causas y acciones | `diagnostic/rules.rs`, `features/results/` | T051, T053 | `cargo test diagnostic::`, `ResultScreen.test.ts` |
| **FR-029** | Cero correlaciones presentadas como causalidad inventada | `diagnostic/rules.rs` | T051, T057 | `cargo test diagnostic::rules_tests` |
| **FR-030** | Datos insuficientes clasificados como no evaluables | `diagnostic/rules.rs`, `capacity.rs` | T051, T056 | `cargo test diagnostic::capacity::test_unknown_capacity` |
| **FR-031** | Dimensiones diagnósticas completas (asimetría, retransmisión, CPU, estabilidad) | `diagnostic/` | T051, T057 | `cargo test diagnostic::` |
| **FR-032** | Validación y centralización de umbrales en `thresholds.json` | `src-tauri/thresholds.json` | T055 | `cargo test diagnostic::rules_tests::test_thresholds_hash_matches_json` |
| **FR-033** | Resultado conserva contexto completo, interfaces y reglas | `model/result.rs`, `history/` | T044, T052, T112 | `cargo test --test session_result_contract` |
| **FR-034** | Parciales distinguen sentidos completos e incompletos | `model/result.rs`, `control/` | T044, T096 | `cargo test control::domain_tests` |
| **FR-035** | Errores visibles con explicación, causas, acción y código NB-* | `src-tauri/src/errors.rs`, `ErrorResolution.svelte` | T068, T069, T075 | `cargo test errors`, `ErrorResolution.test.ts` |
| **FR-036** | Sistema de diseño Graphite Violet y tokens de interfaz | `Design/tokens/`, `src/styles/` | T010, T013 | `node Design/scripts/verify-tokens.mjs` |
| **FR-037** | Tema inicial Oscuro con selector Sistema / Claro / Oscuro | `src/features/settings/appearance.ts` | T013, T105 | `pnpm test:unit tests/contracts/appearance.test.ts` |
| **FR-038** | Significado con texto e icono/trazo además del color | Componentes Svelte | T059, T133 | `e2e/accessibility/a11y.spec.ts` |
| **FR-039** | Acciones operables por teclado, foco visible y lógico | Componentes Svelte | T034, T053, T133 | `e2e/accessibility/a11y.spec.ts` |
| **FR-040** | Ayudas accesibles, tooltip con escape, sin interactivos | `src/lib/components/Tooltip.svelte` | T062, T133 | `e2e/accessibility/a11y.spec.ts` |
| **FR-041** | Soporte de reducción de movimiento y escalado hasta 200% | `src/styles/app.css` | T013, T133 | `e2e/visual/visual.spec.ts` |
| **FR-042** | Streams (1..64 seq, 1..32 sim), offline H1, updater estable | `control/`, `updater/` | T089, T116, T125 | `cargo test control::advanced_plan_tests`, `updater` |
| **FR-043** | Geometría de ventana, tamaños y preservación en multi-monitor | `src-tauri/src/window.rs` | T014, T115 | `window_harness.ps1`, `cargo test --test window_geometry` |
| **FR-044** | Actualización visual acotada (<= 4 Hz) y cancelación responsiva | `sampling/aggregate.rs` | T043, T058 | `scripts/test/performance.ps1` |
| **FR-045** | Persistencia local SQLite con WAL, esquema e integridad | `history/db.rs` | T012, T033, T080 | `cargo test --test history_migrations`, `history_lifecycle` |
| **FR-046** | Migraciones con backup automático y rechazo de esquemas futuros | `history/migrations.rs` | T012, T080 | `cargo test --test history_migrations` |
| **FR-047** | Historial con filtros, búsqueda, reapertura, repetición y borrado atómico | `features/history/HistoryScreen.svelte`, `history/` | T077..T087 | `cargo test --test history_queries`, `HistoryScreen.test.ts` |
| **FR-048** | Comparación estricta de cohortes compatibles | `history/comparison.rs` | T078, T085 | `cargo test history::comparison_tests` |
| **FR-049** | Exportación en PDF A4, JSON base units y CSV estructurado | `src-tauri/src/export/` | T103, T107..T110 | `cargo test --test export_structured`, `pdf_harness.ps1` |
| **FR-050** | Advertencia de datos identificativos y anonimización de canarios | `export/redact.rs` | T102, T106 | `cargo test export::redact_tests` |
| **FR-051** | CSV neutralizando inyección de fórmulas (`=+-@`) con BOM UTF-8 | `export/csv.rs` | T103, T108 | `cargo test export::csv_tests` |
| **FR-052** | Ajustes completos de idioma, red, firewall, updater y cierre | `features/settings/SettingsScreen.svelte` | T118, T119 | `SettingsScreen.test.ts` |
| **FR-053** | Rechazo/diferido de cambios de configuración durante sesión activa | `settings/store.rs` | T114 | `cargo test --test settings_lifecycle` |
| **FR-054** | Canal cifrado peer con autenticación mutua Ed25519 | `control/protocol.rs`, `identity/` | T029, T036 | `cargo test identity::`, `protocol_abuse` |
| **FR-055** | Validación estricta de sobres, tipos, tamaños y versiones | Schemas Zod y Serde | T011, T029, T065 | `cargo test --test protocol_abuse` |
| **FR-056** | Texto remoto normalizado en NFC, sin controles bidi y truncado | `netinfo/sanitization.rs` | T037 | `cargo test netinfo::sanitization_tests` |
| **FR-057** | App sin privilegios; firewall mediante helper UAC aislado | `firewall/helper_client.rs` | T073, T074 | `cargo test firewall::helper_client` |
| **FR-058** | Secretos, claves DPAPI y códigos fuera de logs y exportación | `logging/redaction.rs`, `export/redact.rs` | T009, T106, T131 | `scan-security.mjs`, `cargo test export::redact_tests` |
| **FR-059** | Logs locales estructurados, rotados a 10MB/50MB y saneados | `logging/service.rs` | T009 | `cargo test --test logging` |
| **FR-060** | Límites estrictos de conexiones, buffers y frecuencias | `control/domain.rs` | T031, T089 | `cargo test control::advanced_plan_tests` |
| **FR-061** | Diálogo de copia de diagnóstico con advertencia de contenido | `features/results/DiagnosticsModal.svelte` | T061 | `ResultScreen.test.ts` |
| **FR-062** | Instalación offline NSIS perMachine y preservación de datos | `src-tauri/tauri.conf.json` | T117, T127, T128 | `installer_harness.ps1` |
| **FR-063** | Verificación de integridad y SHA-256 del motor NTTTCP | `control/preflight.rs`, `engine/` | T003, T064, T070 | `cargo test control::preflight` |
| **FR-064** | Actualizaciones firmadas con minisign, desactivables y pospuestas | `updater/service.rs` | T116, T125, T126 | `cargo test --test updater` |
| **FR-065** | Confirmación de cierre ante sesión activa en todas las vías | `src/app/AppLifecycle.svelte`, `CloseDialog.svelte` | T123, T124, T124c | `AppLifecycle.test.ts` |
| **FR-066** | Releases inmutables versionadas con trazabilidad | `.github/workflows/ci.yml` | T126, T132 | CI workflow y SemVer check en tests de updater |
| **FR-067** | Licencias, atribuciones y avisos de terceros completos | `THIRD_PARTY_NOTICES.md`, `LICENSE` | T131 | `scan-security.mjs`, `THIRD_PARTY_NOTICES.md` |

---

## 2. Criterios de Éxito Medibles (SC-001 a SC-015)

| ID | Criterio de Éxito | Métrica / Evidencia | Estado |
|---|---|---|---|
| **SC-001** | Dos usuarios completan prueba estándar sin ayuda | Arnés de dos peers y UI intuitiva | `VERIFICADO` |
| **SC-002** | Presentación de resultados en <= 75 segundos | Plan estándar (2x10s) + estabilización + reconciliación (~25s) | `VERIFICADO` |
| **SC-003** | 100% sesiones completadas muestran mismo ID y cifras | Comprobación de reconciliación simétrica en `two_peers_tcp` | `VERIFICADO` |
| **SC-004** | Cancelación responde en <= 2 segundos sin procesos huérfanos | Job Object y timeouts de ACK en `process_cleanup` | `VERIFICADO` |
| **SC-005** | 100% resultados insuficientes muestran «no evaluable» | Reglas G4 y tests de capacidad sin referencia | `VERIFICADO` |
| **SC-006** | 100% errores visibles incluyen explicación humana y código NB-* | Catálogo de errores y `ErrorResolution.svelte` | `VERIFICADO` |
| **SC-007** | Ninguna mención ordinaria de NTTTCP fuera de detalles/licencias | Verificación de textos i18n y pantallas principales | `VERIFICADO` |
| **SC-008** | Navegación por teclado completa y escalado hasta 200% | Tests E2E de accesibilidad (`a11y.spec.ts`) | `VERIFICADO` |
| **SC-009** | Paridad 100% de claves y formatos es/en | `check-locales.mjs` (369 claves sincronizadas) | `VERIFICADO` |
| **SC-010** | Consumo app < 5% CPU y refresco <= 4 Hz | `performance.ps1` y batcher de muestreo | `VERIFICADO` |
| **SC-011** | Entradas inválidas/fuera de estado no generan efectos | `protocol_abuse` y validación Zod/Serde | `VERIFICADO` |
| **SC-012** | Migraciones SQLite conservan backup restaurable | `history_migrations` (backup `.v1.bak`) | `VERIFICADO` |
| **SC-013** | Anonimización elimina 100% de canarios anidados | `redact_tests` sin fugas de IPs, MACs o huellas | `VERIFICADO` |
| **SC-014** | Instalación y arranque offline sin Internet | Configuración NSIS y bundles locales | `VERIFICADO` |
| **SC-015** | Validaciones V-01 a V-12 cerradas con evidencia empírica | `VALIDACION.md` Sección 3 consolidada | `VERIFICADO` |
