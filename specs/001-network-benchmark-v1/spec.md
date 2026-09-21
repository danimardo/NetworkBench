# Feature Specification: NetworkBench v1

**Feature Branch**: `001-network-benchmark-v1` *(identificador de feature; no se creó rama Git)*

**Created**: 2026-09-21

**Status**: Tasks generated; analizada con `$speckit-analyze` el 2026-09-21

**Constitution**: 0.6.0, no ratificada

**Input**: Contenido completo de `Historias.md` como descripción de la feature.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Medir una conexión entre dos equipos (Priority: P1)

Como responsable de una red, quiero seleccionar otro equipo, autorizar una prueba estándar y
medir la conexión en ambos sentidos para conocer la velocidad útil que consigue.

**Why this priority**: Es el recorrido que aporta el valor esencial del producto.

**Independent Test**: Con dos instalaciones nuevas en equipos conectados, se completa el
descubrimiento o conexión manual, la verificación mutua, la aceptación y una prueba estándar
en ambos sentidos. Ambos equipos terminan con el mismo resultado básico identificado.

**Acceptance Scenarios**:

1. **Given** dos equipos disponibles, **When** el usuario elige el peer, verifica el código y
   el receptor acepta, **Then** la prueba mide secuencialmente los dos sentidos y ambos equipos
   reciben el resultado de la misma sesión.
2. **Given** un peer en otra subred accesible, **When** el usuario introduce su dirección válida,
   **Then** sigue el mismo recorrido sin depender del descubrimiento.
3. **Given** una sesión activa, **When** llega otra solicitud o se intenta iniciar otra,
   **Then** la segunda no comienza y se explica que el equipo está ocupado.
4. **Given** una prueba activa, **When** cualquiera la cancela, **Then** ambos extremos la
   detienen, liberan sus recursos y muestran el origen sin presentar un parcial como éxito.

---

### User Story 2 - Entender el resultado y qué hacer (Priority: P1)

Como técnico, quiero ver la velocidad principal, su contexto, estabilidad y posibles problemas
explicados en lenguaje humano para decidir si la conexión funciona como debería y qué revisar.

**Why this priority**: El producto se define por explicar la medición, no por exponer la salida
del motor.

**Independent Test**: Con resultados predeterminados correctos, degradados, incompletos y no
evaluables, la pantalla distingue hechos, observaciones, posibles causas y acciones; nunca
afirma una causa que los datos no demuestran.

**Acceptance Scenarios**:

1. **Given** una prueba completada con capacidad fiable, **When** se muestra el resultado,
   **Then** la velocidad es la cifra principal y se contextualiza en cada dirección.
2. **Given** datos insuficientes o capacidad no fiable, **When** se interpreta la sesión,
   **Then** el indicador dice «no evaluable» y no se transforma en cero o éxito.
3. **Given** resultados asimétricos, variables, incoherentes o con retransmisiones, **When** se
   consulta la conclusión, **Then** se muestran hechos, importancia, posibles causas y acciones concretas
   tomadas exclusivamente de las tablas de casos G4 (`thresholds.json` y reglas versionadas),
   sin causalidad inventada ni texto libre.
4. **Given** que el usuario abre detalles, **When** consulta una dirección, **Then** ve las
   mediciones de ambos extremos, parámetros y evidencia técnica asociada.

---

### User Story 3 - Resolver bloqueos y fallos (Priority: P2)

Como técnico, quiero que la preparación detecte problemas de red, puertos, permisos, motor o
adaptador y me ofrezca una acción segura para completar la prueba o entender por qué no.

**Why this priority**: Los fallos de entorno son habituales y deben evitar conclusiones falsas.

**Independent Test**: Se provocan errores representativos antes, durante y después de una
medición. Cada uno impide efectos inválidos, conserva datos útiles y muestra explicación,
referencia y acciones adecuadas.

**Acceptance Scenarios**:

1. **Given** que el control funciona pero el tráfico de medición está bloqueado, **When** falla
   la preparación, **Then** se identifica el extremo y se ofrecen configuración autorizada o
   instrucciones manuales.
2. **Given** que el usuario rechaza una elevación, **When** vuelve a la aplicación, **Then** esta
   sigue sin privilegios elevados y permite reintentar o consultar instrucciones.
