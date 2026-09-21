# Evaluación arquitectónica

**Estado:** propuesta, 2026-09-21.

## Estado comprobado

NetworkBench será una aplicación Windows x64 para medir una conexión entre dos equipos y
explicar si funciona como debería. Sus usuarios incluyen personal técnico y técnicos junior;
por eso combina un recorrido sencillo con detalle progresivo.

El stack previsto es Tauri 2 + Rust, frontend Svelte 5/TypeScript/Vite/Tailwind CSS 4 en
WebView2, SQLite local y Microsoft NTTTCP empaquetado. No hay nube, cuentas, telemetría,
servidor central ni backend HTTP. Cada instancia puede iniciar o recibir una prueba.

La aplicación todavía no existe. No hay manifiestos, lockfiles, fuentes de aplicación,
build, lint, tests de producto ni CI. `Design/` contiene documentación y componentes de
referencia sin build. Las versiones constitucionales son una línea base documentada, no una
combinación compilada. El número previsto de desarrolladores no está especificado.

La complejidad real está en:

- consentimiento, confianza e identidad entre peers;
- coordinación distribuida y cancelación segura;
- ejecución y limpieza de procesos nativos;
- precisión de resultados y límites provisionales;
- persistencia y migraciones recuperables;
- accesibilidad y comportamiento Windows real;
- instalación, firewall y actualización con privilegio mínimo.

No existe necesidad demostrada de arquitectura distribuida, abstracciones empresariales,
multi-tenant, escalado horizontal o múltiples motores en v1.

## Decisiones que deben tomarse ahora

- Monolito modular y dirección de dependencias.
- Rust como autoridad de negocio y seguridad; Svelte como presentación.
- APIs públicas de features y prohibición de imports profundos.
- Ownership único de SQLite, configuración, logging y proceso de benchmark.
- Contratos separados para IPC, protocolo peer, resultados y persistencia.
- Architecture Check, política de dependencias y Definition of Done.
- Aceptación o rechazo de ADR-002, que evoluciona la estructura de `Historias.md` §25.

## Decisiones que dependen de evidencia

- Q1–Q4 de la constitución: Windows, cobertura, alcance y tema inicial.
- G1: capacidades y salida reales de NTTTCP.
- G2: orden, identidad, aceptación, reconexión e idempotencia del protocolo.
- G3: tamaño de resultados, temporización y reconciliación.
- G4: fronteras de interpretación y cohortes comparables.
- G5: compatibilidad real de toolchains, WebView2, Windows e instalador.
- G6: referencias a contenido antiguo no localizado.
- Harness nativo y matriz de laboratorio realmente disponibles.

## Decisiones pospuestas deliberadamente

- Segundo motor de benchmark.
- Linux, macOS, ARM64, servicio Windows o modo portable.
- Cloud, cuentas, telemetría o gestión central.
- Generador de contratos Rust/TypeScript.
- Separación en varios crates.
- Pool complejo de conexiones o cache de resultados.
- Framework de actores, bus de eventos general o plugins.
- Mutation testing obligatorio y su umbral.

Las señales para reconsiderarlas son requisitos aprobados, un consumidor independiente,
límites que el compilador deba reforzar, consultas lentas medidas, fallos repetidos de
contratos o coste de build demostrado. «Podría necesitarse» no es evidencia suficiente.

## Riesgos detectados

| Riesgo | Consecuencia | Tratamiento |
|---|---|---|
| Constitución no ratificada | Reglas propuestas pueden confundirse con aprobación | Estado visible en spec, plan y cierre |
| G1–G6 abiertos | Oráculos o contratos ficticios | Bloquear solo la funcionalidad afectada |
| `Historias.md` §25 frente a feature-first | Dos estructuras incompatibles | Resolver ADR-002 y reconciliar la fuente |
| Diseño elige NIC más rápida y producto exige ruta al peer | Medición sobre interfaz incorrecta | Aplicar requisito de producto y corregir referencia autorizadamente |
| Ejemplos de SvelteKit en Design | LLM puede introducir APIs legacy/no aplicables | Convenciones explícitas y controles de imports |
| Capabilities Tauri mal entendidas | Comandos propios accesibles indebidamente | Manifiesto explícito y pruebas de denegación |
| Estado global duplicado TS/Rust | Carreras y autorizaciones inconsistentes | Rust autoritativo; UI como proyección |
| SQLite en async | Bloqueo de workers o pérdida de datos | Trabajador dedicado, cola acotada y transacciones |
| Dos equipos sin transacción distribuida | Historiales divergentes tras corte | G3 y estado de reconciliación explícito |
| Agentes editando archivos centrales | Conflictos y degradación gradual | Ownership por lote e integrador de recursos compartidos |
| Inventario documental envejecido | Decisiones basadas en estado antiguo | Evidencia actual y fecha; actualizar metadatos por tarea acotada |

## Desarrollo paralelo

Cada lote declara objetivo, feature, responsable, archivos de escritura, APIs consumidas,
contratos modificados, dependencias, suites, recursos compartidos e integrador.

Reglas:

1. Un escritor por archivo durante el lote.
2. Acordar contratos antes de paralelizar proveedor y consumidor.
3. UI trabaja contra fixtures aprobados; dominio trabaja sin UI ni plataforma real.
4. Integración prueba después la conexión real.
5. Lockfiles, migraciones, registro IPC y configuración tienen integrador único.
6. No regenerar ni formatear el repositorio completo desde una tarea local.
7. No sobrescribir trabajo ajeno ni usar una rama/worktree sin autorización Git específica.
8. Validar el resultado integrado, no solo cada fragmento aislado.

Pueden avanzarse en paralelo módulos con contratos estables, como parser y componentes de
resultado. Deben coordinarse cambios de `SessionResult`, protocolo, migraciones, dependencias,
capabilities y publicación.

## Integración con SpecKit

```text
bootstrap de gobernanza
  → constitution
  → specify (WHAT/WHY)
  → clarify
  → plan (HOW + Architecture Check)
  → tasks (lotes y ownership)
  → analyze
  → implement
  → converge
```

Cada artefacto identifica la versión constitucional y su ratificación. Una feature no puede
convertir un pendiente en decisión aprobada. Las plantillas gestionadas por SpecKit no se
modifican para aplicar esta gobernanza: los planes enlazan `PLAN-CHECK.md`.

