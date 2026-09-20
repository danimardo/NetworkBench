# NetworkBench — auditoría de la entrega de diseño v3

Fecha: 20 de septiembre de 2026.

## Dictamen

La entrega mejora sustancialmente la anterior, pero **no es suficiente para implementar la v1 completa sin tomar decisiones de producto o diseño pendientes**. Tampoco puede darse por cerrado el diseño de H1.

El propio README identifica la entrega como una ampliación de H1 y deja H2/H3 para después. Entregar por hitos es compatible con el encargo; dejar esas funciones para después no equivale a haberlas diseñado. Además, se han excluido de esta ronda requisitos que la especificación sitúa expresamente en H1, como español/inglés y adaptación de ventana.

Se puede empezar a desarrollar infraestructura y componentes con esta base. No conviene considerar aprobado el recorrido completo ni trasladar literalmente la maqueta al producto.

No se necesitan más documentos por volumen: se necesitan decisiones coherentes, composiciones pendientes y reglas verificables. No hace falta dibujar cada combinación de estados, siempre que una regla compartida permita resolverla sin inventar comportamiento.

## Alcance y comprobaciones

Se han revisado `Especificacion.md`, el inventario de `Design`, README, FLUJOS, PANTALLAS-H1, DESIGN_TOKENS, el código HTML/CSS/JS de la maqueta y los componentes y tokens relevantes.

- `node Design/scripts/verify-tokens.mjs`: pasa. Comprueba determinados literales de color; no certifica la tokenización de dimensiones, tipografía, animación o todos los formatos CSS de color.
- El bloque JavaScript ejecutable de la maqueta pasa una comprobación de sintaxis con Node. No equivale a probar el recorrido.
- Las referencias `var(--...)` de los componentes examinados tienen declaración en el fichero de tokens.
- Se han recalculado los cinco pares de contraste corregidos y los extremos claros del botón primario.
- Se ha comparado la presencia y valores de tokens entre CSS y maqueta; no son copias completamente equivalentes.
- No hay `package.json` ni configuración de compilación del paquete entregado: no se ha ejecutado `svelte-check` ni una aplicación Tauri.
- Se intentó conectar el navegador disponible mediante su herramienta. No hay ningún navegador conectado. **No se ha realizado validación visual renderizada, de interacción o de accesibilidad con lector de pantalla.** Los hallazgos de código son estáticos y los cálculos de contraste son de pares de colores.

No se han modificado la especificación ni los archivos del diseñador.

## Mejoras comprobadas

- Mapa de estados y caminos alternativos en `FLUJOS.md`.
- Documentación adicional de descubrimiento y conexión manual.
- Maqueta con selector/resumen, conexión manual, preparación, ejecución, resultados y simulaciones de errores.
- Componentes nuevos: diálogo, tooltip, notificación interna, campo de texto, código de verificación, elemento de checklist y progreso.
- Separación de disponibilidad, compatibilidad, confianza, favorito y selección en `DeviceCard`.
- Iconos adicionales y tipos de adaptador Ethernet/Wi-Fi/otro.
- Botones de ventana de 48 px y retirada de la separación inferior.
- Nuevos tokens de gráficas, formularios, overlay y superficies opacas.
- Corrección efectiva de cinco pares de colores del tema claro.

## A. Huecos y contradicciones que impiden cerrar H1

### A01. Falta la composición real de aceptación en el receptor

**Evidencia:** `Design/README.md:105`, `Design/maqueta-navegable.html:1065`, `Design/FLUJOS.md`, frente a §9.1 y §10.4.

La maqueta sustituye la vista del receptor por una nota que cuenta qué vería. No existe una composición que muestre juntos solicitante, IP, adaptador, tipo, duración, advertencia de tráfico, código, confianza y acciones.

En el iniciador aparecen «No coincide» y «Coincide, continuar», seguidos de otra aceptación. La especificación pide que el iniciador espere y que el receptor confirme código y prueba en un único diálogo la primera vez.