3. **Given** una pérdida temporal del control, **When** se recupera dentro del plazo y los
   procesos siguen válidos, **Then** continúa y marca los huecos; si no, termina con seguridad.
4. **Given** un motor ausente, alterado o con salida inválida, **When** se intenta probar,
   **Then** no se produce un resultado exitoso y aparece un error accionable.

---

### User Story 4 - Consultar evolución e historial (Priority: P3)

Como técnico, quiero conservar sesiones y comparar pruebas equivalentes del mismo peer para
detectar cambios de rendimiento sin mezclar escenarios incompatibles.

**Why this priority**: Añade valor cuando medición e interpretación ya son fiables.

**Independent Test**: Con sesiones completas e incompletas, el usuario filtra, abre, compara y
elimina datos; las comparaciones solo usan cohortes compatibles.

**Acceptance Scenarios**:

1. **Given** sesiones guardadas, **When** se filtra por equipo, fecha o resultado, **Then** se
   obtienen las coincidentes y se puede reabrir su resultado.
2. **Given** al menos tres sesiones comparables, **When** se completa otra, **Then** se presenta
   su diferencia frente a las cinco comparables más recientes.
3. **Given** una cancelación con un sentido completo, **When** se acepta conservarla, **Then**
   aparece como incompleta; un fallo sin medición no se guarda como resultado.
4. **Given** una petición de borrado confirmada, **When** se ejecuta, **Then** solo se eliminan
   las sesiones elegidas y las restantes conservan su significado histórico.

---

### User Story 5 - Ejecutar pruebas avanzadas y UDP (Priority: P3)

Como técnico avanzado, quiero ajustar una lista cerrada de parámetros y ejecutar pruebas TCP
o UDP dentro de límites seguros para investigar casos que la prueba estándar no cubre.

**Why this priority**: Amplía el diagnóstico sobre validación y medición ya fiables.

**Independent Test**: Se solicitan valores mínimos, máximos e inválidos desde ambos extremos;
solo los válidos se ejecutan y UDP distingue tasa enviada, recibida y pérdida.

**Acceptance Scenarios**:

1. **Given** un plan avanzado válido, **When** el receptor lo revisa o autoacepta dentro de
   límites, **Then** ejecuta exactamente el plan mostrado.
2. **Given** parámetros fuera de rango o puertos solapados, **When** llegan al receptor,
   **Then** se rechazan aunque el peer sea de confianza.
3. **Given** una prueba UDP, **When** termina, **Then** muestra pérdida, tasa objetivo y tasa
   real, declarando si la tasa objetivo era aproximada o excedía la referencia.
4. **Given** una medición simultánea, **When** se presenta, **Then** separa ambos sentidos y no
   aplica un veredicto de asimetría que requiera ejecución secuencial.

---

### User Story 6 - Exportar y compartir resultados (Priority: P3)

Como técnico, quiero exportar sesiones en formatos legibles y estructurados, sabiendo qué datos
identificativos contienen y pudiendo anonimizarlos antes de compartirlos.

**Why this priority**: Facilita soporte y documentación después de tener resultados fiables.

**Independent Test**: Una sesión sintética con identificadores anidados se exporta en cada
formato, con y sin anonimización, y se verifica estructura, idioma y ausencia de identificadores.

**Acceptance Scenarios**:

1. **Given** una sesión seleccionada, **When** se exporta, **Then** se informa primero del
   contenido identificativo y se ofrece ocultar direcciones y direcciones físicas.
2. **Given** anonimización activa, **When** se generan archivos, **Then** los identificadores
   se eliminan también de bloques técnicos y nombres propuestos.
3. **Given** varias sesiones, **When** se exportan juntas, **Then** se conserva el vínculo entre
   sesión, dirección, equipo y muestras sin perder precisión numérica.

---

### User Story 7 - Adaptar y mantener la aplicación (Priority: P3)

Como usuario, quiero elegir idioma, apariencia y ciclo de vida, conservar mis preferencias y
actualizar con confirmación para que la aplicación encaje en mi entorno.

**Why this priority**: Completa la experiencia distribuible tras el recorrido principal.

