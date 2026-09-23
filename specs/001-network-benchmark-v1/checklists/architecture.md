# Comprobación arquitectónica: NetworkBench v1

**Plan**: [plan.md](../plan.md)  
**Revisado**: 2026-09-21  
**Constitución**: 0.7.0, no ratificada  
**Resultado**: apto para generar tareas con bloqueos explícitos; no autoriza implementación

## Autoridad y alcance

| Comprobación | Estado | Evidencia / responsable |
|---|---|---|
| Identifica versión y ratificación constitucional | Cumple | Cabecera y comprobación constitucional |
| Enlaza requisitos, hitos y resultados | Cumple | `spec.md`, lotes y quickstart |
| Declara Q/G/V y cierre | Cumple | comprobación, investigación y tabla de lotes |
| No trata una propuesta como aprobación | Cumple | Q1–Q3 y los conflictos se cerraron por decisión registrada; G1–G6 siguen como gates |
| Mantiene WHAT/WHY en spec y HOW en plan | Cumple | No añade alcance de producto |

## Límites y dependencias

| Comprobación | Estado | Evidencia / responsable |
|---|---|---|
| Responsabilidades asignadas a módulos | Cumple | estructura y fronteras |
| APIs, consumidores y propietarios de escritura declarados | Cumple | contratos y reglas paralelas |
| Sin imports profundos, ciclos o aristas prohibidas | Cumple | API de feature y puertos Rust |
| Rust autoriza/decide; Svelte presenta | Cumple | R02 y contrato IPC |
| Sin estado global mutable ni persistencia expuesta | Cumple | snapshot/revisión y propietario history |
| Impacto transversal declarado | Cumple | secciones de lotes/paralelismo |
| ADR-002 aceptado antes del bootstrap | Cumple | aceptado el 2026-09-21; `docs/governance/ADRS.md` |

## Complejidad y nuevas dependencias

| Comprobación | Estado | Evidencia / responsable |
|---|---|---|
| Cada abstracción resuelve un problema concreto | Cumple | complejidad y R01–R09 |
| Coste, responsable, prueba y retirada documentados | Cumple | investigación/dependencias y pospuestos |
| Dependencias justifican necesidad y alternativa | Cumple | justificación de dependencias |
| Runtime/seguridad/persistencia usan ADR | Cumple | ADR-001–ADR-006 aceptados el 2026-09-21 |

## Contratos, datos y errores

| Comprobación | Estado | Evidencia / responsable |
|---|---|---|
| Entradas, salidas, unidades, nulos, límites y versiones | Cumple | modelo y contratos |
| Ambos lados validan cada frontera | Cumple | fixtures Zod/Serde y revalidación Rust |
| Estado, error, timeout, cancelación, retry e idempotencia | Cumple | contratos protocolo/IPC |
| Compatibilidad separada por contrato | Cumple | README y compatibilidad del resultado |
| Migración, backup, restauración y esquema futuro | Cumple | plan/modelo/quickstart |
| Históricos no se recalculan silenciosamente | Cumple | invariantes SessionResult |
| Evidencia G1–G4 cierra detalles provisionales | Pendiente | responsables L02/L03/L05/L08 |

## Seguridad y privacidad

| Comprobación | Estado | Evidencia / responsable |
|---|---|---|
| Actores/datos no confiables enumerados | Cumple | plan de seguridad |
| Privilegio mínimo y consentimiento local | Cumple | diseño helper/capabilities |
| Pruebas IPC/capability positivas y negativas | Cumple | pruebas de contrato IPC |
| Tamaños, tasas, profundidad, procesos y buffers acotados | Cumple | límites e invariantes |
| Secretos, logs, exportación y temporales revisados | Cumple | plan/modelo/contratos |
| Replay, duplicados, identidad, DoS e inyección | Cumple | variantes hostiles y fixtures |

## Pruebas y calidad

| Comprobación | Estado | Evidencia / responsable |
|---|---|---|
| Elige el nivel suficiente más barato | Cumple | estrategia de pruebas |
| Distingue fakes de garantías reales | Cumple | R11 y quickstart |
| Prueba límites, fallos y limpieza | Cumple | quickstart H1/H2/H3 |
| Incluye accesibilidad, idiomas, responsive y visual | Cumple | L01/L05 y quickstart |
| Incluye evidencia Windows/laboratorio | Cumple | prerrequisitos/escenarios |
| Distingue comandos existentes y futuros | Cumple | quickstart §§1 y 3 |
| Checkpoints observables y evidencia | Cumple | tabla de lotes/registro |
| Umbral Q2 ratificado | Cumple | constitución 0.7.0: 80 % por capa y métrica; 90 % líneas en críticos Rust; T132 lo configura |
| Compatibilidad G5 toolchain/instalador | Pendiente | L00; evidencia medida |

## Conflictos que exigen resolución explícita

| Conflicto | Estado | Acción necesaria |
|---|---|---|
| Windows 10 22H2 de spec frente a propuesta Q1 1809+ | Cumple | constitución 0.6.0 (2026-09-21): Windows 10 22H2 y Windows 11 x64; T002 prueba esa matriz |
| Accesibilidad completa H2 frente a propuesta H1 | Cumple | constitución 0.5.0 (2026-09-21): básica desde H1, completa en H2; coincide con FR-042 |
| 64 streams frente a propuesta de 32 simultáneos | Cumple | decisión del propietario 2026-09-21: FR-042 adopta 1–64 secuencial y 1–32 simultáneo por sentido; V-01 valida viabilidad |
| Manifiesto latest estable frente a URL versionada | Cumple | ADR-007 aceptado el 2026-09-22: `latest.json` estable con artefacto en URL inmutable por etiqueta (FR-042, T125); probado en `tests/updater.rs` |
| Feature-first frente a `Historias.md` §25 | Cumple | ADR-002 aceptado; §25 anota que rige `ARCHITECTURE.md` |
| Identificadores de log en redacción antigua | Cumple en plan | aplicar privacidad constitucional; reconciliar con autorización |

Ningún pendiente puede convertirse en `Cumple` por una decisión de implementación. El propietario
o el gate empírico indicado debe aportar la decisión o evidencia registrada.