**Pedir al diseñador:** vistas explícitas de iniciador y receptor, para peer desconocido, conocido e identidad cambiada; diálogo combinado; espera, rechazo, caducidad y cancelación. Definir qué significa Escape/clic exterior y qué ocurre si el peer cancela mientras el diálogo está abierto. Incluir la casilla de confianza, desmarcada por defecto.

**Cierre:** poder seguir ambos lados del mismo caso sin confirmaciones adicionales inventadas ni notas que sustituyan una composición distinta.

### A02. El selector todavía no resuelve cómo elegir equipo desde la acción principal

**Evidencia:** `Design/maqueta-navegable.html:708` y `:884`, frente a §16.2–16.3.

«Analizar conexión» conduce al resumen de SERVER-01 mediante un destino fijo. Las tarjetas de Inicio permiten elegir ejemplos, pero falta concretar qué abre la acción principal cuando hay cero, uno o varios candidatos y ninguno seleccionado.

También falta ubicar el adaptador local seleccionado automáticamente antes de enviar la solicitud (§10.2).

**Pedir al diseñador:** selector con elección efectiva, selección vacía, candidato elegido y acceso a conexión manual; regla de preselección si se desea alguna; resumen de adaptador local y remoto y regreso sin perder datos.

No es un problema que la demo utilice nombres ficticios. El hueco es que no existe una regla de producto para el caso sin equipo preseleccionado.

### A03. Inicio no cubre todavía todos sus estados

**Evidencia:** `Design/PANTALLAS-H1.md:20` y §7.1, §7.3, §16.2.

Se documentan buscando, vacío, descubrimiento desactivado y algunos cambios de lista. Quedan sin una resolución completa:

- Primera ejecución sin última prueba ni favoritos/recientes.
- Sin ningún adaptador conectado y acción «Abrir configuración de red».
- Muchos equipos descubiertos, no solo muchos favoritos.
- Consulta del nombre completo tras truncarlo y presentación de IPv6/alias largos.
- Edición de alias y gestión de favoritos/recientes más allá del icono de estrella.
- Regla temporal para pasar de «buscando» a «ninguno encontrado» sin detener el descubrimiento continuo.
- Cómo se accede a información/actualización de un equipo incompatible si su tarjeta no es seleccionable.

Hay dos discrepancias adicionales: el enlace para activar descubrimiento apunta a Ajustes → General, pero §23 lo sitúa en Red; la acción rápida para el último equipo se conserva aunque no esté disponible, mientras §16.2 condiciona su aparición. Hay que resolverlas explícitamente.

**Pedir al diseñador:** tabla de estos estados con composición, acciones y destino. No hace falta una pantalla nueva para cada uno.

### A04. La conexión manual contradice los formatos admitidos

**Evidencia:** `Design/maqueta-navegable.html:934`, `Design/PANTALLAS-H1.md:32`, §7.2 y §26/H1.

La documentación reconoce DNS, IPv6 y puerto, pero la maqueta dice «Dirección IP», usa un ejemplo exclusivo de IPv4 y exige «una IPv4 de la red local», pese a presentar esta pantalla como conexión fuera de la red.

**Pedir al diseñador:** etiquetas, ayuda, ejemplos y mensajes finales coherentes con «Dirección del equipo»; resolución DNS, conectando, timeout, corrección y dirección usada. Separar sintaxis inválida de nombre válido que no resuelve. Documentar si se conserva lo introducido al reintentar.

No hace falta que el diseñador programe un parser de direcciones. Sí hace falta cerrar el contenido y los estados. No se deben posponer los errores de conexión exigidos por H1 alegando que el catálogo completo llega en H2.

### A05. Preparación y ejecución no corresponden todavía a la composición requerida

**Evidencia:** `Design/maqueta-navegable.html:952` y `:987`, §10.2, §10.5 y §16.4.

La checklist maquetada contiene cuatro pasos y mezcla verificaciones locales con conjuntas. No representa toda la lista exigida ni documenta una agrupación aprobada.

En ejecución faltan las dos tarjetas de equipos, modelos de NIC, velocidades de enlace y la conexión con flecha de dirección que §16.4 define como elemento principal. Hay velocidad, progreso y gráfica, pero es una composición diferente.