**Independent Test**: Se cambian preferencias, se reinicia, se cierra durante distintos estados
y se simula una actualización válida e inválida. Se conservan decisiones y sesiones activas.

**Acceptance Scenarios**:

1. **Given** un idioma o tema elegido, **When** se reinicia, **Then** se restaura y todo el
   contenido visible usa recursos coherentes.
2. **Given** una sesión activa, **When** se intenta salir o actualizar, **Then** se requiere una
   decisión explícita y la sesión no se abandona silenciosamente.
3. **Given** una geometría de un monitor ausente, **When** se abre, **Then** la ventana queda
   visible y utilizable en un monitor presente.
4. **Given** una actualización no auténtica, incompleta o anterior, **When** se comprueba,
   **Then** no se instala y la función principal sigue disponible.

### Edge Cases

- Ningún adaptador, adaptador retirado durante la sesión o varias rutas posibles.
- Ningún peer, uno o varios; peer ocupado, incompatible o desaparecido.
- DNS inválido o no resoluble, IPv4/IPv6 y dirección con puerto explícito.
- Identidad cambiada para un peer conocido y código no coincidente.
- Solicitudes repetidas, fuera de estado, sobredimensionadas o por encima de límites.
- Cancelación simultánea, cierre inesperado, pérdida de canal y mensajes duplicados.
- Puertos parcialmente ocupados, sin rango alternativo o perfil de red público.
- Medición sin datos, campos ausentes, salida desconocida o contadores inválidos.
- Velocidades distintas, capacidad desconocida o muestras insuficientes.
- Espacio insuficiente, almacenamiento corrupto, migración fallida o esquema futuro.
- Texto remoto con controles, bidi, Unicode no normalizado o longitud excesiva.
- Exportación con fórmulas, identificadores anidados o fallo de destino.
- Alto contraste, reducción de movimiento, texto al 200 %, ventana mínima y DPI alto.
- Sin Internet, sin release estable o actualización durante una sesión.

## Requirements *(mandatory)*

### Functional Requirements

#### Alcance y experiencia general

- **FR-001**: El producto MUST medir conexiones entre dos equipos y explicar si el resultado es
  acorde con la capacidad conocida, sin presentarse como mero lanzador del motor.
- **FR-002**: La matriz de sistemas soportados MUST incluir Windows 10 22H2 y Windows 11,
  únicamente en arquitectura x64.
- **FR-003**: La v1 MUST funcionar sin cuentas, nube, telemetría, servidor central ni Internet
  para descubrir, autorizar, medir, interpretar y consultar datos locales.
- **FR-004**: La v1 MUST incluir H1, H2 y H3; un hito interno MUST NOT presentarse como v1 completa.
- **FR-005**: MUST ofrecer recorrido sencillo y detalle progresivo; opciones avanzadas dentro
  del mismo producto y flujo.
- **FR-006**: MUST estar disponible en español e inglés, con textos, errores, ayudas, informes
  y formatos completos en ambos idiomas.
- **FR-007**: El usuario MUST poder cancelar desde cualquier estado no terminal. La limpieza
  resultante la define FR-023 y la capacidad de respuesta bajo degradación, FR-044.
- **FR-008**: El nombre y detalles internos del motor MUST limitarse a detalles técnicos,
  licencias y acerca de.

#### Equipos, identidad y consentimiento

- **FR-009**: Cada instancia MUST poder iniciar y recibir pruebas sin configurar un rol.
- **FR-010**: Los peers MUST poder localizarse en LAN y conectarse manualmente por nombre o
  dirección cuando el descubrimiento no sea aplicable.
- **FR-011**: Descubrimiento, nombre, dirección e identificador visible MUST NOT establecer
  confianza por sí mismos.
- **FR-012**: El primer contacto MUST mostrar una prueba humana de identidad comparable en ambos
  equipos antes de guardar confianza.
- **FR-013**: Un cambio de identidad MUST invalidar confianza anterior y exigir verificación.
- **FR-014**: Conocido, favorito, de confianza y autoaceptado MUST ser estados distintos.
- **FR-015**: La autoaceptación MUST iniciar desactivada, requerir activación informada y no
  permitir planes inválidos.
