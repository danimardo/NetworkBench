# Matriz de trazabilidad: NetworkBench v1

**Fecha**: 2026-09-24. **Reescrita íntegramente (T163)**: la versión del 2026-09-22
declaraba «100 % Cobertura» citando como evidencia 14 de 54 rutas inexistentes
(`control/coordinator.rs`, `pairing/symmetry.rs`, `features/session/AcceptDialog.svelte`,
`features/session/PairingDialog.svelte`, `features/peers/ManualConnectDialog.svelte`,
`history/db.rs`, `history/migrations.rs`, `settings/store.rs`, `logging/redaction.rs`,
`netinfo/sanitization.rs`, `src-tauri/src/window.rs`, `src-tauri/src/errors.rs`,
`export/csv_tests`, `features/results/DiagnosticsModal.svelte`,
`src/app/AppLifecycle.svelte`) y mapeaba FR-003/FR-004 a «T001..T140», que no es una
trazabilidad. Este documento cita solo rutas comprobadas contra el árbol el 2026-09-24.

**Estados**: `VERIFICADO` (ejecutado en esta sesión o en una entrada de `VALIDACION.md`
con comando y resultado) · `PARCIAL` (implementado, pero incompleto o sin ejecutar el
camino completo) · `NO PRESENTE` (no existe código) · `NO VERIFICABLE` (exige dos
equipos, elevación o una máquina que este entorno no tiene).

**Recuento honesto** (actualizado el 2026-09-24 tras cerrar T169): de 67 FR, 47
`VERIFICADO`, 15 `PARCIAL`, 5 `NO PRESENTE`. De 15 SC, 7 `VERIFICADO`, 3 `PARCIAL`, 4
`NO PRESENTE`, 1 `NO VERIFICABLE`. Nada de esto es
«100 % cobertura».

**Este recuento está desactualizado desde el 2026-09-25**: T144, T147, T150, T151, T154,
T159, T160, T168, T170, T171, T177 y T181 cambiaron el estado de varias filas ese mismo
día (FR-042 pasó de una fila compuesta a FR-042a–e, por ejemplo). Recontar las 67+4 FR y
15 SC fila por fila es su propia tarea, no una comprobación que quepa de paso al cerrar
T170; no se ha hecho aquí.

---

## 1. Requisitos funcionales (FR-001 a FR-067)

### Alcance y experiencia general

| ID | Módulos reales | Tareas | Evidencia | Estado |
|---|---|---|---|---|
| **FR-001** | `diagnostic/rules.rs`, `features/results/ResultScreen.svelte` | T051, T053 | `cargo test diagnostic::`, `pnpm test:unit src/features/results/` | `VERIFICADO` |
| **FR-002** | `src-tauri/tauri.conf.json` (NSIS), `tests/windows/installer/installer_harness.ps1` | T002, T117, T127, T145–T157 | `VALIDACION.md` §1.7ter (13/13 comprobaciones sin elevar) | `PARCIAL` — no instalado en Windows 10 22H2 real |
| **FR-003** | Toda la base; `scripts/architecture/check-imports.mjs` | T011, T130 | 0 endpoints externos detectados por el checker | `VERIFICADO` |
| **FR-004** | `tasks.md` (fases 3–9, US1–US7) | T029–T129 | Todas las historias de usuario tienen implementación con pruebas propias | `VERIFICADO` (con las lagunas declaradas en este documento) |
| **FR-005** | `features/results/ResultScreen.svelte`, `BasicResult.svelte` | T053, T061 | `ResultScreen.test.ts`, `BasicResult.test.ts` | `VERIFICADO` |
| **FR-006** | `src/lib/i18n/`, `locales/es.json`, `locales/en.json` | T010, T022, T062, T131 | `check-locales.mjs`: 369 claves sincronizadas | `VERIFICADO` — SC-009 exige además formatos/plurales, sin comprobar (T169) |
| **FR-007** | `control/domain.rs`, `control/session_flow.rs` (T175) | T031, T066, T175 | `cargo test control::domain_tests`, `process_cleanup`, `control::session_flow::preflight_en_el_dialogo`; `CANCEL`/`CANCEL_ACK` real solo antes de que arranque el motor | `VERIFICADO` |
| **FR-008** | `locales/*.json`, `features/settings/AboutScreen.svelte` | T049, T119, T124d | Inspección de textos: NTTTCP solo en detalles técnicos y Acerca de | `VERIFICADO` |

