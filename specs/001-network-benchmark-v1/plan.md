# Plan de implementación: NetworkBench v1

**Branch**: `001-network-benchmark-v1` *(identificador de feature; no existe rama Git)*  
**Date**: 2026-09-21  
**Spec**: [spec.md](./spec.md)  
**Constitution**: 0.7.0, no ratificada

## Resumen

NetworkBench v1 se entregará como un monolito modular de escritorio para Windows x64. Un
frontend Svelte presenta una proyección del estado; Rust conserva la autoridad sobre identidad,
consentimiento, sesión, validación, procesos, resultados y persistencia. Dos instancias se
comunican directamente por un canal TLS local, ejecutan NTTTCP como único motor y guardan un
resultado común en SQLite. No habrá nube, cuentas, servidor HTTP ni servicio en segundo plano.

El trabajo se divide en una base compilable y lotes H1, H2 y H3. Cada lote cierra contratos,
pruebas y evidencia antes de que otro amplíe el mismo límite. G1–G6 y V-01–V-12 son puertas de
investigación o laboratorio: bloquean únicamente el comportamiento que depende de ellas y no
se sustituyen por supuestos. Los detalles de las decisiones están en [research.md](./research.md).

## Contexto técnico (Technical Context)

**Language/Version**: Rust 1.98.1, edición 2024; TypeScript 6.0.3; Svelte 5.57.1; Node.js
24.21.0 LTS solo para desarrollo y build. Versiones documentadas, todavía no compiladas juntas.

**Primary Dependencies**: Tauri 2.11.x, Vite 8.3, Tailwind CSS 4.3, Zod 4.6, Tokio 1.53,
rustls/tokio-rustls, Serde, rusqlite con SQLite embebido, mdns-sd, quick-xml, `windows`,
WebView2 Evergreen y Microsoft NTTTCP 5.40 x64. Versiones exactas según la Constitución; G5
debe resolver lockfiles y compatibilidad antes de consolidarlas.

**Storage**: SQLite local en WAL para peers y sesiones; JSON versionado y atómico para ajustes;
DPAPI para secretos de identidad; logs locales rotados. La WebView no es almacenamiento
autoritativo.

**Testing**: `cargo test`, Vitest/V8, Testing Library, Playwright/axe-core, integración Rust con
recursos aislados y pruebas Windows/laboratorio. Tauri mock prueba lógica de integración, pero
no WebView2, UAC, firewall, instalación ni procesos nativos. El harness nativo se decide por
piloto. Umbrales de cobertura fijados por Q2 (constitución 0.7.0).

**Target Platform**: Windows 10 22H2 y Windows 11, solo x64. Target de build
`x86_64-pc-windows-msvc`. Instalador NSIS por máquina con WebView2 offline.

**Project Type**: aplicación de escritorio local, peer-to-peer, con frontend WebView y núcleo
nativo; un único crate `src-tauri` con dos binarios (`[[bin]]`): la aplicación y el helper
elevado de firewall en `src-tauri/helper/`. No hay workspace multi-crate.

**Performance Goals**: prueba estándar de unos 55 segundos; muestras cada 500 ms; presentación
a un máximo de 4 Hz; menos del 5 % de un procesador lógico para aplicación y procesos WebView2
atribuibles, excluido NTTTCP; cancelar no espera más de 2 segundos al ACK remoto.

**Constraints**: funcionamiento principal offline; una sesión activa; privilegio mínimo;
sin argumentos libres al motor; buffers, mensajes y colas acotados; precisión por encima de UI
y logging; español e inglés; escala y texto hasta 200 %; sin pérdida de precisión de enteros.

**Scale/Scope**: dos peers por sesión, 1–64 streams por sentido en secuencial y 1–32 en simultáneo (FR-042c),
historial local sin caducidad automática, H1–H3 completos para v1 y siete áreas de UI. No se
diseña para múltiples motores, cloud, Linux/macOS, ARM64 ni monitorización continua.

## Comprobación constitucional (Constitution Check)

### Gate inicial, antes de la fase 0

**Resultado**: APTO PARA PLANIFICACIÓN; NO APTO TODAVÍA PARA IMPLEMENTACIÓN COMPLETA.

