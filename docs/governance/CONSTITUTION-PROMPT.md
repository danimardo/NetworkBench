# Texto recomendado para el flujo oficial de constitución

**Estado:** propuesta. Este fichero no es la constitución y no ratifica sus principios.

Copiar exactamente el bloque siguiente como entrada de `/speckit.constitution` o la
invocación equivalente de la integración instalada:

```text
Prepara una evolución de la constitución de NetworkBench mediante el flujo
oficial de SpecKit, partiendo de la versión 0.4.0 existente.

No implementes funcionalidades, no ejecutes specify, no crees ramas y no
modifiques plantillas, skills, requisitos, diseño ni documentos históricos.

El objetivo es una constitución breve formada por principios duraderos.
Versiones concretas, rutas de implementación, comandos, porcentajes y matrices
de plataforma deben pertenecer a documentación operativa subordinada.

Antes de retirar cualquier detalle vigente, presenta una tabla de trazabilidad
que indique obligación actual, destino operativo, autoridad y efecto del cambio.
No elimines obligaciones ni traslades contenido protegido sin autorización.
Si los destinos todavía no existen o no están aprobados, deja la sustitución
pendiente y presenta el diff propuesto.

Mantén Q1–Q4 pendientes hasta decisión explícita del propietario. Mantén G1–G6
y las verificaciones empíricas abiertas hasta contar con evidencia. No registres
una fecha de ratificación ni declares compatibilidad por inferencia.

Utiliza los siguientes principios como núcleo propuesto:

I. Precisión y honestidad
El sistema MUST distinguir mediciones, interpretaciones y posibles causas.
MUST conservar procedencia suficiente para explicar resultados históricos.
MUST NOT convertir ausencia o insuficiencia de datos en cero, éxito o certeza.
Los efectos visuales MUST NOT comprometer precisión, seguridad o cancelación.

II. Modularidad y autoridad
Cada módulo MUST tener responsabilidad y API pública explícitas.
Las dependencias MUST seguir límites documentados y permanecer acíclicas.
Los consumidores MUST NOT depender de detalles privados.
La presentación MUST NOT decidir autorización ni duplicar reglas de negocio.
Las reglas de negocio MUST poder probarse sin interfaz gráfica ni I/O real.

III. Simplicidad justificada
Toda nueva capa, framework, servicio, cola, cache o abstracción transversal
MUST justificar un problema concreto y explicar por qué no basta una solución
local más sencilla.
El proyecto SHOULD preferir convenciones explícitas y composición directa.
MUST NOT introducir infraestructura para posibilidades puramente hipotéticas.
MAY extraer abstracciones cuando exista reutilización demostrada o una frontera
de confianza que lo requiera.

IV. Contratos y validación
Toda frontera de confianza MUST validar datos antes de producir efectos.
El tipado estático MUST NOT sustituir validación en ejecución.
Entradas y salidas MUST tener contratos explícitos sobre tipos, unidades,
límites, ausencias y compatibilidad.
Implementaciones de un contrato en distintos lenguajes MUST comprobar su
equivalencia. Los datos inválidos MUST NOT autorizar operaciones.

V. Seguridad, consentimiento y privacidad
Las operaciones MUST aplicar privilegio mínimo y autorización explícita.
Descubrimiento, nombres y direcciones MUST NOT considerarse autenticación.
Los secretos MUST quedar fuera del cliente, logs y exportaciones.
El producto MUST preservar su funcionamiento local y MUST NOT introducir
telemetría, cuentas o servicios externos sin una decisión de alcance explícita.
La aplicación MUST limitar recursos consumidos por entradas no confiables.

VI. Integridad y evolución de datos
Las escrituras relacionadas MUST preservar consistencia e idempotencia cuando
puedan repetirse.
Los cambios de esquema MUST incluir compatibilidad, recuperación y pruebas.
MUST NOT destruir ni reinicializar datos silenciosamente.
Una versión MUST NOT escribir formatos futuros que no comprende.
Los resultados históricos MUST NOT reinterpretarse silenciosamente.

VII. Calidad demostrable
Cada cambio de comportamiento MUST tener pruebas proporcionadas a su riesgo.
Las reglas mecánicamente comprobables MUST disponer de controles automáticos
antes de cerrar la implementación afectada.
Los controles obligatorios MUST bloquear el cierre cuando fallen o no puedan
ejecutarse.
La cobertura MUST NOT sustituir aceptación, seguridad ni evidencia empírica.
Los defectos corregidos SHOULD incorporar una regresión automatizada.
Las pruebas MUST NOT limitarse a reproducir la implementación.

VIII. Interfaz y accesibilidad
Cada entrega de interfaz MUST cumplir los requisitos de accesibilidad,
internacionalización y diseño aplicables a su alcance.
Los errores MUST comunicar qué ocurrió y cómo actuar.
La información esencial MUST NOT depender exclusivamente de color, movimiento
o interacción con ratón.
La cancelación y las acciones críticas MUST permanecer disponibles.

IX. Errores y observabilidad
Los fallos MUST representarse de forma estructurada y conservar contexto seguro.
MUST NOT ocultarse fallos convirtiéndolos en resultados satisfactorios.
Los diagnósticos MUST ser útiles, locales, acotados y libres de secretos.
El fallo del sistema de logging MUST NOT impedir cancelación ni comprometer
la función principal.
La configuración MUST ser explícita, validada y separada de secretos.

X. Dependencias y compatibilidad
Las dependencias nuevas MUST justificar necesidad, alternativas, mantenimiento,
licencia, seguridad e impacto.
El proyecto SHOULD reutilizar soluciones ya adoptadas.
Las publicaciones MUST identificar sus dependencias y artefactos de forma
trazable y comprobar la compatibilidad declarada.
MUST NOT ampliar APIs, privilegios o plataformas silenciosamente.
Los cambios incompatibles MUST incluir una estrategia explícita de transición.

XI. Trazabilidad y responsabilidad de agentes
Humanos y agentes MUST seguir los mismos límites y criterios de calidad.
Cada lote MUST vincular requisitos, implementación y evidencia de cierre.
El trabajo paralelo MUST declarar ownership y coordinar recursos compartidos.
Los agentes MUST NOT modificar fuentes protegidas, ampliar permisos o realizar
operaciones Git fuera de la autorización recibida.
Una propuesta, un test simulado o un comando no ejecutado MUST NOT presentarse
como aprobación o verificación real.

XII. Evolución consciente
Una feature MUST respetar la constitución y la arquitectura adoptada.
Un conflicto MUST resolverse mediante rediseño o decisión arquitectónica
explícita; MUST NOT introducirse como excepción silenciosa.
Las decisiones difíciles de revertir MUST registrar contexto, alternativas,
consecuencias y estado.
Toda enmienda MUST identificar motivo, impacto, compatibilidad, evidencia
y artefactos afectados, conservando la historia de las decisiones.

Gobernanza:
Conserva al propietario actual y el procedimiento de autorización.
Propón la versión adecuada según el alcance real del cambio y explica el motivo.
La primera ratificación requiere una decisión explícita del propietario.
No rebajes obligaciones existentes bajo la apariencia de simplificación.
La aceptación documental no equivale a verificar la implementación.

Antes de escribir la constitución, presenta el diff y su impacto y espera la
autorización específica exigida por AGENTS.md.
```

