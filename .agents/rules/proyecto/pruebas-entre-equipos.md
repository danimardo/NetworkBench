# Pruebas entre equipos: consulta OpenObserve primero

**Ámbito:** este repositorio. **Estado:** VERIFICADO el 2026-09-30 (servicios listados y
`query_logs` ejecutado) salvo lo marcado como NO VERIFICADO. **Decisión del propietario:** 2026-09-30.

## Regla

Cuando el propietario cuente una prueba hecha contra otro ordenador, un fallo, o pida
analizar un comportamiento que involucre a **dos equipos** (descubrimiento, emparejamiento,
sesión de medición, firewall del otro extremo…):

1. **Antes de preguntarle nada ni proponer una causa**, consulta en OpenObserve la traza de
   **los dos equipos**: la de esta máquina y la del equipo de pruebas (hoy, una VM Windows 11
   llamada `WIN11D`).
2. Reconstruye la secuencia de lo ocurrido con esos registros — qué se descubrió, a qué
   dirección se conectó, dónde falló — y **cuéntale lo que has entendido** en lugar de pedirle
   que te detalle la prueba.
3. Solo pregunta lo que los registros no puedan responder, y dilo: qué buscaste, qué
   encontraste y qué falta.

Esto amplía la regla «Registros ante un problema» de `AGENTS.md`, que cubre los ficheros
locales. OpenObserve añade lo que los ficheros de esta máquina no tienen: la vista del **otro**
equipo.

## Cómo consultar

- Herramientas MCP `mcp__openobserve__*`. Empieza por `list_services`: hoy hay `netbench`,
  `pruebas` y `payments_api`. Los registros de la aplicación están en **`netbench`**;
  `payments_api` no es de este proyecto. `pruebas` estaba vacío el 2026-09-30.
- **`query_logs`** con `since`, `limit`, `level` y `max_pages` es lo que funcionó.
  **`search_logs` (texto libre) devolvió `BackendError` del gateway el 2026-09-30**: si te
  falla, no insistas; usa `query_logs` con ventana temporal ajustada y filtra tú el texto.
  Apunta el `Request-ID gateway` del error si lo informas.
- Ajusta la ventana a la prueba (`since`/`from`/`to`) y pide más páginas si hace falta. El
  nivel por defecto de producción es `info`: el detalle sensible (IP:puerto, huella, línea de
  comandos del motor) solo aparece con **Debug/Trace** activado en Ajustes → Diagnóstico
  (constitución 0.8.0, principio XIII). Si falta ese detalle, dilo y pide activarlo en ambos
  equipos y repetir la prueba.
- Los ficheros locales siguen siendo la vía inmediata para esta máquina:
  `%LOCALAPPDATA%\NetworkBench\logs\` (`networkbench.log`, `tracing.log`). El de la VM solo
  te llega por OpenObserve o si el propietario aporta la carpeta.

## Límites (no los presentes como garantía)

- El envío a OpenObserve es **opcional y está apagado por defecto**: si el equipo de pruebas
  no lo tiene activado, no habrá traza suya. Comprueba que hay eventos de ese equipo antes de
  concluir nada.
- NO VERIFICADO: cómo se distingue en el servicio `netbench` un equipo de otro (campo de
  host o de identidad). Averígualo en el primer uso y actualiza esta regla.
- La ausencia de registros no prueba que algo no ocurriera: puede ser nivel de log, envío
  apagado, hora desajustada entre máquinas o un fallo del gateway.
- Los registros pueden contener datos de la red del propietario: no los copies a informes,
  commits ni artefactos más allá de lo necesario.