| Área | Estado | Evidencia y condición |
|---|---|---|
| Autoridad | Pendiente de ratificación | Constitución 0.7.0 no ratificada. Q1 (0.6.0), Q2 (0.7.0) y la accesibilidad de Q3 (0.5.0) enmendadas; Q4 respondido en `spec.md`. ADR-001–ADR-006 aceptados. |
| Arquitectura | Cumple para diseño | Monolito modular, Rust autoritativo, Svelte presentacional y adaptadores estrechos. ADR-002 aceptado el 2026-09-21. |
| Versiones | Pendiente G5 | La línea base está documentada; faltan manifiestos, lockfiles, build, instalador y arranque reales. |
| Seguridad | Cumple | mTLS por identidad, consentimiento, validación repetida, capabilities mínimas y helper allowlisted. |
| Datos | Cumple | Propietario único de SQLite, migraciones inmutables, backup/restauración y rechazo de esquema futuro. |
| Accesibilidad | Cumple | Constitución 0.5.0 (VI): básica desde H1 (teclado, foco, contraste, texto además de color, reducción de movimiento); revisión completa con Narrador, alto contraste y 200 % en H2. Coincide con FR-042e. |
| Rendimiento | Pendiente V-04/V-12 | Presupuesto y método definidos; falta evidencia en hardware real. |
| Pruebas | Cumple | Estrategia y herramientas definidas; umbrales Q2 fijados en la constitución 0.7.0 (T132 los configura tras medir la baseline). |
| Distribución | Cumple | `spec.md` FR-042d alineado el 2026-09-21: `latest.json` estable cuyo artefacto apunta a la URL inmutable de la etiqueta `vX.Y.Z`. T125 verifica esa forma; no queda conflicto. |
| Puertas técnicas | Pendiente | G1–G6 y V-01–V-12 tienen responsable, lote y criterio de cierre en este plan. |

No se introduce una excepción. Los conflictos no impiden diseñar ni ejecutar trabajo
independiente; sí impiden declarar cerrado el lote que materialice la regla en disputa.

### Gate posterior a la fase 1

**Resultado**: APTO PARA GENERAR TAREAS CON BLOQUEOS EXPLÍCITOS.

- Los contratos separan IPC, protocolo peer y resultado, y no congelan campos afectados por
  G1–G4 hasta obtener evidencia.
- El modelo mantiene datos ausentes como tales, versiona límites y evita duplicar autoridad.
- `quickstart.md` distingue comprobaciones disponibles, futuras y de laboratorio.
- Q2 y ADR-001–ADR-006 quedaron decididos el 2026-09-21; solo la ratificación constitucional
  y Q4 siguen pendientes del propietario. Los conflictos de accesibilidad y
  updater quedaron resueltos el 2026-09-21 (constitución 0.5.0 y FR-042e/FR-042d).
- No quedan marcadores de aclaración; los pendientes son gates registrados, no supuestos.

## Estrategia de entrega

| Lote | Alcance | Prerrequisitos | Checkpoint observable |
|---|---|---|---|
| L00 | Bootstrap, contratos, configuración, logging y harness | ADR-001/002 aceptados; G5 | Build reproducible y smoke instalado sin afirmar compatibilidad no ejecutada |
| L01 | Shell, navegación, i18n, tema, ventana y geometría | L00; accesibilidad básica desde H1 (constitución 0.5.0, VI) | UI mínima es/en y comportamiento de ventana probado en Windows |
| L02 | Identidad, descubrimiento, conexión manual, pairing y consentimiento | L00/L01; G2 parcial | Dos identidades se emparejan/rechazan sin que mDNS conceda confianza |
| L03 | Plan, NTTTCP, estados, TCP bidireccional, cancelación y resultado básico | L02; G1–G3 | Dos instancias completan/cancelan una prueba sin procesos huérfanos |
| L04 | Persistencia H1, migración y recuperación | Contratos L02/L03 | Reinicio conserva peers/resultados y fallo de migración restaura backup |
| L05 | Muestreo, diagnóstico, gráficas y detalle | L03/L04; G4, V-04/V-06/V-07 | Resultado explica hechos sin inventar causas y cumple presupuesto medido |
| L06 | Firewall y errores completos | L02/L03; helper y entorno Windows | Consentimiento/UAC aceptado y rechazado, reglas propias y cleanup |
| L07 | Historial, comparación y exportación | L04/L05; V-09 | Filtros/cohortes y exportaciones reales con anonimización |
| L08 | Avanzado, UDP y simultáneo | L03/L06; G1, V-01/V-02/V-03/V-05; V-01 valida los límites 64/32 de FR-042c | Límites y pérdida UDP validados en dos equipos |
| L09 | Ajustes, bandeja, cierre, autoarranque y updater | L01/L04/L06; forma del updater fijada en FR-042d y registrada por T125 | Ciclo de vida nativo y actualización auténtica/pospuesta |
| L10 | Distribución y cierre v1 | Todos; G5 y V-01–V-12 | Instalador exacto probado, evidencias registradas y release autorizable |

Cada lote implementa comportamiento y pruebas conjuntamente. `tasks.md` debe agrupar trabajo
por estos checkpoints, no crear primero toda la aplicación y después todos los tests.