- **FR-016**: Por defecto, cada prueba MUST requerir consentimiento local tras mostrar peer,
  sentido, tipo, duración, adaptador y aviso de tráfico intenso.

#### Sesión y medición

- **FR-017**: Cada instancia MUST admitir una sola sesión activa y rechazar otras explícitamente.
- **FR-018**: La prueba estándar MUST medir TCP en ambos sentidos de forma secuencial y usar
  parámetros recomendados derivados de la capacidad disponible.
- **FR-019**: Antes de medir, MUST comprobar motor, adaptador, familia de dirección, versión,
  espacio y recursos aplicables.
- **FR-020**: Antes de cada sentido, ambos extremos MUST comprobar conectividad, interfaces,
  puertos, permisos y preparación.
- **FR-021**: El receptor MUST revalidar el plan y construir una ejecución desde campos
  permitidos; MUST NOT aceptar argumentos libres.
- **FR-022**: Ambos extremos MUST mantener estados coherentes y rechazar transiciones o mensajes
  fuera de estado.
- **FR-023**: Cancelación, timeout, desconexión y cierre MUST converger en limpieza idempotente
  que no termine procesos ajenos ni deje recursos propios (complementa FR-007 y FR-044).
- **FR-024**: Una pérdida temporal MAY recuperarse si identidad y sesión coinciden; los huecos
  MUST marcarse y un sentido parcial MUST NOT reanudarse como continuo.
- **FR-025**: Una sesión completada y confirmada MUST conservar mismo identificador y resultado
  en ambos extremos; cualquier reconciliación degradada MUST declararse.
- **FR-026**: La precisión MUST tener prioridad sobre animación, logging y refresco visual.

#### Resultado e interpretación

- **FR-027**: La velocidad oficial MUST proceder del receptor; las muestras de interfaz solo
  describen evolución y MUST declarar que pueden incluir tráfico ajeno.
- **FR-028**: El resultado MUST separar hechos, observaciones, posibles causas y acciones
  (estructura de presentación; el contenido lo fija FR-031).
- **FR-029**: Una correlación MUST NOT presentarse como causa única demostrada (regla de
  prudencia aplicable a toda causa de FR-028).
- **FR-030**: Datos ausentes o inválidos MUST conservarse como no disponibles/no evaluables,
  nunca como cero, éxito o normalidad.
- **FR-031**: La interpretación MUST contemplar capacidad, rendimiento, estabilidad, asimetría,
  retransmisiones, CPU, coherencia y tráfico ajeno (dimensiones que alimentan FR-028).
- **FR-032**: Los valores provisionales MUST identificarse y validarse antes de publicación.
- **FR-033**: El resultado MUST conservar datos suficientes para explicar medición, plan, peer,
  interfaces, reglas y versiones.
- **FR-034**: Los parciales MUST distinguir sentidos completos e incompletos y guardarse solo
  cuando exista medición efectiva de ambos extremos del sentido.
- **FR-035**: Cada error visible MUST incluir explicación, posibles causas, acciones y código
  secundario de referencia.

#### Interfaz, diseño y accesibilidad

- **FR-036**: La interfaz MUST respetar Graphite Violet, sus tokens/componentes y apariencia
  de aplicación Windows.
- **FR-037**: El tema inicial MUST ser Oscuro; el usuario MUST poder elegir posteriormente
  entre Sistema, Claro y Oscuro.
- **FR-038**: Cada estado MUST comunicar significado con texto e icono/trazo además de color.
- **FR-039**: Toda acción MUST operar por teclado, con foco visible, lógico y recuperable.
- **FR-040**: Las ayudas MUST explicar términos no evidentes, funcionar por foco/puntero,
  cerrarse con Escape y no contener controles interactivos.
- **FR-041**: La interfaz MUST respetar reducción de movimiento y ampliación de texto sin pérdida.
- **FR-042**: La instalación sin red y la persistencia MUST estar disponibles desde H1; cada
  sentido MUST admitir entre 1 y 64 streams en ejecución secuencial y entre 1 y 32 en
  ejecución simultánea; la actualización MUST consultar un único
  manifiesto estable (`latest.json`) cuyo artefacto apunte a la URL inmutable de su etiqueta
  `vX.Y.Z`, nunca a un binario mutable; y la accesibilidad completa MUST entregarse en H2,
  conforme a `Historias.md`.