**Pedir al diseñador:** checklist completa o agrupación explícita con correspondencia a todos sus pasos; estados de acción requerida; composición de equipos y conexión; calentamiento, enfriamiento, reconexión y análisis, con acciones siempre accesibles. Definir el cambio de perspectiva local/remoto sin invertir el significado de A→B/B→A.

Las gráficas completas pueden entregarse con H2. La identificación de extremos y dirección pertenece al recorrido H1.

### A06. El resultado básico necesita corregirse antes de ampliarlo

**Evidencia:** `Design/maqueta-navegable.html:1026`, `:1042` y `:1442`, §13, §16.5 y §18.

- No hay cifra principal de sesión con la jerarquía requerida; aparecen dos cifras por dirección y un veredicto.
- Se muestra «Pérdida de paquetes» en una prueba estándar TCP. La especificación reserva ese resultado principal a UDP y define retransmisiones para TCP.
- Falta una variante propia de resultado incompleto, diferenciada de cancelado.
- No quedan resueltos resultado parcial, dirección no ejecutada, dato desconocido, capacidad no determinable y resultado calculado localmente.
- Los ejemplos numéricos no están vinculados a las reglas del veredicto. Por ejemplo, debe saberse qué capacidad se utiliza para calificar un resultado de unos 900 Mbit/s cuando se presentan adaptadores de 10 Gbit/s.

**Pedir al diseñador:** resultado H1 fiel a sus datos, y composición H2 con cifra principal, capacidad, métricas y explicación progresiva. Entregar ejemplos coherentes con §13, sin deducir veredictos a partir del aspecto visual. Definir dónde se ofrecen datos parciales y cómo se vuelve a consultarlos.

### A07. El bloqueo de navegación empieza demasiado tarde y no protege todas las salidas

**Evidencia:** `Design/FLUJOS.md:104`, `Design/README.md:300` y `Design/maqueta-navegable.html:1162`, §16.1.

La nueva documentación limita el bloqueo a PREPARING en adelante. Debe abarcar la sesión activa desde CONNECTING, incluidas confirmación y espera, hasta su terminación.

Además, Inicio sigue siendo un botón activo. En la maqueta la navegación puede sustituir la vista de sesión; no está documentado si pulsar Inicio durante la prueba debe conservar la pantalla, ser inerte o mostrar otra cosa. No puede permitir abandonar inadvertidamente el recorrido.

**Pedir al diseñador:** tabla única de navegación permitida por estado, tratamiento de Inicio, vuelta atrás, Escape y cancelación. Su implementación corresponde a desarrollo.

### A08. Responsive e idiomas son pendientes de H1, no exclusiones aceptadas

**Evidencia:** `Design/README.md:118`, §4, §16.9 y §26/H1.

El README afirma que responsive/i18n quedaban fuera de H1 en el correo. La especificación los exige en H1 y el encargo pidió estas variantes para cada bloque.

**Pedir al diseñador:** reglas y ejemplos de las pantallas H1 en 800×600, 1100×760 y ancho mayor de 1400; sidebar solo iconos por debajo de 1000; apilamiento y scroll; textos en español e inglés. Incorporar el diseño para 115/130 % de texto, cuya funcionalidad corresponde a H2, para evitar rehacer composiciones.

No es necesario entregar una imagen de cada combinación. Sí una matriz de pruebas y reglas completas de adaptación, incluyendo diálogos altos y nombres largos.

### A09. Los avisos simulados no cierran el catálogo necesario

**Evidencia:** `Design/maqueta-navegable.html:1277`, `Design/FLUJOS.md`, §21.2 y §26/H1.

Hay avisos humanos, lo cual es una mejora. Faltan casos y algunas acciones no coinciden con la especificación: versión incompatible dice actualizar «uno de los dos» sin identificarlo; código no coincidente y rechazo omiten acciones previstas; la simulación de fallo de puertos de prueba usa NB-FW-001, que corresponde al canal de control, en lugar de distinguir el diagnóstico.

