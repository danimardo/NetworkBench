# Contratos: NetworkBench v1

Estos contratos separan tres fronteras con ciclos de compatibilidad distintos:

| Contrato | Frontera | Autoridad | Versión |
|---|---|---|---|
| [ipc.md](./ipc.md) | Svelte/WebView ↔ Rust/Tauri | Rust autoriza; Zod y Serde validan | ligada a la app; cambios revisados |
| [peer-protocol.md](./peer-protocol.md) | peer NetworkBench ↔ peer | máquina de estados Rust | protocolo v1 después de G2 |
| [session-result.md](./session-result.md) | runtime ↔ historial/exportación/UI | modelo diagnóstico Rust | `schemaVersion: 1` después de G3/G4 |

## Reglas comunes

- Los datos externos entran como `unknown`/bytes y se validan antes de convertirlos al dominio.
- Los tipos por sí solos no validan datos en ejecución.
- Los campos desconocidos se rechazan o se ignoran solo donde lo autorice cada contrato.
- Ausente, inválido, cero y no evaluable son estados diferentes.
- Los enteros grandes cruzan JSON como cadenas decimales si JavaScript no los representa exactamente.
- El texto se normaliza y limita antes de persistir, mostrar, registrar o retransmitir.
- Los errores usan códigos/claves estables y campos estructurados; strings internos no son API.
- Cada cambio añade fixtures válidos e inválidos y actualiza productores y consumidores.
- Un cambio incompatible requiere nueva versión y decisión explícita de compatibilidad/migración.

## Política de gates

Los documentos dejan deliberadamente como gates los detalles dependientes de G1–G4 en vez de
inventarlos. Ninguna tarea puede retirar un gate eligiendo la implementación más cómoda: debe
adjuntar la evidencia exigida y actualizar contrato, pruebas y ADR cuando corresponda.