- **FR-043**: La ventana MUST funcionar en tamaños compacto/estándar/amplio y restaurarse visible
  tras cambios de monitor o escala.
- **FR-044**: Durante medición, actualizaciones visuales MUST estar acotadas y cancelar MUST
  responder aunque presentación o logging estén degradados (complementa FR-007 y FR-023).

#### Historial, exportación y ajustes

- **FR-045**: Identidad, peers, preferencias y resultados MUST persistir localmente con esquema,
  validación e integridad.
- **FR-046**: Migraciones MUST conservar originales y backup recuperable; una versión antigua
  MUST rechazar datos futuros desconocidos.
- **FR-047**: Historial MUST listar, filtrar, buscar, abrir, repetir y eliminar sesiones con
  confirmación cuando exista pérdida.
- **FR-048**: Comparaciones MUST usar sesiones compatibles por peer, interfaces, protocolo,
  sentidos y parámetros relevantes.
- **FR-049**: MUST exportar sesiones individuales/múltiples en documento imprimible, formato
  estructurado y tabla de resumen/muestras.
- **FR-050**: Antes de exportar MUST informar datos identificativos y ofrecer anonimización
  que alcance bloques técnicos anidados.
- **FR-051**: Exportación tabular MUST conservar precisión/idioma y neutralizar fórmulas remotas.
- **FR-052**: Ajustes MUST cubrir idioma, apariencia, accesibilidad, cierre, red, descubrimiento,
  confianza, firewall, datos, actualizaciones y diagnóstico según el hito.
- **FR-053**: Un ajuste que afecte una sesión activa MUST rechazarse o diferirse explícitamente.

#### Seguridad, privacidad y diagnóstico

- **FR-054**: El canal peer MUST proteger confidencialidad e identidad mutua; un peer no
  verificado MUST NOT iniciar medición autorizada.
- **FR-055**: Mensajes, archivos, datos persistidos y texto remoto MUST validarse por tipo,
  estado, longitud, rango, tamaño y versión antes de efectos.
- **FR-056**: Texto remoto MUST normalizarse, limitarse y mostrarse como texto no ejecutable.
- **FR-057**: La aplicación principal MUST funcionar sin elevación; cambios de firewall MUST
  requerir consentimiento local y limitarse a recursos propios.
- **FR-058**: Claves, códigos de verificación, credenciales y secretos MUST quedar fuera de UI
  no autorizada, logs, diagnósticos y exportaciones.
- **FR-059**: Logs/diagnósticos MUST ser locales, estructurados, acotados y saneados; su fallo
  MUST NOT impedir cancelación segura.
- **FR-060**: MUST limitar conexiones, solicitudes, tamaños, frecuencia y buffers.
- **FR-061**: Antes de copiar diagnóstico, MUST informar datos incluidos y ofrecer anonimización.

#### Distribución y ciclo de vida

- **FR-062**: Instalar MUST dejar aplicación, motor, WebView2 offline y licencias operativos sin
  red; desinstalar MUST retirar solo recursos propios (binarios, reglas de firewall propias,
  autoarranque) y conservar los datos de usuario por defecto, ofreciendo borrarlos.
- **FR-063**: MUST verificar integridad del motor antes de usarlo y negarse si falta o se alteró.
- **FR-064**: Actualizaciones MUST ser auténticas, desactivables, confirmadas y pospuestas
  durante una sesión activa.
- **FR-065**: Todas las vías de cierre MUST aplicar decisión coherente y confirmar si se cancela.
- **FR-066**: Una release MUST ser trazable a versión/artefacto inmutable y superar pruebas
  aplicables de instalación, actualización, desinstalación y recuperación.
- **FR-067**: La v1 MUST incluir licencia, atribuciones y avisos de distribución comprensibles.

### Key Entities