## Estructura del proyecto

### Documentación de esta feature

```text
specs/001-network-benchmark-v1/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── README.md
│   ├── ipc.md
│   ├── peer-protocol.md
│   └── session-result.md
├── checklists/
│   ├── architecture.md
│   └── requirements.md
└── tasks.md
```

### Código fuente futuro

```text
src/
├── main.ts
├── app/
├── features/
│   ├── peers/
│   ├── session/
│   ├── results/
│   ├── history/
│   ├── export/
│   └── settings/
└── lib/
    ├── api/
    ├── contracts/
    ├── components/
    ├── design-system/
    ├── i18n/
    ├── config/
    ├── logging/
    └── window.ts

src-tauri/
├── src/
│   ├── app.rs
│   ├── config/
│   ├── ipc/
│   ├── model/
│   ├── control/
│   ├── pairing/
│   ├── identity/
│   ├── discovery/
│   ├── engine/ntttcp/
│   ├── sampling/
│   ├── netinfo/
│   ├── diagnostic/
│   ├── history/
│   ├── settings/
│   ├── export/
│   ├── firewall/
│   ├── platform/
│   ├── logging/
│   ├── updater/
│   └── errors/
├── helper/                      # segundo binario del mismo crate
├── nsis/
├── permissions/
├── capabilities/
└── tests/

engine/
locales/
tests/{contracts,fixtures,windows}/
e2e/
scripts/{architecture,security,package,test}/
artifacts/validation/            # evidencia de consolidación (T130–T136)
```

**Decisión de estructura**: feature-first en frontend y módulos por capacidad en Rust, según
`ARCHITECTURE.md`. Una feature exporta solo `index.ts`; Rust expone servicios/traits estrechos.
No se crean carpetas vacías: cada lote añade solo rutas necesarias. ADR-002 (aceptado) registra
la evolución respecto a la estructura técnica de `Historias.md` §25, que ya lo anota.

## Fronteras y APIs públicas

- `app` compone features; ninguna feature importa estado privado de otra.
- `session` y `history` pueden consumir la API pura de `results`; el resto de aristas entre
  features requiere Architecture Check.
- `lib/api` es el único transporte Tauri del frontend. Los comandos se agrupan por área y
  validan entrada y salida con schemas locales al contrato.
- `ipc` convierte DTO a tipos de dominio, aplica autorización y delega en servicios; no contiene
  reglas de negocio ni SQL.
- `control` posee la única sesión activa y serializa mandatos; muestras usan una vía acotada que
  no retrasa cancelación.
- `engine/ntttcp` es el único módulo que construye argumentos o analiza XML.
- `history` es el único propietario de esquema y SQL; `export` recibe resultados explícitos y
  no consulta la base ni recalcula veredictos.
- El helper elevado solo acepta operaciones de firewall allowlisted y nunca toca UI, sesión o BD.

Los contratos públicos iniciales se describen en [contracts/](./contracts/). Cambiar versión,
semántica, límites o compatibilidad exige fixtures válidos/inválidos y revisión de consumidores.

## Datos, migración y compatibilidad

- `data-model.md` es el modelo conceptual; SQLite puede normalizar consultas frecuentes y
  conservar snapshots JSON versionados sin crear un ORM.
- Una transacción persiste sesión, direcciones, muestras e interfaces de forma atómica e
  idempotente por `sessionId`.
- Las migraciones son numeradas e inmutables una vez distribuidas. Antes de migrar se crea y
  verifica un backup coherente con WAL; ante fallo se preservan original y backup.
- Una versión antigua rechaza un esquema futuro. El historial no recalcula resultados con reglas
  nuevas; muestra el snapshot y sus versiones.
- Aplicación, protocolo peer, `SessionResult`, esquema SQLite y ajustes tienen versiones
  independientes.
- Enteros que puedan superar el rango seguro de JavaScript cruzan JSON como decimal en cadena.

## Plan de seguridad y privacidad

Actores no confiables: WebView, peer, mDNS, DNS, sockets, XML/stdout/stderr del motor, SQLite,
archivos importados/exportados, manifiesto de actualización, variables y texto del sistema.

Controles:

1. Validación estructural y semántica en cada frontera; Rust vuelve a validar antes del efecto.
2. Capabilities Tauri explícitas por ventana, permisos positivos/negativos y sin shell/SQL/filesystem
   genéricos. Registrar un comando no autoriza su invocación.
