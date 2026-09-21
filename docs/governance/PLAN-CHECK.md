# Architecture Check para `plan.md`

Todo plan registra **cumple**, **pendiente** o **no aplica**, junto con evidencia y responsable.
«No aplica» exige explicación. El check se ejecuta antes de investigación y se repite después
del diseño y tras cualquier cambio material.

## Autoridad y alcance

- [ ] Identifica versión y estado de ratificación de la constitución.
- [ ] Enlaza requisitos de `Historias.md`, hito y criterios observables.
- [ ] Declara Q/G/V aplicables y condición de cierre.
- [ ] No usa una propuesta pendiente como autorización.
- [ ] Mantiene WHAT/WHY en `spec.md` y HOW en el plan.

## Límites y dependencias

- [ ] Asigna cada responsabilidad a una feature/módulo de `ARCHITECTURE.md`.
- [ ] Declara APIs públicas, consumidores y archivos de escritura.
- [ ] No introduce imports profundos, ciclos o aristas prohibidas.
- [ ] Mantiene autorización y negocio en Rust y presentación en Svelte.
- [ ] No exporta estado mutable ni detalles de persistencia.
- [ ] Explica el impacto en otras features y los recursos compartidos.

## Complejidad y dependencias nuevas

- [ ] Justifica cada capa, servicio, cola, cache, framework o abstracción transversal.
- [ ] Explica problema observado, alternativa local, coste, dueño, pruebas y retirada.
- [ ] Para dependencias: necesidad, alternativa existente, mantenimiento, licencia,
      seguridad, compatibilidad y tamaño/rendimiento.
- [ ] Usa ADR cuando la decisión sea transversal o difícil de revertir.

## Contratos, datos y errores

- [ ] Define entradas, salidas, unidades, nulos, límites, campos desconocidos y versiones.
- [ ] Define validación en ambos lados de cada frontera.
- [ ] Define estado, error, timeout, cancelación, reintento e idempotencia.
- [ ] Explica compatibilidad de IPC/protocolo/resultado cuando cambien.
- [ ] Incluye migración, backup, restauración y downgrade/rechazo cuando afecta datos.
- [ ] No recalcula silenciosamente resultados históricos.

## Seguridad y privacidad

- [ ] Enumera datos y actores no confiables.
- [ ] Mantiene privilegio mínimo y consentimiento local.
- [ ] Incluye pruebas de denegación de IPC/capabilities.
- [ ] Limita tamaños, frecuencias, profundidad, procesos y buffers.
- [ ] Revisa secretos, logging, exportación, anonimización y archivos temporales.
- [ ] Modela abuso relevante: replay, duplicados, identidad cambiada, DoS e inyección.

## Testing y calidad

- [ ] Selecciona el nivel más barato que pruebe cada riesgo.
- [ ] Distingue fakes/fixtures de ejecución real y qué garantía no cubren.
- [ ] Define tests de límites, errores y limpieza.
- [ ] Incluye accesibilidad, idiomas, responsive y visuales cuando afecta UI.
- [ ] Incluye Windows/laboratorio cuando la historia promete comportamiento nativo.
- [ ] Define comandos existentes o tareas explícitas para crearlos.
- [ ] Establece un checkpoint observable y evidencia de cierre.

## Resolución de conflictos

Una feature que no puede cumplir una regla no introduce una excepción silenciosa. El plan:

1. propone un diseño compatible;
2. si no basta, documenta alternativas e impacto mediante ADR;
3. tramita enmienda constitucional si afecta a un principio;
4. espera la decisión correspondiente antes de implementar la desviación.