La preparación y algún error muestran `ntttcp.exe`, pese a la regla global de ocultar el nombre del motor fuera de Detalles técnicos y Acerca de.

**Pedir al diseñador y responsable de textos:** patrón de aviso y tabla final para todos los códigos de H1 con título, explicación, causas prudentes, acciones/destinos y texto es/en. En H2, completar el resto. Las notas técnicas de demo deben estar separadas inequívocamente del texto final de producto.

## B. Entregables que siguen faltando para la v1 completa

La tabla no solicita nuevas funcionalidades: recoge el alcance de la especificación que la entrega declara pendiente o no resuelve.

| ID | Entregable | Contenido mínimo de cierre |
|---|---|---|
| B01 | Resultado H2 completo | Capacidad y origen, estabilidad, asimetría, CPU por equipo, retransmisiones, Hechos/Observaciones/Posibles causas/Acciones, métricas ausentes y variantes de veredicto. |
| B02 | Gráficas H2 | Ejes/unidades/ticks, leyenda, calentamiento/enfriamiento, ambas series y patrones, gaps sin interpolar, tooltip por teclado, sin datos y escala. Incluir la advertencia de §12.2 sobre tráfico total del adaptador frente a velocidad oficial. |
| B03 | Detalles técnicos H2 | Dirección/equipo, secciones, unidades, comandos/XML copiables, feedback de copia, datos largos; panel lateral y alternativa compacta opaca. |
| B04 | Firewall H2 | Diagnóstico, consentimiento, elevación, UAC denegado, directiva corporativa, reglas desactualizadas y acciones/instrucciones por escenario. |
| B05 | Historial y evolución H3 | Filtros, búsqueda, selección múltiple, vacíos, sesiones parciales, reapertura, repetición, borrado, comparaciones y series históricas. |
| B06 | Opciones avanzadas y UDP H3 | Todos los campos de §17, ayudas, validación, valores recomendados, persistencia del plan, direcciones únicas/simultáneas y resultado de pérdida UDP. |
| B07 | Ajustes | Todas las secciones de §23, estados y aplicación de cambios; General/Firewall parciales H2 y resto H3. Incluir gestión de confianza y aceptación automática de §9. |
| B08 | Exportación H3 | Diálogo PDF/JSON/CSV, idioma, privacidad, muestras CSV, exportación múltiple admitida, errores y éxito; plantilla A4 multipágina con tablas y gráficas de impresión. |
| B09 | Cierre, bandeja y actualizaciones H3 | Cierre normal/activo, recordar decisión, notificaciones nativas, estados de bandeja, actualización disponible/descargando/error/instalación y sesión activa. |
| B10 | Movimiento | Secuencias y tiempos de transiciones, aparición de equipos, cambio de dirección y resultado; interrupción inmediata, reducción de movimiento y restricciones RUNNING. |
| B11 | Biblioteca restante | Casillas, radios, conmutadores, selector, campos numéricos, botón destructivo/con carga, menús, panel lateral, tablas, filtros, desplegables, scrollbars y esqueletos. Diseños con estados bastan; no se exige todo el código Svelte. |
| B12 | Recursos de producto | Icono definitivo de app y bandeja, originales vectoriales y exportaciones/tamaños, variantes necesarias y licencias. Los SVG de interfaz embebidos en `icons.ts` sí existen; no se han encontrado archivos gráficos independientes ni originales externos de las propuestas. |

El diseñador puede entregar por bloques, pero deben quedar responsables y entregables comprometidos para todos ellos. Llamar al icono «activo de producto fuera del sistema» no elimina la necesidad de entregarlo para la aplicación.

## C. Correcciones transversales de la entrega

### C01. Respaldo opaco declarado como completo, pero parcial en código

El README afirma que los tres materiales tienen fallback completo. Hay tokens y ejemplos, y `Dialog`, `Toast` y `Tooltip` sí implementan `@supports not (...)`. No lo hacen `Card`, `Sidebar`, `Button` y `StatusPill`. Tampoco existen clases CSS compartidas que lo aporten automáticamente.