### Equipos, identidad y consentimiento

| ID | Módulos reales | Tareas | Evidencia | Estado |
|---|---|---|---|---|
| **FR-009** | `control/server.rs` (escucha), `control/service.rs` | T143 | `VALIDACION.md` §1.8: servidor real, TLS mutuo, HELLO | `VERIFICADO` |
| **FR-010** | `discovery/anuncio.rs` (T151), `discovery/mdns.rs` (conexión manual), `ipc/peers.rs::peers_discovered_list` | T037, T151 | `cargo test discovery::anuncio`; `tests/mdns_real.rs` (`--ignored`, multicast real entre dos instancias de una máquina); `conectar_y_saludar` sobre TLS real. Sin pantalla que muestre los equipos descubiertos; sin prueba entre equipos distintos (V-08) | `PARCIAL` |
| **FR-011** | `control/tls.rs` (huella del certificado, no del `instanceId` declarado) | T141, T142 | `test_un_instance_id_falsificado_no_cambia_la_huella` | `VERIFICADO` |
| **FR-012** | `pairing/mod.rs`, `control/pairing_flow.rs`, `control/pairing_incoming.rs` (T182) | T030, T143, T153, T182 | `test_ambos_extremos_derivan_el_mismo_codigo`; `session_service_real t182` (TLS real): el lado que recibe verifica antes de preguntar, muestra el mismo código y solo con un sí guarda `Trusted` sin autoaceptación. Sin pantalla que lo use (T181) | `PARCIAL` |
| **FR-013** | `control/pairing_flow.rs::atender_emparejamiento` | T030, T143 | `test_una_huella_distinta_invalida_el_emparejamiento` | `VERIFICADO` |
| **FR-014** | `model/peer.rs::TrustState` (Unknown/Known/Trusted + `auto_accept`) | T029, T038 | `cargo test --test peer_contract` | `VERIFICADO` |
| **FR-015** | `ipc/pairing.rs` (`auto_accept` forzado a `false` al aceptar) | T038, T143 | Código explícito en `peers_pairing_confirm`; sin prueba de UI que lo exija activar con aviso | `PARCIAL` |
| **FR-016** | `control/consent.rs`, `ipc/consent.rs` (T177); `features/session/SessionScreen.svelte` (sin conectar, T181) | T034, T048, T177 | `cargo test control::consent`; `session_service_real` (`--ignored`): aceptar de verdad completa la sesión con NTTTCP real. Backend cableado y probado; sin pantalla que lo use todavía (T181) | `PARCIAL` |

### Sesión y medición