- **Instancia**: ejecución en un equipo; posee identidad, nombre y estado.
- **Peer**: instancia remota identificada; puede tener alias, favorito, confianza y dirección.
- **Solicitud**: petición con plan, duración, identidad e interfaz propuesta.
- **Plan**: protocolo, sentidos, duración, concurrencia, tamaños, puertos, familia e interfaz.
- **Sesión**: prueba entre dos instancias con estado, plan, eventos y resultados.
- **Dirección**: sentido con emisor, receptor, hechos y muestras.
- **Resultado**: snapshot histórico con hechos, interpretación, procedencia y versiones.
- **Muestra**: observación temporal de tráfico y CPU durante una dirección.
- **Interfaz**: snapshot del adaptador usado, direcciones y capacidad.
- **Regla de interpretación**: transforma hechos en nivel, observaciones, causas y acciones.
- **Preferencias**: configuración validada de usuario con versión de esquema.
- **Regla de firewall**: recurso propio con consentimiento, alcance y estado.
- **Registro de diagnóstico**: evento local saneado y acotado, sin autoridad.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Dos usuarios pueden completar conexión, verificación, aceptación y prueba estándar
  bidireccional sin ayuda externa.
- **SC-002**: La prueba estándar presenta resultado en ambos equipos como máximo 75 segundos
  después de la aceptación, excluyendo decisiones del usuario (plan estándar de ~55 s más
  preparación, cambio de sentido y reconciliación).
- **SC-003**: El 100 % de sesiones completadas muestra el mismo identificador, plan y cifras en
  ambos extremos o declara reconciliación degradada.
- **SC-004**: Cancelar desde cualquier estado activo no espera más de 2 segundos la confirmación
  remota, libera los recursos propios y deja ambos extremos en un estado terminal explícito.
- **SC-005**: El 100 % de resultados insuficientes muestra «no evaluable/no disponible» y
  ninguno se clasifica correcto por defecto.
- **SC-006**: El 100 % de errores visibles incluye explicación humana, acción cuando exista y
  código secundario.
- **SC-007**: Ningún recorrido ordinario menciona el motor fuera de detalles/licencias/acerca de.
- **SC-008**: Todas las acciones de las pantallas entregadas funcionan con teclado y conservan
  contenido y función con escalado de Windows y ampliación de texto de hasta el 200 %.
- **SC-009**: Español e inglés tienen las mismas claves y formatos/plurales coherentes.
- **SC-010**: Durante una prueba, la aplicación completa excluido el motor utiliza menos del
  5 % de un núcleo de referencia y actualiza la presentación como máximo 4 veces por segundo
  en cada ejecución de laboratorio requerida.
- **SC-011**: Ninguna entrada inválida, grande, repetida o fuera de estado inicia procesos,
  cambia confianza, eleva privilegios o persiste un resultado válido.
- **SC-012**: Toda migración distribuida conserva backup restaurable y recupera originales ante
  los fallos contemplados.
- **SC-013**: La anonimización elimina el 100 % de identificadores sintéticos definidos de todos
  los campos y bloques anidados exportados.
- **SC-014**: Instalación, desinstalación y primer arranque funcionan sin Internet en la matriz
  ratificada, salvo consulta/descarga opcional de actualización.
- **SC-015**: Antes de publicar v1, V-01 a V-12 aplicables están cerradas con evidencia y no
  quedan valores provisionales presentados como validados.

## Assumptions

- La feature describe la v1 pública; H1, H2 y H3 son checkpoints internos.
- Se usa entre dos equipos Windows con conectividad IP; no mide acceso a Internet.
- Solo existe un motor en v1, sin argumentos libres ni receptor externo manual.
- Cloud, cuentas, telemetría, servicio sin sesión, monitorización continua, portable, Linux,
  macOS, ARM64 y otros motores quedan fuera de v1.
- El historial no caduca automáticamente; el usuario controla su eliminación.
- Los valores provisionales requieren validación antes de release.
- La constitución 0.6.0 no está ratificada. Para esta feature, el propietario eligió Windows
  10 22H2/Windows 11 x64 en Q1, mantener las reglas de `Historias.md` en Q3 y tema Oscuro
  inicial en Q4. Q2 corresponde al plan de calidad. Estas respuestas no modifican ni ratifican
  por sí solas `.specify/memory/constitution.md`.
- `Design/` rige presentación donde no contradice producto, seguridad o accesibilidad; la
  maqueta es ilustrativa.
- G1–G6 bloquean únicamente implementación/aceptación afectadas; trabajo independiente puede
  continuar.