**Solicitud:** completar las superficies y comprobar la variante sin blur. Los tokens definen el aspecto; su aplicación al código puede asumirla desarrollo. Corregir la declaración de estado de la entrega.

### C02. Contraste mejorado, pero la auditoría no está cerrada

Los nuevos valores sobre `#F8F7FB` dan: muted 5,29:1, success 5,34:1, warning 5,35:1, danger 4,90:1 e info 5,38:1. Las correcciones son reales.

El botón primario claro conserva blanco sobre un gradiente cuyo extremo `#9D72D3` da 3,63:1, y el extremo de hover `#A97FDC` da 3,11:1. Esto identifica zonas de riesgo; no demuestra por sí solo qué contraste tienen todos los glifos en cada tamaño de botón.

**Solicitud:** revisar el botón en sus tamaños/estados y medir el fondo efectivo bajo el texto; entregar matriz de contraste para superficies reales, transparencias, foco, gráficos y estados. No declarar AA global basándose solo en el fondo principal.

### C03. Tokenización y paridad todavía parciales

Persisten tamaños como 11,5 px, medidas de diálogos, duraciones y `saturate(125%)` en componentes. El verificador no cubre esos valores. La maqueta conserva colores literales en superficies de resultado y difiere, por ejemplo, en `--color-chart-warmup-zone`.

No toda ausencia de token en la maqueta implica un defecto visual: algunos no se utilizan. Sí contradice una equivalencia total de las dos fuentes.

**Solicitud:** fuente de verdad inequívoca, tokens compartidos o sincronización comprobable, y completar §16.10 o documentar excepciones aprobadas. Desarrollo puede ampliar el verificador.

### C04. Accesibilidad de componentes pendiente de integración y corrección

Hallazgos estáticos relevantes:

- `Dialog.svelte` mueve el foco inicial, pero no atrapa Tab ni hace inerte el fondo. Delega el retorno del foco y no hay una integración entregada que lo complete.
- `Tooltip.svelte` no enlaza el texto al disparador mediante `aria-describedby`. Sus disparadores deshabilitados o no enfocables no reciben foco por teclado; no hay cierre con Escape ni regla de colisión con bordes.
- `Card.svelte` procesa Enter/Espacio de forma que también puede recibir el evento propagado desde el botón interno de favorito; no filtra el origen. Debe separarse la acción de tarjeta de la acción de favorito.
- `ProgressBar.svelte` expone `role="progressbar"` sin un nombre accesible ni prop para suministrarlo.
- La confianza conocida y de confianza usa el mismo escudo con diferencias visuales y tooltip sobre un span no enfocable. Hay que garantizar identificación y acceso sin depender del color/ratón.
- La barra determinada anima `width`, mientras §16.10 restringe las animaciones de RUNNING a transform/opacity.

**Solicitud al diseñador:** reglas de foco, interacción, estados y alternativa accesible. **Trabajo de desarrollo:** semántica HTML/ARIA, manejo de eventos, focus trap, optimización y pruebas. No hace falta encargar al diseñador una librería de accesibilidad programada.

### C05. Documentación que aún se contradice

- README sigue describiendo oscuro «por defecto» en su introducción, aunque luego corrige a Sistema.
- Una fila presenta `Toast.svelte` como aviso para app minimizada; el componente y su API lo distinguen correctamente del toast nativo.
- Opciones avanzadas y exportación se etiquetan en partes como H2, mientras la especificación las coloca en H3.
- DESIGN_TOKENS conserva referencias a `CodeInput.svelte`, que no existe; el componente entregado es `VerificationCode.svelte`.
- La documentación de accesibilidad de tokens describe la antigua card con botón nativo, mientras el código usa `div role="button"`.
- `adapterType` sigue teniendo Wi-Fi como valor por defecto pese a documentar que nunca debe asumirse Wi-Fi. Debe exigirse el dato o definir el estado desconocido.

**Solicitud:** limpiar referencias obsoletas y mantener un único inventario de estado. «Hecho» debe significar que documentación y recurso implementan la misma decisión.