| ID | Módulos reales | Tareas | Evidencia | Estado |
|---|---|---|---|---|
| **FR-017** | `control/domain.rs::SessionStateMachine` | T031 | `cargo test control::domain_tests` (sesión única, rechazo explícito) | `VERIFICADO` |
| **FR-018** | `control/orquestador.rs`, `engine/ntttcp/` | T041, T042, T144 | Medición real en bucle local (`engine_real.rs`); `session_service_real.rs` (`--ignored`): sesión TCP en ambos sentidos entre dos servicios completos con NTTTCP real, en bucle local (VALIDACION §1.16). Sin UDP/simultáneo en el diálogo (T178) ni entre dos equipos (T180) | `PARCIAL` |
| **FR-019** | `control/preflight.rs`, `control/session_flow.rs` (T173) | T064, T070, T173 | `cargo test control::preflight control::session_flow::preflight_en_el_dialogo`; solo el NIC del iniciador se invoca desde la sesión real, motor/disco/versión siguen sin invocarse ahí | `VERIFICADO` |
| **FR-020** | `control/preflight.rs`, `control/session_flow.rs` (T173) | T067, T070, T072, T173 | `firewall_harness.ps1`, `cargo test control::preflight control::session_flow::preflight_en_el_dialogo`: puertos de datos del receptor comprobados antes de aceptar, con NTTTCP real de por medio | `VERIFICADO` |
| **FR-021** | `control/plan.rs::BenchmarkPlan::validate` | T039, T093 | `cargo test control::advanced_plan_tests` | `VERIFICADO` |
| **FR-022** | `control/domain.rs::can_transition_to` | T031, T065 | `cargo test --test protocol_abuse` | `VERIFICADO` |
| **FR-023** | `engine/ntttcp/job_object.rs::JobObject` (Drop), `engine/ntttcp/process.rs::NtttcpProcess` (Drop), `control/session_flow.rs` (T175) | T042, T066, T071, T175, T144 | `session_service_real` (`--ignored`) cancela a mitad de medición sin dejar `ntttcp.exe`, y cancela antes de arrancarlo con `CANCEL`/`CANCEL_ACK` real | `VERIFICADO` — **corregido el 2026-09-25**: la limpieza real es RAII (`Drop` de `JobObject`/`NtttcpProcess`), no `control/cleanup.rs::CleanupCoordinator`, que sigue existiendo con su propio test (`process_cleanup.rs`) pero **sin invocarse desde `AppState` ni desde ningún camino de ejecución real** (verificado por grep, cero usos fuera de su propio fichero) — la cita anterior era incorrecta |
| **FR-024** | `control/session_flow.rs::conservar_lo_completado`, `control/service.rs`, `sampling/` | T171 | `session_flow::perdida_de_canal`; `session_service_real t171` (`--ignored`, NTTTCP real): tras perder el canal en la segunda pata se conserva la primera, la cortada no aparece como continua y la sesión se guarda `incomplete`. La reconexión con el mismo `sessionId` («MAY») y el latido de §8.5 **no existen** | `PARCIAL` |
| **FR-025** | `control/protocol.rs::ResultadoSincronizacion`, `control/session_flow.rs` (T176) | T044, T152, T176 | `cargo test control::protocol control::session_flow::reconciliacion_de_resultado`; `session_service_real` (`--ignored`): B adopta el `resultSource: "initiator"` de A tras validar `SESSION_RESULT` real | `VERIFICADO` |
| **FR-026** | `sampling/vivo.rs`, `control/service.rs::lanzar_aplicador` | T043, T058, T174 | `cargo test sampling::vivo`, `session_service_real` (`--ignored`, NTTTCP real): muestreo real por interfaz, salvo en bucle local (VALIDACION §1.17); arnés de `performance.ps1` sigue midiendo procesos existentes (T159) | `PARCIAL` |
| **FR-027** | `control/orquestador.rs::resultado_de_direccion` | T044, T144 | `test_la_velocidad_oficial_es_la_del_receptor` | `VERIFICADO` |
| **FR-028** | `diagnostic/rules.rs::SessionVerdict` (facts/observations/causes/actions) | T051, T057 | `cargo test diagnostic::` | `VERIFICADO` |
| **FR-029** | `diagnostic/rules.rs` | T051, T057 | `cargo test diagnostic::rules_tests` | `VERIFICADO` |
| **FR-030** | `control/orquestador.rs` (ausente ≠ cero), `diagnostic/capacity.rs` | T056, T144 | `test_sin_receptor_no_hay_velocidad_oficial` | `VERIFICADO` |
| **FR-031** | `diagnostic/rules.rs`, `diagnostic/capacity.rs`, `diagnostic/udp.rs` | T051, T057, T097 | `cargo test diagnostic::` | `VERIFICADO` |
| **FR-032** | `diagnostic/thresholds.json`, `diagnostic/mod.rs::THRESHOLDS_HASH` | T055 | `test_thresholds_hash_matches_json` | `VERIFICADO` |
| **FR-033** | `model/result.rs::SessionResult` | T044, T052 | `cargo test --test session_result_contract` | `VERIFICADO` |
| **FR-034** | `control/orquestador.rs::resultado_de_direccion` (`completed`/`incomplete`/`notStarted`) | T044, T144 | `test_direccion_sin_datos_es_no_iniciada` | `VERIFICADO` |
| **FR-035** | `errors/mod.rs`, `features/session/ErrorResolution.svelte` | T068, T069 | `ErrorResolution.test.ts` | `VERIFICADO` |

### Interfaz, diseño y accesibilidad

