# Architecture Decision Records iniciales

**Estado común:** ADR-001 a ADR-006 aceptados el 2026-09-21 por el propietario. La aceptación
autoriza el bootstrap (L00); la compatibilidad técnica sigue sometida a G5.

Cada ADR registra responsable, fecha, evidencia y artefactos afectados.
Una decisión posterior la sustituye mediante otro ADR; no se reescribe su historia.

## ADR-001 — Runtime y distribución local

**Context.** El producto integra APIs Windows, procesos, WebView2, red local y un instalador,
sin nube ni servicio central.

**Decision.** Usar Tauri 2 + Rust y Svelte 5/TypeScript/Vite dentro de WebView2. Empaquetar
NTTTCP como único motor de v1. Mantener Node fuera del runtime instalado.

**Alternatives considered.** Electron, UI totalmente nativa y aplicación web acompañada de
un servicio local.

**Consequences.** Existen dos lenguajes y una frontera IPC. Se obtiene acceso nativo sin
añadir servidor. La compatibilidad conjunta y con Windows mínimo sigue sometida a G5.

**Status.** Aceptado el 2026-09-21 por el propietario (Daniel Diez Mardomingo) en sesión de planificación; registrado en `specs/001-network-benchmark-v1/checklists/architecture.md`. El stack procede de requisitos; su compatibilidad la verifica G5 (T002/T014).

## ADR-002 — Monolito modular orientado a features

**Context.** El proyecto debe soportar trabajo paralelo y evitar carpetas técnicas globales
que acumulen lógica y conflictos. `Historias.md` §25 propone una estructura mayormente técnica.

**Decision.** Organizar el frontend por features con `index.ts` público y módulos Rust por
capacidad. Separar dentro de cada módulo reglas, servicio y adaptadores solo cuando exista esa
responsabilidad. Un único crate `src-tauri` con dos binarios (`[[bin]]`): la aplicación y el
helper elevado de firewall; sin workspace multi-crate.

**Alternatives considered.** Organización exclusivamente por capas, Clean Architecture
completa y un crate por feature.

**Consequences.** Ownership e imports más claros, sin multiplicar proyectos. `Historias.md`
§25 queda reconciliado: anota que la estructura vigente es la de `ARCHITECTURE.md`.

**Status.** Aceptado el 2026-09-21 por el propietario (Daniel Diez Mardomingo) en sesión de planificación; registrado en `specs/001-network-benchmark-v1/checklists/architecture.md`. Artefactos afectados: `ARCHITECTURE.md`, `plan.md` §Estructura, `Historias.md` §25.

## ADR-003 — Autoridad de estado y contratos

**Context.** Dos peers, una sesión activa, eventos asíncronos, reconexión y procesos pueden
producir órdenes duplicadas o fuera de estado.

**Decision.** Rust es autoridad de sesión, autorización y resultados. Un coordinador por
instancia serializa mandatos. Svelte mantiene una proyección mediante snapshots/eventos.
IPC y protocolo peer tienen contratos y versiones independientes.

**Alternatives considered.** Estado autoritativo TypeScript, objetos globales con locks y un
bus genérico de eventos.

**Consequences.** Se deben probar orden, duplicados, cancelación y recuperación. G2 y G3
cierran los detalles de protocolo; este ADR no los anticipa.

**Status.** Aceptado el 2026-09-21 por el propietario (Daniel Diez Mardomingo) en sesión de planificación; registrado en `specs/001-network-benchmark-v1/checklists/architecture.md`.

## ADR-004 — Persistencia local y recuperable

**Context.** Identidad, peers y resultados deben persistir desde H1; habrá migraciones y
recuperación ante fallos.

**Decision.** SQLite con rusqlite, WAL, claves foráneas, propietario único de esquema,
transacciones y backup coherente. Ajustes en JSON versionado y atómico; secretos con DPAPI.

**Alternatives considered.** IndexedDB/localStorage, JSON para todo, ORM y base externa.

**Consequences.** SQL y migraciones explícitos; pruebas reales de WAL, reinicio, backup y
restauración. No se añade servicio ni ORM.

**Status.** Aceptado el 2026-09-21 por el propietario (Daniel Diez Mardomingo) en sesión de planificación; registrado en `specs/001-network-benchmark-v1/checklists/architecture.md`. La tecnología procede de requisitos (constitución IV).

## ADR-005 — Confianza y privilegio mínimo

