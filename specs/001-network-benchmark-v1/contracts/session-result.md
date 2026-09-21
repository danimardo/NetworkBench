# Contrato `SessionResult`

**Esquema**: `schemaVersion: 1` provisional; se congela tras G3/G4  
**Consumidores**: UI de resultado, historial, comparación, exportación y reconciliación

## Forma raíz

```text
SessionResult {
  schemaVersion,
  sessionId,
  startedAt,
  finishedAt,
  status,
  initiator,
  responder,
  plan,
  capacity,
  directions,
  asymmetry,
  verdict,
  resultSource,
  versions            // incluye thresholdsHash (constitución I/IV)
}
```

Esta notación define semántica, no una implementación concreta. Toda unión anidada se discrimina
y analiza exhaustivamente. Una versión futura desconocida se rechaza para escritura.

## Campos raíz

| Campo | Contrato |
|---|---|
| `schemaVersion` | entero positivo; empieza en 1 solo tras cerrar G3/G4 |
| `sessionId` | UUID compartido; sensible en políticas de logs/exportación |
| `startedAt`, `finishedAt` | UTC ISO 8601; fin no anterior al inicio |
| `status` | `completed` o `incomplete`; fallo sin dirección completa no se guarda como resultado |
| `initiator`, `responder` | snapshots acotados y referencias de interfaz; sin material privado |
| `plan` | `BenchmarkPlan` aceptado exacto y versión de umbrales/reglas provisionales |
| `capacity` | capacidades de extremos, referencia opcional y origen |
| `directions` | uno/dos resultados pedidos; unicidad y completitud comprobadas |
| `asymmetry` | solo dos direcciones secuenciales completas; en otro caso no evaluable |
| `verdict` | interpretación estructurada/versionada; nunca texto libre remoto |
| `resultSource` | `initiator` o `local`; local implica reconciliación degradada |
| `versions` | versiones de app, motor, protocolo, reglas y esquemas independientes, más `thresholdsHash` (SHA-256 del `thresholds.json` embebido usado en el veredicto; constitución I/IV) |

## Snapshot del peer

Contiene etiqueta visible, id de instancia, endpoint/interfaz saneados y referencia abreviada de
identidad suficiente para relacionar historial. Se prohíbe certificado/material privado completo.
La exportación anónima elimina o reemplaza de forma determinista identificadores de equipo/red,
incluidos detalles crudos anidados y nombres sugeridos.

## DirectionResult

```text
DirectionResult {
  direction: forward | reverse | both,
  status: completed | incomplete | notStarted,
  sender: EngineResult?,
  receiver: EngineResult?,
  officialBps: decimal-integer-string?,
  utilization?,
  stability,
  retransmission,
  cpuSender,
  cpuReceiver,
  samples,
  gaps,
  consistency
}
```

- `officialBps` procede solo del receptor y está ausente si no es fiable/válido.
- Una dirección completa tiene ambos resultados válidos. Una incompleta nunca aporta throughput
  exitoso ni veredicto global de rendimiento.
- La utilización está ausente si se desconoce la capacidad de referencia.
- Las muestras identifican host/dirección; rangos ausentes usan huecos, nunca datos inventados.
- Datos crudos acotados, con origen, y excluidos/saneados según el modo de exportación.

## Estados de métricas

Cada métrica usa una de estas variantes:

```text
available { value, unit, source, quality }
notAvailable { reason }
notEvaluable { reason, availableEvidence? }
invalid { reason, diagnosticId? }
```

La serialización puede usar objetos discriminados en vez de null. Cero solo es válido donde lo
permita la definición. G4 aporta tablas de fronteras antes de implementar.

## Veredicto diagnóstico

```text
Verdict {
  rulesVersion,
  level: ok | warn | problem | notEvaluable,
  performance,
  stability,
  asymmetry,
  retransmission,
  cpu,
  consistency,
  facts[],
  observations[],
  possibleCauses[],
  actions[]
}

DiagnosticItem {
  ruleId,
  messageKey,
  safeParams,
  evidenceRefs[]
}
```

Los elementos son claves i18n con parámetros tipados, nunca strings renderizados. `possibleCauses`
usa lenguaje prudente y no afirma causa única. El orden estable procede de prioridad de reglas.

## Tamaño y precisión

- Tasas, contadores y timestamps grandes usan enteros exactos; JSON codifica como string decimal
  los enteros inseguros para JavaScript.
- XML/stdout/stderr tienen límites por campo y total. G3 decide si la evidencia cruda viaja aparte.
- Muestras con frecuencia, duración y total acotados. La trama soporta el máximo sin usar
  compresión como frontera de seguridad.
- JSON conserva unidades base/precisión; CSV convierte para presentación solo al exportar.

## Persistencia y reconciliación

1. El iniciador construye el resultado canónico tras direcciones/análisis.
2. El receptor valida esquema, vínculo sesión/plan/identidad y coherencia antes del ACK.
3. Cada lado persiste idempotentemente por `sessionId` dentro de una transacción.
4. El ACK identifica contenido validado; G3 elige campos exactos de digest/versión.
5. Si falla transferencia/ACK, la recuperación es explícita con `resultSource: local`; no se
   promete igualdad ni se sobrescribe evidencia distinta silenciosamente.
6. Lecturas históricas devuelven snapshot/versiones, nunca veredicto recalculado con umbrales nuevos.

## Compatibilidad

- Un consumidor antiguo puede ignorar campos opcionales añadidos si no cambia la semántica.
- Campos obligatorios nuevos, unidades/significado cambiados o variantes retiradas exigen versión.
- Una app antigua puede leer esquema soportado, pero no escribir persistencia más nueva.
- Exportadores identifican versión y declaran bloques omitidos no soportados.

## Pruebas de contrato

- fixtures canónicos completos/incompletos TCP y UDP;
- ausente/cero/máximo y frontera de precisión JavaScript;
- una/dos/simultáneas y duplicados inválidos;
- muestras insuficientes, capacidad desconocida y retransmisión ausente;
- cada categoría de veredicto con fronteras exactas de G4;
- máximo de datos crudos/muestras bajo la decisión G3;
- desacuerdo iniciador/receptor, persistencia repetida, ACK perdido y fallback local;
- renderizado es/en desde idénticas claves semánticas;
- canarios de anonimización en cada campo anidado/crudo;
- lectura/escritura de esquemas anteriores/posteriores.