| ID | Módulos reales | Tareas | Evidencia | Estado |
|---|---|---|---|---|
| **FR-036** | `src/lib/design-system/tokens.css` | T012 | `node Design/scripts/verify-tokens.mjs` | `VERIFICADO` |
| **FR-037** | `src/lib/design-system/theme.svelte.ts` (Oscuro por defecto) | T120 | `tests/contracts/appearance.test.ts` | `VERIFICADO` |
| **FR-038** | Componentes Svelte (`StatusPill.svelte`, `Icon.svelte`) | T034, T059, T147 | Pruebas de componente sobre fixtures; `e2e/accessibility/a11y.spec.ts` con axe-core real sobre Inicio/Historial/Ajustes (cero violaciones) | `VERIFICADO` |
| **FR-039** | Componentes Svelte con manejo de teclado; `SettingsScreen.svelte` (tablist), `AppearanceSettings.svelte` (radiogroup), `Dialog.svelte` (atrapado de foco) | T034, T053, T147 | `e2e/accessibility/a11y.spec.ts`: flechas/Home/End en tablist y radiogroup (huecos APG reales encontrados y corregidos), foco atrapado y devuelto al cerrar un diálogo | `VERIFICADO` |
| **FR-040** | `src/lib/components/Tooltip.svelte`, `HelpTooltip.svelte` | T040, T062 | Igual que FR-038 | `PARCIAL` |
| **FR-041** | `src/lib/design-system/theme.svelte.ts` (reduceMotion) | T120, T147 | `appearance.test.ts` cubre la preferencia; `e2e/visual/visual.spec.ts` comprueba zoom 100/150/200 % sin desbordamiento (aproximación con zoom CSS, no DPI real de Windows) | `PARCIAL` |
| **FR-042a** *(instalación sin red)* | `tauri.conf.json` (NSIS `offlineInstaller`) | T146 | `installer_harness.ps1`: 209,9 MiB, `offlineInstaller` | `VERIFICADO` |
| **FR-042b** *(persistencia desde H1)* | `history/database.rs`, `settings/mod.rs` | T025, T026 | `cargo test --test history_migrations`, `settings_store` | `VERIFICADO` |
| **FR-042c** *(streams 1–64/1–32)* | `control/plan.rs::BenchmarkPlan::validate` | T093 | `cargo test control::advanced_plan_tests` | `VERIFICADO` |
| **FR-042d** *(manifiesto `latest.json` estable)* | `updater/mod.rs` | T125, T126 | `test_updater_rejects_mutable_latest_urls` | `VERIFICADO` |
| **FR-042e** *(accesibilidad completa en H2)* | Fuera de alcance de v1 (H1); `e2e/accessibility/a11y.spec.ts` (T147) cubre la accesibilidad básica de H1, no la revisión completa que exige H2 | T133, T147 | `e2e/accessibility/a11y.spec.ts`: axe-core real sobre Inicio/Historial/Ajustes, cero violaciones — es la comprobación de H1, no la de H2 (Narrador, alto contraste, 200 % integral) | `NO PRESENTE` — H2 no ha empezado |
| **FR-043** | `platform/window.rs` | T121 | `cargo test --test window_geometry` (regla 100×100, fallback) | `VERIFICADO` — multi-monitor real sin probar (`window_harness.ps1` no abre ventana, T159) |
| **FR-044** | `sampling/aggregate.rs` (batching ≤4 Hz), `sampling/vivo.rs` (T174) | T043, T058, T174 | `cargo test sampling::aggregate sampling::vivo` | `VERIFICADO` — medición de cancelación bajo degradación real sin probar |

### Historial, exportación y ajustes

| ID | Módulos reales | Tareas | Evidencia | Estado |
|---|---|---|---|---|
| **FR-045** | `history/database.rs`, `history/migrations/mod.rs` | T025 | `cargo test --test history_migrations` (WAL, FK, backup) | `VERIFICADO` |
| **FR-046** | `history/migrations/mod.rs` | T025, T080 | `cargo test --test history_migrations::test_future_schema_rejected` | `VERIFICADO` |
| **FR-047** | `history/queries.rs`, `history/delete.rs`, `features/history/HistoryScreen.svelte` | T077–T087 | `cargo test --test history_queries`, `HistoryScreen.test.ts` | `VERIFICADO` |
| **FR-048** | `history/comparison.rs` | T078, T085 | `cargo test history::comparison_tests` | `VERIFICADO` |
| **FR-049** | `export/pdf.rs`, `export/json.rs`, `export/csv.rs` | T103, T107–T110 | `cargo test --test export_structured`, `pdf_harness.ps1` | `VERIFICADO` |
| **FR-050** | `export/redact.rs` | T102, T106 | `cargo test export::redact_tests` | `VERIFICADO` |
| **FR-051** | `export/csv.rs` | T103, T108 | `cargo test --test export_structured` (BOM, neutralización `=+-@`) | `VERIFICADO` |
| **FR-052** | `features/settings/*.svelte` (8 pestañas) | T118, T119, T124b–T124d | `SettingsScreen.test.ts` | `VERIFICADO` |
| **FR-053** | `settings/mod.rs` | T114 | `cargo test --test settings_lifecycle` | `VERIFICADO` |