## D. Decisiones que no puede resolver el diseñador por su cuenta

### D01. Secuencia de emparejamiento y PRECHECK

La especificación sitúa PAIRING antes de PRECHECK en §10.1, exige PRECHECK antes de molestar al peer en §10.2 y combina emparejamiento con aceptación en §9.1.5. El nuevo flujo reproduce las tres indicaciones sin resolver su orden operativo.

**Responsable:** producto y desarrollo, con el diseñador reflejando la decisión. Se necesita una secuencia canónica de mensajes, comprobaciones y diálogos para peer nuevo y conocido, aclarando cuándo se conocen y muestran los datos de la prueba. No inventar una segunda aceptación para salvar la contradicción.

### D02. Divergencia de la barra de título

El diseñador deja expresamente pendiente la aprobación del degradado frente al plano integrado requerido por §16.13. La documentación también contiene referencias a propuestas aprobadas no adjuntas.

**Responsable:** propietario del producto. Elegir el diseño y sincronizar especificación y material. Mientras no se apruebe un cambio, la especificación vigente sigue siendo la referencia normativa; no se presume aprobación por recibir archivos.

### D03. Requisitos incorporados por referencia a Historias.md

`Historias.md` no está en el directorio revisado y el diseñador dice no haberlo recibido. La especificación incorpora secciones «íntegramente» y conserva textos «tal cual», por ejemplo en §13.8, §16.11, §16.12 y el recorrido global de §26.

**Responsable:** propietario del producto. Facilitar el documento o integrar en Especificacion.md el contenido que siga siendo obligatorio. No se puede certificar cobertura de contenido no entregado. No corresponde al diseñador rellenarlo por intuición.

### D04. Alcance e hitos

**Responsable:** producto y desarrollo. Mantener H1/H2/H3 de la especificación, o aprobar una modificación expresa. La entrega puede adelantar un diseño de H3; eso no autoriza a retrasar requisitos de H1 ni a eliminar entregables del encargo.

## Qué solicitar ahora

1. **Corrección y cierre de H1:** A01–A09, incluyendo una vista real del receptor y los requisitos responsive/es/en. Prioridad máxima a aceptación, selector, ejecución y resultado TCP.
2. **Paquete H2/H3 completo:** B01–B12, con fechas o bloques de entrega y sin confundir «referenciado» con «diseñado».
3. **Correcciones de sistema:** C01–C05, con estados de contraste, materiales y accesibilidad descritos con precisión.
4. **Anexo de decisiones:** D01–D04, resueltas por el responsable correspondiente y reflejadas en una única versión de los documentos.

No pedir imágenes decorativas arbitrarias, una nueva identidad visual ni todos los componentes programados. La integración de Tailwind, persistencia, Tauri, rendimiento del motor, gestión de estado y pruebas técnicas corresponde a desarrollo.

## Criterio de aceptación de la siguiente entrega

Para cada pantalla o estado, la matriz final debe contener:

| Campo | Qué debe permitir comprobar |
|---|---|
| ID y requisito | Qué punto de la especificación cubre y en qué hito. |
| Rol y estado | Qué ve cada extremo y qué condición lo activa. |
| Referencia visual | Pantalla/componente concreto o variante basada en reglas completas. |
| Datos | Campos, unidades, ejemplos coherentes, ausencias y contenido largo. |
| Acciones | Qué hace cada control, destino, espera, reintento y cancelación. |
| Interacción | Teclado, foco, Escape, clic exterior y cambios remotos. |
| Adaptación | Tema, ancho, texto ampliado e idioma. |
| Recursos | Tokens/iconos y originales o exportaciones necesarios. |
| Estado de cierre | Resuelto, pendiente o cambio de requisito aprobado, con responsable. |

La revisión estará cerrada cuando no queden decisiones de producto/diseño necesarias en «pendiente», «probablemente» o «lo decidirá el programador», y se verifique el recorrido renderizado. Siempre habrá decisiones técnicas normales de implementación; no es necesario que el diseñador elija algoritmos o arquitectura para eliminar huecos de diseño.
