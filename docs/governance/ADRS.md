# Architecture Decision Records iniciales

**Estado común:** propuestos. Una propuesta no autoriza implementación.

Cuando se acepten, cada ADR registrará responsable, fecha, evidencia y artefactos afectados.
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

**Status.** Propuesto; el stack procede de requisitos, su compatibilidad aún no está verificada.

## ADR-002 — Monolito modular orientado a features

**Context.** El proyecto debe soportar trabajo paralelo y evitar carpetas técnicas globales
que acumulen lógica y conflictos. `Historias.md` §25 propone una estructura mayormente técnica.

**Decision.** Organizar el frontend por features con `index.ts` público y módulos Rust por
capacidad. Separar dentro de cada módulo reglas, servicio y adaptadores solo cuando exista esa
responsabilidad. Un crate principal y un helper separado.

**Alternatives considered.** Organización exclusivamente por capas, Clean Architecture
completa y un crate por feature.

**Consequences.** Ownership e imports más claros, sin multiplicar proyectos. Requiere
reconciliar expresamente `Historias.md` §25 antes del bootstrap.

**Status.** Propuesto.

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

**Status.** Propuesto.

## ADR-004 — Persistencia local y recuperable

**Context.** Identidad, peers y resultados deben persistir desde H1; habrá migraciones y
recuperación ante fallos.

**Decision.** SQLite con rusqlite, WAL, claves foráneas, propietario único de esquema,
transacciones y backup coherente. Ajustes en JSON versionado y atómico; secretos con DPAPI.

**Alternatives considered.** IndexedDB/localStorage, JSON para todo, ORM y base externa.

**Consequences.** SQL y migraciones explícitos; pruebas reales de WAL, reinicio, backup y
restauración. No se añade servicio ni ORM.

**Status.** Propuesto; la tecnología procede de requisitos.

## ADR-005 — Confianza y privilegio mínimo

**Context.** Descubrimiento y tráfico local no son confiables; configurar firewall requiere
elevación, pero la aplicación principal no debe ejecutarse elevada.

**Decision.** Identidad criptográfica, emparejamiento verificado, autorización por operación,
validación repetida en Rust y helper elevado limitado a operaciones allowlisted.

**Alternatives considered.** Confianza por IP/hostname, elevación de toda la aplicación y
cuentas centrales.

**Consequences.** Se requieren pruebas hostiles y de consentimiento. mDNS nunca concede
confianza. G2 debe cerrar el primer contacto y la reconexión.

**Status.** Propuesto; principios derivados de requisitos de seguridad.

## ADR-006 — Compatibilidad y publicación reproducible

**Context.** Windows, WebView2 Evergreen, NTTTCP, toolchains y runner pueden evolucionar de
forma independiente. Una etiqueta publica un instalador y manifiesto de actualización.

**Decision.** Versiones directas y acciones fijadas; instalación sin red; artefactos por
versión/hash; publicación explícita desde etiqueta; probar el instalador exacto distribuido.

**Alternatives considered.** `latest` flotante, bootstrap online obligatorio, portable y
actualización silenciosa.

**Consequences.** Se necesita una matriz Windows/WebView2 identificada y laboratorio. Q1,
Q3 y G5 siguen pendientes.

**Status.** Propuesto.

No se crean ADR para formato, nombres de archivos o componentes triviales.