### Seguridad, privacidad y diagnóstico

| ID | Módulos reales | Tareas | Evidencia | Estado |
|---|---|---|---|---|
| **FR-054** | `control/tls.rs`, `control/server.rs` | T141 | `VALIDACION.md` §1.8: TLS mutuo real, `client_auth_mandatory` | `VERIFICADO` |
| **FR-055** | `model/protocol.rs`, `src/lib/contracts/*.ts` | T016, T029, T160 | `cargo test --test protocol_abuse`; `cargo test --test contract_fixtures` + `tests/contracts/rust-fixtures.test.ts` (el JSON que Rust emite, parseado con Zod) | `VERIFICADO` |
| **FR-056** | `netinfo/resolve.rs::sanitize_display_name` | T037 | `test_conexion_manual_toma_la_huella_del_certificado` (nombre saneado) | `VERIFICADO` |
| **FR-057** | `firewall/helper_client.rs`, `firewall/validation.rs`, `helper/main.rs`, `firewall/inspect.rs` | T073, T074, T154 | `cargo test firewall::validation`; `cargo test --test firewall_helper` (binario real, `--validate-only`): lista blanca estricta y única, petición por argumentos sin fichero temporal. **La elevación real con `ShellExecuteExW` no se ha ejecutado** (abre un UAC) | `PARCIAL` |
| **FR-058** | `logging/mod.rs` (redacción), `export/redact.rs` | T020, T106 | `scripts/security/scan-security.mjs` (0 secretos), `redact_tests` | `VERIFICADO` |
| **FR-059** | `logging/mod.rs` | T020, T021b | `cargo test --test logging` (rotación 10/50 MB) | `VERIFICADO` |
| **FR-060** | `control/domain.rs`, `model/protocol.rs::MAX_FRAME_SIZE_BYTES` | T060 | `cargo test --test protocol_abuse` (tramas sobredimensionadas) | `VERIFICADO` |
| **FR-061** | `logging/diagnostics.rs`, `features/results/TechnicalDetails.svelte` | T061 | `ResultScreen.test.ts` (aviso previo a copiar) | `VERIFICADO` |

### Distribución y ciclo de vida

| ID | Módulos reales | Tareas | Evidencia | Estado |
|---|---|---|---|---|
| **FR-062** | `src-tauri/tauri.conf.json`, `src-tauri/nsis/hooks.nsh` | T145, T146, T157 | `VALIDACION.md` §1.7ter: 13/13 comprobaciones sin elevar; instalación/desinstalación reales sin ejecutar | `PARCIAL` |
| **FR-063** | `control/engine_port.rs::MotorNtttcp`, `engine/SHA256` | T042, T145 | `test_un_motor_con_hash_distinto_no_se_ejecuta` (verificado en cada ejecución, no solo al instalar) | `VERIFICADO` |
| **FR-064** | `updater/mod.rs` | T116, T126 | `cargo test --test updater`: SemVer, URL inmutable, sesión activa. **La firma no se verifica criptográficamente**, solo que el campo no esté vacío | `PARCIAL` |
| **FR-065** | `app/CloseDialog.svelte`, `platform/lifecycle.rs` | T123 | `AppLifecycle.test.ts` (el componente real es `CloseDialog.svelte`, no `AppLifecycle.svelte`) | `VERIFICADO` |
| **FR-066** | `.github/workflows/ci.yml` | T156, T167 | Workflow completo con build release y verificación de hash; **no se ha ejecutado en GitHub Actions, solo localmente por partes** | `PARCIAL` |
| **FR-067** | `LICENSE`, `THIRD_PARTY_NOTICES.md`, `engine/LICENSE` | T013, T131 | `scan-security.mjs` (inventario de dependencias, licencias) | `VERIFICADO` |

---

## 2. Criterios de éxito medibles (SC-001 a SC-015)