**Context.** Descubrimiento y tráfico local no son confiables; configurar firewall requiere
elevación, pero la aplicación principal no debe ejecutarse elevada.

**Decision.** Identidad criptográfica, emparejamiento verificado, autorización por operación,
validación repetida en Rust y helper elevado limitado a operaciones allowlisted.

**Alternatives considered.** Confianza por IP/hostname, elevación de toda la aplicación y
cuentas centrales.

**Consequences.** Se requieren pruebas hostiles y de consentimiento. mDNS nunca concede
confianza. G2 debe cerrar el primer contacto y la reconexión.

**Status.** Aceptado el 2026-09-21 por el propietario (Daniel Diez Mardomingo) en sesión de planificación; registrado en `specs/001-network-benchmark-v1/checklists/architecture.md`. Principios derivados de la constitución III.

## ADR-006 — Compatibilidad y publicación reproducible

**Context.** Windows, WebView2 Evergreen, NTTTCP, toolchains y runner pueden evolucionar de
forma independiente. Una etiqueta publica un instalador y manifiesto de actualización.

**Decision.** Versiones directas y acciones fijadas; instalación sin red; artefactos por
versión/hash; publicación explícita desde etiqueta; probar el instalador exacto distribuido.

**Alternatives considered.** `latest` flotante, bootstrap online obligatorio, portable y
actualización silenciosa.

**Consequences.** Se necesita una matriz Windows/WebView2 identificada y laboratorio. Q1
(Windows 10 22H2 y 11 x64) y el updater (FR-042) están decididos; G5 sigue pendiente.

**Status.** Aceptado el 2026-09-21 por el propietario (Daniel Diez Mardomingo) en sesión de planificación; registrado en `specs/001-network-benchmark-v1/checklists/architecture.md`.

## ADR-007 — Arquitectura del actualizador: manifiesto estable con artefactos inmutables y no intrusivos

**Context.** La aplicación debe poder comprobar y aplicar actualizaciones de software en Windows sin comprometer la integridad de las mediciones de red en curso ni permitir descargas de orígenes no verificados o mutables.

**Decision.** El canal de actualización consulta un manifiesto estable `latest.json`, cuyos enlaces de descarga deben apuntar estrictamente a artefactos inmutables alojados en URLs versionadas (`releases/download/vX.Y.Z/…`), prohibiendo enlaces a binarios mutables (como `latest.exe`). El manifiesto debe incluir versión semántica estricta (rechazo de versiones iguales o downgrades), hash criptográfico / firma, y notas de versión. Las actualizaciones son opt-in/configurables; si existe una sesión de medición activa en el coordinador, cualquier comprobación o aviso de actualización se pospone de forma obligatoria hasta que la sesión concluya o sea cancelada, garantizando cero interrupciones en la toma de métricas (FR-042).

**Alternatives considered.** Actualización en segundo plano silenciosa con reinicio forzado, descarga de binarios desde enlaces flotantes (`/latest/download`), y auto-sustitución directa sin comprobación de estado de ejecución.

**Consequences.** Se implementa verificación semántica y validación de URLs en Rust (`src-tauri/src/updater/`), cobertura de pruebas en `src-tauri/tests/updater.rs` (T116), e integración con la máquina de estados para diferir avisos mientras la sesión esté activa.

**Status.** Aceptado el 2026-09-22 por el propietario (Daniel Diez Mardomingo); registrado en `specs/001-network-benchmark-v1/checklists/architecture.md`.

No se crean ADR para formato, nombres de archivos o componentes triviales.

## Desviaciones de la línea base técnica registradas en Bootstrap (L00 / T004)

- **Gestor de paquetes pnpm**: la línea base constitucional sugería `12.5.1`. Se adopta `pnpm@11.6.0` (versión verificada en el host local y compatible con Node 24), resolviendo `pnpm-lock.yaml` de forma exitosa y reproducible.
- **Identificador de binario auxiliar en Cargo**: Cargo prohíbe caracteres de punto (`.`) en nombres de crates y binarios. El binario auxiliar de firewall en `src-tauri/Cargo.toml` se declara como `networkbench-firewall-helper` (mapeando a `networkbench-firewall-helper.exe`), en sustitución de la denominación con punto `NetworkBench.FirewallHelper`.
- **Rust Toolchain**: Se fija formalmente `1.98.1` en `rust-toolchain.toml`, verificado presente en el entorno con `rustc 1.98.1`.