3. mTLS e identidad por huella; emparejamiento humano antes de confianza; autoaceptación opt-in.
4. Límites de tamaño, profundidad, frecuencia, conexiones, buffers, streams y rangos de puertos.
5. NTTTCP por ruta absoluta, hash, argumentos estructurados y Job Object con propiedad probada.
6. Helper elevado separado, consentimiento local y allowlist; la app principal nunca se eleva.
7. Logs/diagnósticos saneados y acotados; sin secretos, huellas, `sessionId` ni payload crudo.
8. Exportación informa y anonimiza recursivamente; CSV neutraliza fórmulas.
9. Actualización auténtica, desactivable, confirmada y nunca durante sesión activa.

El threat model y las pruebas deben cubrir replay, duplicados, mensajes fuera de estado, identidad
cambiada, inundación, inyección, sustitución de ejecutables y fallos durante cleanup/migración.

## Pruebas y calidad

El nivel más barato prueba cada riesgo: reglas puras con unitarios; contratos con fixtures
cruzados; SQLite/TLS/procesos con integración; WebView y foco en navegador; UAC, ventana,
instalación y red real en Windows/laboratorio.

El futuro `pnpm verify` agregará, en orden: instalación locked, formato/lint/typecheck, checks
de arquitectura y contratos, builds, unitarios/integración/cobertura y escaneos. Hoy ese comando
NO PRESENTE debe crearse en L00 antes de cerrar la primera implementación. E2E y laboratorio se
ejecutan por riesgo y checkpoint, no en cada edición.

Q2 (constitución 0.7.0) fija 80 % general por lenguaje/métrica y 90 % de líneas en módulos
críticos Rust. L00 mide primero la baseline completa; T132 configura el gate con esos umbrales y
ningún porcentaje inferior se interpreta como aprobación.

## Justificación de dependencias

| Dependencia/capacidad | Necesidad | Alternativa rechazada | Riesgo y prueba |
|---|---|---|---|
| Tauri + WebView2 | UI local con núcleo Rust y APIs Windows | Electron añade runtime/consumo; UI nativa duplica sistema de diseño | G5, arranque e instalador en matriz real |
| Tokio + rustls | protocolo TLS asíncrono y cancelable | sockets bloqueantes complican control; TLS del SO no elimina framing/estado | integración con dos peers, timeouts y abuso |
| rusqlite bundled | historial transaccional y migrable | JSON no ofrece consultas/atomicidad; ORM añade abstracción | WAL real, corrupción, backup y downgrade |
| Zod + Serde | validar ambos lados de IPC/JSON | tipos TypeScript no validan runtime; codegen aún no justificado | fixtures válidos/inválidos compartidos |
| mdns-sd | descubrimiento LAN estándar | implementación propia aumenta riesgo; escaneo IP invade red | V-08 y fallback manual |
| quick-xml | parser acotado de salida del motor | DOM completo y regex son más costosos/frágiles | G1/V-03 con XML real |
| `windows` + helper | APIs nativas y elevación mínima | app completa elevada viola privilegio mínimo | pruebas Windows/UAC y allowlist |
| Playwright + axe-core | recorridos UI, foco y accesibilidad | DOM simulado no prueba CSS/foco real | bridge explícito; no sustituye E2E Tauri |

Licencia, mantenimiento, vulnerabilidades, tamaño y compatibilidad se comprueban al resolver
lockfiles en L00. No se instala una segunda librería de estado, gráficas, iconos, logging, SQL,
validación o testing sin una justificación nueva y Architecture Check.

## Reglas de desarrollo paralelo

- Un escritor por archivo y lote; contratos acordados antes de separar proveedor/consumidor.
- UI puede avanzar con fixtures versionados mientras Rust implementa el contrato; la integración
  real ocurre antes del checkpoint.
- Lockfiles, migraciones, registro IPC, locales, capabilities, configuración y CI tienen un
  integrador único por lote.
- L01 puede avanzar con L02 tras L00; parser NTTTCP y componentes de resultado pueden avanzar en
  paralelo tras fijar fixtures; L04 espera el snapshot estable de L03.
- G1/G2/G3 coordinan motor/protocolo/resultado; nadie cambia uno desde una feature consumidora.
- Cada tarea declara rutas de escritura, API consumida, contrato modificado, pruebas y recurso
  compartido. No formatea ni regenera el repositorio completo.
- Se valida el resultado integrado; una prueba aislada de cada rama no prueba el contrato final.

## Control de complejidad

No se solicita ninguna violación arquitectónica. El coordinador, los puertos de I/O, los
schemas, el propietario SQLite y el helper existen para resolver concurrencia, testabilidad,
validación, integridad y privilegio mínimo concretos. No se añaden microservicios, bus genérico,
ORM, DI container, CQRS, event sourcing, cache, plugins, múltiples crates ni segundo motor.

Los conflictos normativos registrados requieren decisión o enmienda; no se justifican como
complejidad necesaria ni se implementan silenciosamente.