| ID | Criterio | Evidencia real | Estado |
|---|---|---|---|
| **SC-001** | Dos usuarios completan conexión, verificación, aceptación y prueba estándar sin ayuda | Emparejamiento verificado extremo a extremo (T143); la prueba estándar completa entre dos peers reales no se ha ejecutado (T144 parcial) | `PARCIAL` |
| **SC-002** | Resultado en ≤ 75 s tras aceptación | Sin medición end-to-end real que lo cronometre; el plan estándar (~55 s) y el resto son cifras de diseño, no cronometradas (T165 pendiente de unificar la cuenta) | `NO PRESENTE` |
| **SC-003** | 100 % de sesiones completadas con mismo ID/plan/cifras en ambos extremos | Dos servicios completos en bucle local persisten el mismo `session_id` y las mismas cifras oficiales en ambos extremos (§1.16); sin dos equipos reales (T180) | `PARCIAL` |
| **SC-004** | Cancelación ≤ 2 s, libera recursos, estado terminal explícito | `cargo test --test process_cleanup` (limpieza); la cancelación local vuelve en < 2 s con motor en marcha (§1.16) y en ~0,45 s con `CANCEL`/`CANCEL_ACK` real antes de arrancarlo (§1.19); sin `CANCEL` cooperativo con el motor ya corriendo (declarado, no resuelto) | `VERIFICADO` |
| **SC-005** | 100 % de resultados insuficientes muestran «no evaluable» | `test_sin_receptor_no_hay_velocidad_oficial`, `diagnostic::capacity` (sin `refBps` no hay veredicto) | `VERIFICADO` |
| **SC-006** | 100 % de errores visibles con explicación, acción y código | `errors/mod.rs` catálogo completo, `ErrorResolution.test.ts` | `VERIFICADO` |
| **SC-007** | Ningún recorrido ordinario menciona el motor fuera de detalles/licencias | Inspección de `locales/*.json` y pantallas: NTTTCP solo en `AboutScreen.svelte` y detalles técnicos | `VERIFICADO` |
| **SC-008** | Todas las pantallas funcionan por teclado y escalan a 200 % | `e2e/accessibility/a11y.spec.ts`, `e2e/visual/visual.spec.ts` (T147): 9 pruebas reales en verde contra un build de producción, con axe-core real | `PARCIAL` — cubre Inicio/Historial/Ajustes y el diálogo de conexión manual; el resto de pantallas (Sesión, Resultado, consentimiento) sin ejercitar aquí |
| **SC-009** | Español e inglés con las mismas claves y formatos/plurales coherentes | `check-locales.mjs`: 369 claves sincronizadas, marcadores `{placeholder}` coherentes en las 7 claves interpoladas; sin claves de plural que verificar todavía (T169) | `VERIFICADO` |
| **SC-010** | < 5 % de CPU y ≤ 4 Hz de refresco durante la prueba | `sampling/aggregate.rs` limita a ≤4 Hz por diseño; `scripts/test/performance.ps1` (T159/T168) sigue el método exacto de la constitución (% de un procesador lógico, NTTTCP excluido, hardware documentado) y mide CPU real de la app release en reposo: 0,531 % con animación, 0,374 % sin animación — falta medirla durante `RUNNING_*` | `PARCIAL` |
| **SC-011** | Ninguna entrada inválida/repetida/fuera de estado tiene efecto | `cargo test --test protocol_abuse`, validación Zod/Serde en fixtures | `VERIFICADO` |
| **SC-012** | Toda migración conserva backup restaurable | `cargo test --test history_migrations` (`.v1.bak`, rechazo de esquema futuro) | `VERIFICADO` |
| **SC-013** | La anonimización elimina el 100 % de identificadores sintéticos | `cargo test export::redact_tests` (fixture con IP/MAC/huellas anidadas, sin fugas) | `VERIFICADO` |
| **SC-014** | Instalación y primer arranque sin Internet | WebView2 offline embebido (209,9 MiB) y sidecars locales; instalación real sin ejecutar | `PARCIAL` |
| **SC-015** | V-01 a V-12 cerradas con evidencia empírica antes de publicar v1 | `VALIDACION.md` §3: **2 de 12 cerradas** (V-03, V-06), **2 parciales** (V-04, V-12: medidas en reposo, T159). Las otras ocho, `NO PRESENTE` con su motivo | `NO PRESENTE` |

---

## 3. Lo que este documento no resuelve

Corregir esta matriz no cierra ninguna de las tareas abiertas que cita: T144 (orquestación
completa entre dos peers), T147 (E2E y accesibilidad reales), T150/T151/T154/T159/T160
(eventos, mDNS, elevación del helper, arneses reales, checks de arquitectura), T165/T166
(inconsistencias internas de `tasks.md`) y T168–T171 (definiciones y pruebas pendientes).
Su función es que la trazabilidad deje de afirmar cobertura que no existe; no sustituye a
esas tareas.
