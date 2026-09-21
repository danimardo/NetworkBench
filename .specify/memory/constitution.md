<!--
Sync Impact Report — 2026-09-21
Versión: 0.3.0 -> 0.4.0 (política transversal de logging y depuración).
Principio XIII añadido: abstracción, niveles, privacidad, formato y coste del logging.
Tabla de versiones, CI y discrepancias actualizadas; no se eliminan principios.
Plantilla resuelta con resolve-template.ps1 constitution-template -Json.
Solo se modifica constitution.md; no se implementan comandos, esquemas ni configuración.
Seguimiento: Q1-Q4, TODO(RATIFICATION_DATE), verificaciones V-01 a V-12 y puertas G1-G6.
Informe temporal de revisión: retirar antes del commit de ratificación.
-->

# NetworkBench Constitution

Propuesta basada en `Historias.md` y el sistema visual existente en `Design/`.
**Estado: pendiente de las decisiones Q1-Q4 y de ratificación por el propietario.**
Las reglas expresan una propuesta completa; una pregunta sin respuesta no es una aprobación.
La versión de la constitución es independiente de la versión de la aplicación.

## Core Principles

### I. Precisión y honestidad del resultado

- NetworkBench DEBE medir una conexión entre dos equipos y explicar sus resultados.
  Prioridades: seguridad y consentimiento; integridad y precisión; accesibilidad y comprensión;
  rendimiento de la aplicación; efectos visuales.
- La velocidad oficial de cada dirección DEBE proceder del receptor NTTTCP.
  Los contadores del adaptador solo alimentan series temporales; UI e informes DEBEN declarar
  que incluyen tráfico ajeno. NO sustituyen la velocidad oficial.
- Hechos, observaciones, posibles causas y acciones DEBEN mantenerse separados en modelo,
  interfaz e informes. Una correlación NO se presenta como causa demostrada.
- Datos ausentes, insuficientes o inválidos NO se convierten en cero ni éxito. Sin indicadores
  evaluables, el resultado DEBE decir «No evaluable», nunca «Correcto».
  Las sesiones parciales DEBEN identificarse y conservar solo mediciones efectivas.
- Reglas y umbrales DEBEN residir en Rust y un único `thresholds.json` embebido.
  Las ayudas consumen sus valores como parámetros. Cada resultado DEBE conservar versión
  de app, motor, reglas y hash de umbrales para poder explicar el diagnóstico histórico.
- Los valores provisionales de Historias.md requieren las verificaciones V-01 a V-12.
  Una cifra especificada NO es evidencia de exactitud.

### II. Separación de presentación, dominio e infraestructura

- Svelte 5 y TypeScript estricto gestionan presentación, interacción, accesibilidad, traducción
  y formato. NO deciden veredictos, confianza, permisos ni ejecución del benchmark.
- Rust contiene planes, validación, estados, interpretación y casos de uso. El dominio DEBE
  probarse sin Tauri, WebView2, SQLite, red ni procesos reales.
- El dominio define contratos; los adaptadores implementan Windows, persistencia, TLS,
  descubrimiento, reloj y motor. El dominio NO depende de APIs Tauri ni componentes Svelte.
- Comandos Tauri DEBEN ser adaptadores finos con validación, errores tipados y capacidades
  mínimas. El frontend NO recibe acceso general a shell, procesos, SQL o archivos.
- Solo `engine/ntttcp/` construye argumentos y analiza XML. El resto consume contratos
  versionados `BenchmarkPlan`, `EngineResult` y `SessionResult`.
- Reloj, identificadores, procesos, red y almacenamiento DEBEN ser sustituibles en pruebas.
  Rust repite toda validación de confianza aunque la UI ya haya validado.
- Contratos IPC/red DEBEN verificarse mediante fixtures o generación de tipos; NO mantener
  dos definiciones manuales sin prueba de equivalencia.

### III. Consentimiento, privilegio mínimo y entradas no confiables

- Control: TLS 1.3 mutuo, identidad Ed25519 y huella SHA-256. mDNS, IP, hostname e instanceId
  NO autentican. Peer nuevo requiere verificar código; huella cambiada invalida confianza
  anterior y aceptación automática.
- Conocido, favorito, de confianza y aceptación automática son propiedades distintas.
  Aceptación automática empieza desactivada por peer y exige activación informada.
- Sin emparejamiento y autorización válidos NO arranca el benchmark. Una sesión activa por
  instancia. Los límites de Historias.md §8.6 DEBEN acotar conexiones, mensajes y solicitudes.
- Mensajes se validan por tipo, rango, tamaño, estado, sesión y correlación. Campos opcionales
  desconocidos pueden ignorarse; mensajes o transiciones inválidos no se ejecutan.
  Duplicados y reconexiones NO repiten procesos ni elevaciones.
- Solo se ejecuta el motor empaquetado por ruta absoluta y argumentos estructurados:
  nunca shell ni argumentos libres. Rechazar XML con DTD/entidades externas, expansión
  ilimitada, rutas remotas o contenido renderizable como HTML.
- La app principal NO se ejecuta elevada. El helper de firewall usa COM, lista cerrada de
  operaciones/programas/perfiles/puertos, consentimiento local y UAC. Proteger el intercambio
  con el helper contra sustitución de archivos y alteraciones entre validación y lectura.
- Clave privada cifrada con DPAPI de usuario, nunca en logs/exportaciones.
  Verificar hash de NTTTCP al arrancar y antes de cada prueba.
- WebView solo con recursos locales, CSP restrictiva, sin eval/scripts remotos/permisos
  globales. Enlaces externos limitados a esquemas y destinos autorizados.
  Todo texto recibido se sanea y se muestra como texto.

### IV. Datos locales, íntegros y recuperables

- SQLite embebido con rusqlite, WAL y claves foráneas activadas en
  `%APPDATA%\NetworkBench\networkbench.db`. NO usar localStorage, IndexedDB o nube como
  fuente de verdad.
- Ajustes: `%APPDATA%\NetworkBench\settings.json`; identidad: subcarpeta `identity`;
  logs: `%LOCALAPPDATA%\NetworkBench\logs`; temporales:
  `%TEMP%\NetworkBench\<sessionId>`. Resolver rutas mediante APIs de Windows.
- Ajustes con versión de esquema, validación y escritura atómica. Sesiones y datos asociados
  se guardan en transacción, con sessionId único e idempotencia.
- Persistir identidad, peers y resultados desde H1; H3 añade pantalla y gestión de historial.
  Conservar snapshots de interfaces y resultados históricos sin recalcularlos silenciosamente.
- Antes de migrar: backup coherente mediante API SQLite o equivalente compatible con WAL,
  comprobación de integridad y prueba de restauración. Copiar solo un .db abierto no basta.
  Un fallo preserva originales y backup; nunca reinicializa silenciosamente.
  Una app antigua NO escribe en un esquema más nuevo que no comprende.
- Historial sin caducidad automática; borrar requiere confirmación y desinstalar conserva
  datos por defecto. Logs: rotación diaria, máximo 10 archivos y 50 MB.
  Conservar tres backups de migración correctos; no purgar durante una prueba.
- Unidades: bit/s y bytes enteros; intervalos en ms y fechas UTC ISO 8601.
  En IPC/JSON, u64 que pueda exceder Number.MAX_SAFE_INTEGER se codifica como cadena decimal
  según contrato; NO convertir a number perdiendo precisión.
- Funcionalidad disponible sin Internet. Sin cuentas, telemetría ni analítica.
  Actualizaciones desactivables, sin enviar identidad, resultados o redes.
  Documentar aparte las conexiones de actualización propias de WebView2 Evergreen.
- Exportación informa del contenido y ofrece anonimización de campos, XML, stdout/stderr,
  comandos y nombres de archivo. Si un bloque crudo no se puede sanear con garantías,
  omitirlo y declarar la omisión. CSV escapa celdas y neutraliza fórmulas de texto no confiable.
- La BD no se cifra íntegramente en v1: protección por permisos del perfil Windows;
  DPAPI para secretos. NO prometer cifrado inexistente.

### V. Interfaz comprensible y coherente

- Prueba estándar sencilla, detalle progresivo y opciones avanzadas dentro del flujo.
  La velocidad es la cifra principal; el porcentaje es apoyo. NTTTCP solo se menciona
  en Detalles técnicos y Acerca de, nunca en el recorrido ordinario.
- Cada error explica qué ocurrió, posibles causas y acciones; código NB-* secundario.
  Cancelación siempre disponible durante la sesión.
- Reutilizar Graphite Violet, tokens y componentes de Design, adaptados a contratos reales.
  Una maqueta NO demuestra implementación. Tema inicial propuesto: Oscuro; opciones
  Sistema/Claro/Oscuro (Q4).
- `Design/` es el sistema de diseño y la referencia obligatoria de reglas visuales y de
  interacción. Antes de crear o modificar UI, consultar `Design/README.md`,
  `Design/DESIGN_TOKENS.md`, `Design/FLUJOS.md`, `Design/PANTALLAS-H1.md`,
  `Design/RESPONSIVE-I18N.md`, `Design/AVISOS.md` y los componentes/tokens de `Design/src/lib/`.
  Reutilizar los patrones existentes y documentar cualquier ampliación del sistema.
  Ante contradicciones, aplicar la jerarquía de Governance; no copiar errores de la maqueta.
- Tailwind CSS DEBE usarse como herramienta de estilos, integrado mediante el plugin oficial
  de Vite y compilado a CSS local, sin CDN ni generación de estilos en runtime.
  Sus utilidades DEBEN consumir los tokens del sistema mediante un mapeo central de tema
  (`@theme`/`@theme inline` cuando corresponda), sin crear otra paleta o escala visual.
  Los breakpoints y variantes de tema DEBEN respetar Design y las decisiones de producto,
  no los valores por defecto de Tailwind. Se permite CSS propio para materiales, animaciones
  y componentes existentes; no se exige reescribirlos íntegramente como utilidades.
- Las clases Tailwind DEBEN ser detectables en build (nombres completos en mapas explícitos,
  no concatenaciones dinámicas). Valores arbitrarios solo para referencias a tokens o
  excepciones estructurales documentadas. Revisar Preflight, cascada y resets para conservar
  controles, foco y estilos de impresión. Verificar el CSS generado en el WebView2 mínimo.
- Centralizar color, tipografía, espaciado, radios, materiales y animación en tokens.
  Permitir constantes estructurales documentadas; el verificador NO confunde coordenadas
  SVG o números de dominio con estilos prohibidos.
- Apariencia de escritorio, barra propia, geometría persistente y comportamiento de ventana
  cumplen Historias.md §16.9 y §16.13–16.15.
- UI, ayudas, errores e informes en es/en desde su hito, con plurales/formatos externalizados
  y claves sincronizadas. Almacenar claves i18n y parámetros, no solo textos traducidos.
- Tooltips explican términos, funcionan con foco/puntero/Escape, usan aria-describedby
  y no contienen controles interactivos.

### VI. Accesibilidad verificable desde la primera pantalla

- Cada pantalla entregada cumple los criterios aplicables de WCAG 2.2 AA, con WCAG2ICT
  para escritorio. NO posponer la accesibilidad de H1 a H2.
- Todas las acciones accesibles por teclado; foco visible, orden lógico, retorno del foco
  al cerrar diálogos y ausencia de trampas. Controles propios conservan semántica, nombres,
  estados y patrones de teclado equivalentes.
- Contraste mínimo 4,5:1 para texto normal; 3:1 para texto grande y elementos relevantes.
  Estados/series usan texto, iconos o trazos además de color. Cristal se vuelve opaco
  cuando comprometa legibilidad o alto contraste.
- Ampliación propia de texto hasta 200 % sin pérdida de contenido o funciones:
  ampliar 100/115/130 % con 150/175/200 %. Bloquear zoom de navegador no exime este requisito.
  Probar ventana mínima, DPI alto, ambos idiomas y temas.
- Respetar prefers-reduced-motion y reducción de Windows incluso con ajuste local «No».
  «Sí» siempre reduce; «No» nunca anula la preferencia del sistema. Ninguna animación
  retrasa aceptar, rechazar, cancelar o mostrar información.
- Anunciar fases/errores al lector, no cada muestra a 4 Hz. Gráficas con resumen/tablas
  accesibles y navegación por muestras.
- Antes de cerrar un hito: teclado, Narrador, contraste alto, reducción de movimiento,
  idiomas y escalado. Las pruebas automáticas NO sustituyen esta revisión manual.

### VII. Medición aislada del coste visual y cierre seguro

- Red, procesos y muestreo en Rust fuera del hilo UI. Inicialmente muestras cada 500 ms
  y eventos de gráfica a máximo 4 Hz; colas acotadas. La congestión visual no bloquea control.
- Durante RUNNING_* solo se animan gráfica/conexión, máximo 30 fps, sin blur animado
  ni efectos a pantalla completa. Los huecos NO se interpolan como datos medidos.
- Presupuesto: menos del 5 % de UN procesador lógico durante medición, sumando app y
  procesos WebView2 atribuibles, excluyendo NTTTCP. Fórmula:
  `100 × suma(delta CPU usuario+sistema) / delta tiempo real`, sin dividir por nº de CPUs.
  Registrar media y picos en ventanas de un segundo.
- V-04: cinco ejecuciones release por escenario, con/sin animación, hardware y entorno
  documentados (CPU/GPU/RAM/NIC, Windows, WebView2, resolución/DPI y plan energético).
  Cada ejecución cumple el límite medio. V-12 registra GPU y compara materiales;
  no se inventa un límite GPU ausente de la especificación.
- Si falla: reducir primero presentación/efectos. Cambiar muestreo requiere medir impacto
  estadístico y versionar decisión/umbrales.
- NTTTCP pertenece a un Job Object con cierre de descendientes. Cancelación, timeout,
  desconexión y salida convergen en limpieza idempotente. NO terminar procesos solo
  por nombre/ruta; demostrar propiedad de la instancia.
- Duraciones con reloj monotónico; UTC para fechas. START.startAt NO presupone relojes
  sincronizados: fijar negociación de inicio/offset y tolerancia antes de implementar.

### VIII. Pruebas útiles y cobertura independiente

- Todo cambio que afecte a componentes Svelte, TypeScript, navegación, layouts, configuración
  de Svelte/Vite o contratos de componentes DEBE validarse mediante `pnpm check`.
  El repositorio DEBE proporcionar ese comando reproducible, usando el svelte-check local
  fijado y el tsconfig real: `svelte-check --tsconfig ./tsconfig.json --fail-on-warnings`.
  Si hay varios tsconfig de aplicación, el comando DEBE cubrirlos sin excluir código propio
  para obtener un resultado favorable; cualquier cambio de ruta se refleja en el script.
- Este proyecto usa Svelte + Vite, no SvelteKit: NO se añade `svelte-kit sync` ni se instala
  SvelteKit para satisfacer este control. Si una enmienda futura adopta SvelteKit, `pnpm check`
  DEBE ejecutar primero `svelte-kit sync` y después svelte-check con el tsconfig real.
- La validación completa y CI DEBEN ejecutar `pnpm check` antes de tests unitarios, build
  y E2E, bloqueando esas etapas si falla. Cero errores y cero advertencias es condición de
  finalización; no se permite ocultarlos mediante filtros, exclusiones o supresiones evasivas.
- Los agentes y desarrolladores DEBEN registrar comando, revisión validada, código de salida
  y resumen de diagnósticos. NO afirmar «validado» si no se ejecutó, quedó incompleto o
  corresponde a una revisión anterior al último cambio afectado. Documentación pura no
  exige ejecutar svelte-check, pero tampoco autoriza a afirmar que el código ha pasado.
- Fallos preexistentes DEBEN identificarse explícitamente con ubicación y evidencia de
  procedencia cuando sea posible. No se ocultan, ignoran ni corrigen fuera del alcance sin
  autorización. Su existencia mantiene el control fallido; se informa del bloqueo concreto.
- Propuesta Q2: Rust, al menos 80 % de líneas; TypeScript/Svelte, al menos 80 % en cada
  métrica de líneas, sentencias, funciones y ramas. No promediar capas ni métricas.
- Cada módulo crítico Rust (estados/control, emparejamiento, planes, parser, diagnóstico,
  saneamiento) alcanza 90 % de líneas. Herramientas: cargo-llvm-cov y Vitest/V8.
- Incluir código propio no ejecutado por tests. Excluir solo generado, terceros, fixtures,
  tipos sin ejecución y maqueta Design. Código trasladado de Design a producción sí cuenta.
  FFI propio requiere contratos y pruebas reales, no exclusión general de cobertura.
- Probar límites exactos, nulos y divisiones por cero; XML real del motor fijado, corrupto,
  incompleto y desconocido; mensajes inválidos/truncados/grandes, identidad cambiada,
  duplicados, reconexión y estados no permitidos.
- Integración con dos pares aislados: éxito, rechazo, cancelación desde ambos extremos,
  caída de canal/proceso, persistencia y recuperación. Harness con perfiles/mutex/puertos
  exclusivos de test; NO debilitar instancia única de producción.
- UI por comportamiento observable, complementada con Tauri real y dos equipos Windows.
  Playwright web NO demuestra IPC, UAC, firewall, instalación ni WebView2 empaquetado.
- Cada defecto lógico corregido incluye regresión automatizada cuando sea posible.
  No exigir TDD ceremonial o tests que reproduzcan implementación sin comprobar conducta.
  Cobertura nunca sustituye aceptación ni evidencia empírica.

### IX. Dependencias y publicaciones trazables

- Pila obligatoria según tabla. Añadir dependencias exige necesidad, licencia, mantenimiento,
  impacto CPU/tamaño y compatibilidad Windows. Sin segundo framework UI/backend/motor en v1.
- Versiones directas exactas; transitivas en pnpm-lock.yaml y Cargo.lock versionados.
  CI frozen/locked. Compiladores/herramientas fijados; sin latest, comodines o ramas flotantes.
- Cada release conserva inventario, hashes, toolchain, imagen CI, componentes Microsoft,
  fuentes y acciones por SHA completo. windows-latest NO es un entorno reproducible.
- GPL-3.0-or-later y SPDX para código propio; atribuciones/licencias de lo redistribuido
  en THIRD_PARTY_NOTICES.md y el instalador. No redistribuir fuentes del SO sin permiso.
  Vulnerabilidades aplicables críticas/altas bloquean release; falsos positivos documentados.
- Publicar solo con etiqueta explícita vX.Y.Z y controles sobre ese commit. Secretos de firma
  nunca en logs o PR de forks; no publicar automáticamente desde main.
- NSIS por máquina, x64, es/en, WebView2 offline. Actualizaciones firmadas mediante Tauri,
  confirmadas antes de instalar y nunca durante sesión activa.
  Documentar ausencia de Authenticode v1 sin garantizar evasión de SmartScreen/políticas.

### X. Trazabilidad y decisiones explícitas

- Cada requisito implementado enlaza sección de Historias.md, hito, tarea y evidencia.
  Hito interno NO equivale a v1 completa; v1 pública exige H1, H2 y H3.
- Referencias a documentos ausentes o apartados inexistentes NO crean requisitos inferidos.
  Recuperar textos del antiguo Historias.md §§29–54 o definir criterios autosuficientes
  antes de cerrar las tareas afectadas.
- Cada incertidumbre tiene responsable, evidencia y condición de cierre.
  Un pendiente bloquea la funcionalidad afectada, no trabajo independiente.
- Reconciliar discrepancias en la primera planificación, no elegir silenciosamente
  el documento más cómodo durante implementación.

### XI. Validación runtime en toda frontera de confianza

- Todo dato externo o no confiable DEBE validarse en runtime antes de entrar en el dominio
  o provocar efectos. El tipado estático, `as`, `as unknown as`, `!`, `satisfies` o un genérico
  de API NO constituyen validación. En TypeScript las entradas sin validar se reciben como
  `unknown` y solo se consumen tras parsing satisfactorio.
- Zod es la librería estándar para validación y parsing en TypeScript. Los tipos DEBEN
  inferirse con `z.infer`, o `z.input`/`z.output` cuando existan transformaciones; no duplicar
  interfaces equivalentes sin comprobación. Los esquemas se centralizan por contrato y se
  reutilizan entre formularios, adaptadores y pruebas.
- Toda frontera de entrada/salida DEBE tener contrato y esquema: formularios/FormData,
  parámetros URL, configuración, variables de entorno, respuestas/eventos/comandos Tauri,
  red, archivos, datos persistidos, importaciones y exportaciones. APIs, webhooks,
  localStorage o salidas de LLM, si una futura feature autorizada los incorpora, reciben
  el mismo tratamiento; enumerarlos aquí NO amplía el alcance ni autoriza esos servicios.
- Las salidas se validan antes de cruzar la frontera y las entradas al recibirlas.
  En Rust, el esquema se expresa mediante tipos Serde y validadores explícitos de dominio;
  deserializar sin comprobar rangos e invariantes no basta. Zod no se ejecuta en Rust ni
  sustituye su validación autoritativa. Fixtures compartidos DEBEN comprobar equivalencia
  entre contratos Rust y Zod para casos válidos e inválidos.
- Los esquemas DEBEN definir obligatoriedad, nulos, rangos, longitudes, unidades, versiones,
  campos desconocidos y conversiones. Coerciones solo explícitas y no ambiguas: por ejemplo,
  la cadena `false` NO se convierte en verdadero por su mera existencia. Los límites de
  bytes/profundidad/colecciones se aplican antes de parsing costoso. No se aceptan defaults
  que conviertan un dato crítico ausente o inválido en una operación autorizada.
- Una validación fallida NO produce efectos de negocio. Devolver un error estructurado
  con `code`, `issues` (ruta y motivo seguro) y clave i18n/parámetros cuando sea visible.
  No exponer payload completo, secretos ni detalles internos. El rechazo de un mensaje de
  red sigue además la política de cierre/log del protocolo; un formulario conserva errores
  por campo sin convertir un fallo esperable en excepción sin gestionar.
- Cada contrato DEBE probar valores válidos, inválidos, límites, omisiones y versiones.
  Cambiar un esquema exige revisar consumidores, compatibilidad y migraciones afectadas.

### XII. Configuración explícita y separación de secretos

- `.env` se reserva a configuración de desarrollo/build; NO es almacén de identidad,
  preferencias del usuario o secretos distribuibles. La app instalada usa settings.json,
  SQLite y DPAPI según el principio IV y NO necesita un `.env` junto al ejecutable.
- Se prohíbe `process.env` en el código de aplicación. El frontend accede a configuración
  solo mediante un módulo tipado y validado con Zod; únicamente ese adaptador puede leer
  `import.meta.env`. Prohibidos polyfills de process y la inyección del entorno completo
  mediante `define`, serialización o comandos IPC.
- Con Vite, las variables públicas propias usan `VITE_` y una lista explícita de claves
  permitidas. Todo valor enviado a WebView o incluido en el bundle se considera público:
  nunca claves privadas, contraseñas, tokens o credenciales, aunque su nombre sugiera secreto.
  NO ampliar envPrefix para exponer variables privadas o prefijos indiscriminados.
- Los scripts y configuración Node de build usan un adaptador aislado con `loadEnv` de Vite
  y validación Zod, seleccionando solo claves necesarias. La prohibición de process.env
  se aplica al código propio de aplicación; cualquier acceso imprescindible en herramientas
  Node queda limitado a ese adaptador de infraestructura, documentado y nunca importable
  desde el frontend. No se pretende modificar cómo leen el entorno las herramientas externas.
- Rust lee únicamente variables privadas autorizadas mediante un adaptador de configuración
  tipado y validado antes de iniciar servicios; no dispersar std::env por el dominio.
  No introducir un runtime JavaScript en Rust para validar. Claves de firma de publicaciones
  se inyectan desde secretos CI en la etapa necesaria y nunca se empaquetan.
- Cada variable DEBE declarar nombre, propósito, ámbito público/privado, tipo, obligatoriedad,
  default seguro si existe y momento de lectura. Validar configuración propia en el arranque
  de desarrollo/build con Zod, antes de montar el frontend y al iniciar Rust donde corresponda.
  Falta o invalidez de configuración crítica detiene la etapa afectada antes de abrir red,
  ejecutar pruebas de rendimiento o escribir datos. El error identifica la clave, no su valor.
- Versionar `.env.example` sin secretos, con valores ficticios y documentación suficiente.
  Excluir de Git y del empaquetado `.env`, `.env.local`, `.env.<modo>` y sus variantes locales;
  ejemplos solo mediante excepción explícita. CI inyecta configuración de prueba reproducible
  y nunca depende del archivo personal del desarrollador. No imprimir el entorno completo.
  Un secreto filtrado requiere revocación/rotación, no solo borrarlo del archivo.
- Precedencia para Vite: entorno del proceso sobre archivos; archivo del modo sobre genérico
  y variante local sobre su equivalente no local. DEBE documentarse el modo usado en cada
  comando. Cambios en `.env` requieren reiniciar Vite; valores embebidos requieren reconstruir
  y redistribuir. Cambiar un archivo NO actualiza una app ya compilada. Configuración mutable
  del usuario se aplica mediante settings.json y validación, no mediante variables de build.
- `$env/dynamic/private`, `$env/dynamic/public`, `PUBLIC_` y la protección de `.server.ts`
  son convenciones de SvelteKit y NO se usan en esta arquitectura. Una adopción futura
  autorizada DEBE revisar esta política, aislar imports privados del cliente y validar con
  Zod al arrancar el servidor. «Dynamic» no garantiza recarga automática de archivos `.env`.
- Tests y CI DEBEN comprobar configuración ausente/inválida, conversiones, aislamiento de
  módulos privados y ausencia de secretos/archivos `.env` en el bundle y los artefactos.

### XIII. Logs estructurados, seguros y útiles para depuración

#### Tecnología y API única

- Rust DEBE usar `tracing` y `tracing-subscriber`; el frontend TypeScript DEBE usar
  `loglevel`, con las versiones fijadas en esta constitución. Pino pertenece al ecosistema
  Node.js y NO se incorpora al backend Rust ni justifica añadir un servidor Node.
- Todo código de funcionalidad DEBE emitir logs mediante una abstracción propia de logger:
  `src/lib/logging/` en TypeScript y `src-tauri/src/logging/` en Rust, con contrato común
  de niveles y eventos. Son adaptadores por lenguaje, no un módulo JavaScript dentro de Rust.
  Solo esos adaptadores importan directamente las librerías de logging.
- Quedan prohibidas las llamadas directas a `console.log/debug/info/warn/error/trace`,
  `console.table` y equivalentes, así como `println!`, `eprintln!` o `dbg!` para registrar
  diagnósticos en código de aplicación. La salida a consola solo se permite dentro del sink
  del logger y en herramientas ajenas a la app; no se prohíbe el funcionamiento interno
  de loglevel. Las macros de tracing también quedan encapsuladas por el adaptador Rust.
- Los wrappers DEBEN admitir un sink sustituible en tests y contexto tipado. La lógica
  de negocio no conoce bibliotecas, formatos, rutas de archivo ni variables de entorno.
  No registrar argumentos automáticamente mediante instrumentación sin una lista permitida.

#### Niveles y configuración

| Nivel | Uso obligatorio |
|---|---|
| `trace` | Detalle excepcional de diagnóstico, nunca muestras/payloads sin límites |
| `debug` | Decisiones internas y datos técnicos saneados para investigar un problema |
| `info` | Hitos de ciclo de vida: inicio/fin, transición de sesión, actualización o cambio de configuración relevante |
| `warn` | Degradación o anomalía recuperable que merece revisión |
| `error` | Operación fallida o estado que impide continuar una función |
| `off` | Desactivar emisión; no es un nivel de evento |

- El umbral incluye el nivel seleccionado y los más graves. Un error irrecuperable usa
  `error` con `fatal: true`, no un nivel diferente por biblioteca. Cancelar o rechazar
  voluntariamente una prueba no es un error. Un mismo fallo se registra una vez en la
  frontera responsable; propagarlo no autoriza a duplicarlo en todas las capas.
- Backend: `LOG_LEVEL`; frontend Vite: `VITE_LOG_LEVEL`, no `PUBLIC_LOG_LEVEL` de SvelteKit.
  Valores permitidos: los seis de la tabla, en minúsculas. Variables ausentes usan los
  defaults documentados; valores inválidos fallan antes de arrancar funcionalidad.
  Validación Rust en backend y Zod en el adaptador de entorno TypeScript.
- Defaults: desarrollo `debug` en ambos; producción `info` en backend y `warn` en frontend.
  El `info` de producción es de bajo volumen: transiciones y resúmenes, nunca cada muestra,
  heartbeat, sondeo o render. Dependencias de terceros se filtran a `warn` por defecto.
- Precedencia: selección temporal explícita de diagnóstico en Ajustes, después variable
  del entorno correspondiente, después default. La selección temporal no se persiste y
  se restablece al reiniciar. loglevel NO guarda niveles en localStorage/cookies:
  configurar el nivel sin persistencia para respetar esta precedencia.
- La UI muestra nivel efectivo y avisa cuando se activa `debug`/`trace`. Cambiarlo no permite
  desactivar saneamiento ni límites. El backend aplica la selección a sus sinks y al puente
  del frontend; los eventos reenviados conservan origen y umbral del frontend, evitando
  que LOG_LEVEL anule silenciosamente VITE_LOG_LEVEL o la selección explícita.
- Las variables VITE se fijan al compilar; no prometer cambios en un cliente instalado
  editando `.env`. Rust lee LOG_LEVEL del entorno del proceso al arrancar; un `.env` no
  se carga automáticamente. Aplican las reglas y el adaptador del principio XII.

#### Contrato, privacidad y correlación

- Cada evento DEBE tener versión de esquema, timestamp UTC RFC 3339 con milisegundos,
  nivel, origen (`backend`/`frontend`/`helper`), módulo, código de evento estable y mensaje
  breve. Campos opcionales tipados: código NB-*, duración, estado y correlación de diagnóstico.
  Los códigos y nombres de campos no se traducen ni se construyen con entradas del peer.
- Nunca registrar secretos, tokens, contraseñas, claves privadas, cookies, cabeceras de
  autorización, códigos de emparejamiento o identificadores de sesión de autenticación.
  Tampoco registrar el `sessionId` real del benchmark: usar un `diagnosticId` aleatorio
  distinto, local y sin autoridad para autenticar, reconectar o iniciar operaciones.
  Su asociación con la sesión se mantiene solo en memoria y no se exporta como tabla.
- No registrar por defecto IP/MAC, hostname, nombre de usuario, rutas personales, huellas
  completas o abreviadas, línea de comandos cruda, XML, contenido de BD ni payloads de red.
  Usar alias efímeros de diagnóstico y campos permitidos, sin hash estable de datos personales.
  Los resultados guardados y los informes explícitos tienen su propia política de datos;
  no se copian íntegros al log por comodidad.
- El saneamiento se aplica antes de TODOS los sinks, también consola de desarrollo,
  excepciones, stack traces y logs DEBUG/TRACE. Usar lista permitida de campos; omitir o
  redactar datos desconocidos, limitar tamaño/profundidad y escapar controles/saltos para
  impedir inyección de entradas falsas. Nunca serializar arbitrariamente objetos de error.

#### Formato legible y almacenamiento

- Desarrollo y depuración DEBEN ofrecer salida legible para humanos. Al renderizar logs
  para inspección, fechas y horas DEBEN usar formato español, reloj de 24 horas y zona
  `Europe/Madrid`, incluyendo milisegundos y offset UTC para desambiguar el cambio horario.
  Ejemplo: `21/09/2026 14:05:06.123 +02:00 [INFO] backend session.started ...`.
- El formateador DEBE aplicar reglas reales de horario de verano, no un offset fijo,
  y ser independiente de la zona/locale del equipo. TypeScript puede usar Intl con zona
  explícita; Rust debe usar soporte de zona comprobado y versionado antes de incorporarlo.
- `LOG_FORMAT` y `VITE_LOG_FORMAT` controlan la salida de consola de sus adaptadores,
  valores `pretty` o `json`; default `pretty` en desarrollo y `json` en producción.
  La app empaquetada no abre una consola. El flujo de depuración DEBE permitir renderizar
  el registro estructurado con el mismo formato legible, sin exigir leer timestamps crudos.
- Los archivos persistidos DEBEN ser JSON Lines UTF-8, una entrada por evento, con UTC
  canónico. El formateo humano no altera los datos ni el orden original. Duraciones se
  miden con reloj monotónico; timestamps de equipos distintos no prueban orden causal.
- Rust centraliza la escritura en la ruta y rotación del principio IV (10 archivos,
  50 MB totales). Frontend reenvía eventos saneados por un comando Tauri específico,
  con tamaño y frecuencia acotados; backend vuelve a validar. No hay destinos remotos.
  Capturar errores globales y promesas rechazadas mediante el wrapper, sin parchear
  indiscriminadamente console ni crear bucles al fallar el propio transporte de logging.

#### Coste, fallos y verificación

- Filtrar por nivel antes de construir mensajes costosos; usar buffers acotados y escritura
  fuera del camino crítico del benchmark. Priorizar warn/error ante saturación, descartar
  detalle primero y emitir un resumen acotado de pérdidas cuando sea posible.
  Ninguna cola infinita o escritura síncrona por muestra; aplica el presupuesto de CPU.
- Fallos de disco, permisos o transporte del logger NO deben bloquear cancelación ni tirar
  el benchmark. Usar fallback seguro y aviso visible de diagnóstico no disponible, sin
  recursión. El vaciado al cerrar tiene plazo acotado; no prometer persistencia tras un crash.
- CI DEBE impedir imports/llamadas que eludan el wrapper y probar filtrado, precedencia,
  captura frontend, ausencia de duplicados/bucles, redacción anidada, rotación, buffers llenos,
  fallos del sink y formato Madrid en invierno/verano y durante el cambio de hora.
  Toda feature nueva que emita logs queda sujeta a estas mismas reglas.

## Restricciones técnicas y versiones

### Plataforma y alcance

Propuesta Q1: Windows 10 1809 (build 17763) o posterior y Windows 11, solo x64.
Target `x86_64-pc-windows-msvc`, edición Rust 2024. El equipo de desarrollo puede requerir
Windows más reciente que el cliente; son matrices distintas. No se promete soporte de
Microsoft para todas las versiones Windows admitidas por la app.

Sin cloud, servicio sin sesión, portable, ARM64, Linux/macOS, otros motores, speedtest Internet,
monitorización continua o argumentos libres en v1. No implementar anticipadamente extensiones.

### Línea base propuesta, consultada el 2026-09-21

Se verificó existencia de versiones y restricciones declaradas. **No se ha compilado ni
validado el conjunto en Windows 10 1809.** El primer plan DEBE resolver dependencias,
compilar, empaquetar y comprobar arranque antes de consolidarlo. Conflictos requieren
enmienda explícita, no downgrade silencioso.

| Componente | Versión fijada | Uso / restricción |
|---|---|---|
| Rust, Cargo, rustfmt, Clippy, LLVM tools | Toolchain 1.98.1 | Misma distribución en rust-toolchain.toml |
| Node.js | 24.21.0 LTS | Solo desarrollo/build |
| pnpm | 12.5.1 | packageManager exacto |
| TypeScript | 6.0.3 | strict, noUncheckedIndexedAccess; no TS 7 |
| Zod | 4.6.5 | Validación runtime y parsing de contratos/configuración TypeScript |
| Svelte | 5.57.1 | Sin SvelteKit/SSR |
| Vite / @sveltejs/vite-plugin-svelte | 8.3.0 / 7.3.0 | Frontend local estático |
| tailwindcss / @tailwindcss/vite | 4.3.3 / 4.3.3 | Estilos obligatorios; plugin compatible declarado con Vite 8 |
| @types/node | 24.13.6 | Herramientas de build |
| tauri / tauri-build | 2.11.6 / 2.6.3 | Runtime/build Rust |
| @tauri-apps/api / @tauri-apps/cli | 2.11.1 / 2.11.5 | IPC / build y empaquetado |
| tauri-plugin-updater / @tauri-apps/plugin-updater | 2.12.0 / 2.12.0 | H3 |
| tokio / rustls / tokio-rustls | 1.53.1 / 0.23.45 / 0.26.5 | Async/TLS; fijar features y proveedor |
| rcgen / sha2 / uuid | 0.14.10 / 0.11.0 / 1.26.1 | Certificados, hash, UUIDv4 |
| windows / webview2-com | 0.61.3 / 0.38.2 | Familia usada por Tauri 2.11.6 |
| rusqlite / libsqlite3-sys | 0.40.2 / 0.38.2 | Feature bundled |
| SQLite embebido | 3.53.2 | Comprobado en sqlite3.h de libsqlite3-sys |
| serde / serde_json | 1.0.229 / 1.0.151 | Serialización |
| quick-xml / mdns-sd | 0.42.0 / 0.21.3 | Parser / descubrimiento |
| tracing / tracing-subscriber | 0.1.44 / 0.3.23 | Logs |
| loglevel | 1.9.2 | Frontend; solo accesible mediante el wrapper propio |
| csv / unicode-normalization | 1.4.0 / 0.1.25 | Exportación / NFC |
| Microsoft NTTTCP Windows | 5.40 x64 | Único motor v1 |
| NSIS / nsis-tauri-utils | 3.11 / 0.5.3 | Distribuidos por Tauri CLI 2.11.5 |
| Visual Studio Build Tools 2022 | 17.14.41, build 17.14.37710.0 | Bootstrapper fijo; C++ x64 |
| MSVC | v143, familia 14.44 | Revisión exacta del layout anterior; registrar cl /Bv |
| Windows SDK | 10.0.26100.9169 | SDK de build, no SO mínimo |
| svelte-check | 4.7.6 | Cero avisos/errores |
| ESLint / typescript-eslint / eslint-plugin-svelte | 10.11.0 / 8.70.0 / 3.23.0 | TS 6 dentro del rango admitido |
| Prettier / prettier-plugin-svelte | 3.9.8 / 4.1.1 | Formato |
| Vitest / @vitest/coverage-v8 | 5.0.1 / 5.0.1 | Versiones idénticas |
| @testing-library/svelte / jsdom | 5.4.2 / 30.1.0 | Componentes/DOM |
| @playwright/test / axe-core | 1.63.0 / 4.13.0 | Pruebas UI/accesibilidad |
| cargo-llvm-cov | 0.9.1 | Cobertura Rust |
| cargo-about / cargo-deny / license-checker | 0.9.2 / 0.20.2 / 25.0.1 | Licencias/inventario/dependencias |
| WCAG / TLS / mDNS / DNS-SD | 2.2 AA / 1.3 / RFC 6762 / RFC 6763 | Estándares |
| Protocolo NetworkBench / SessionResult | 1 / schemaVersion 1 | Independientes de app y esquema BD |

NTTTCP x64 5.40, SHA-256:
`f66561d09af91305412fd60ca4b28d57c7b650035d3c1edcc00a57b079e2247e`.
Calculado sobre el artefacto oficial descargado sin ejecutarlo. Comprobar de nuevo al
incorporarlo; `engine/VERSION` contiene `5.40`.

No añadir librería de gráficas/iconos: SVG y componentes existentes, versionados con el commit.
PDF mediante PrintToPdf de WebView2 y plantilla local.
Fuentes propuestas: familias Windows de Design con fallback local; registrar versiones en QA.
Para estas fuentes se sustituye la obligación genérica de embeber tipografía de §16.14.
Toda fuente adicional debe empaquetarse con versión, hash y licencia; nunca cargarse de Internet.

**Excepciones al pin estático:** WebView2 Evergreen se actualiza fuera de la app;
NO declararlo congelado a una versión. Cada release registra versión/hash del instalador offline,
mínima de runtime realmente probada y versiones exactas usadas en QA. Configurar y comprobar
esa mínima antes del funcionamiento. Instalación sin red aunque el runtime se actualice después.
Windows, fuentes del sistema y runner alojado también varían: registrar sus builds exactas.

Acciones GitHub, plugins adicionales y herramientas aún no necesarias NO quedan autorizados
como «última versión»: antes de incorporarlos, fijar versión/SHA y actualizar esta base cuando
sean dependencias directas. Spec Kit local es el material existente en .specify/ y .agents/;
fijar su procedencia por commit/hash al versionarlo. No asumir que corresponde a la última
release remota ni actualizarlo durante esta redacción.

### Fuentes de versiones y estándares

- [Registro npm](https://registry.npmjs.org/) y [crates.io](https://crates.io/):
  metadatos de cada paquete y archivos de las versiones publicadas.
- [Toolchain Rust](https://static.rust-lang.org/dist/channel-rust-stable.toml) y
  [Node.js 24.21.0](https://nodejs.org/dist/v24.21.0/).
- [NTTTCP 5.40](https://github.com/microsoft/ntttcp/releases/tag/v5.40).
- [NSIS fijado por Tauri CLI](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.5/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs).
- [Instalador Tauri](https://tauri.app/distribute/windows-installer/) y
  [Updater](https://v2.tauri.app/plugin/updater/).
- [Tailwind con Vite](https://tailwindcss.com/docs/installation/using-vite) y
  [variables de tema](https://tailwindcss.com/docs/theme).
- [svelte-check](https://svelte.dev/docs/cli/sv-check),
  [validación e inferencia Zod](https://zod.dev/basics) y
  [entorno y modos Vite](https://vite.dev/guide/env-and-mode).
- [tracing](https://docs.rs/tracing/0.1.44/tracing/) y
  [loglevel](https://github.com/pimterry/loglevel): adaptadores de logging backend/frontend.
- [WebView2 Evergreen](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/evergreen-vs-fixed-version).
- [Build Tools](https://learn.microsoft.com/en-us/visualstudio/releases/2022/release-history)
  y [Windows SDK](https://learn.microsoft.com/en-us/windows/apps/windows-sdk/downloads).
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) y [Guía WCAG2ICT](https://www.w3.org/TR/wcag2ict/).

## Flujo de desarrollo y controles de calidad

### Planificación y revisión

Cada spec.md, plan.md y tasks.md identifica la constitución utilizada.
Constitution Check cubre arquitectura, versiones, seguridad, datos, accesibilidad, rendimiento,
pruebas y distribución. Un incumplimiento exige modificar el plan o tramitar enmienda.

Cada PR incluye objetivo, requisitos, pruebas y efectos relevantes sobre contratos,
migraciones, privacidad y rendimiento. El mantenedor puede revisar en un proyecto individual
dejando evidencia; no se exige un equipo ficticio. Cambiar protocolo/esquema/reglas requiere
fixtures y compatibilidad explícita.

### CI y Definition of Done

En push/PR y de nuevo sobre la etiqueta de release DEBEN pasar:

1. Instalación frozen/locked y comprobación de toolchains, versiones, hash del motor,
   licencias, secretos y dependencias.
2. Formato, ESLint, `pnpm check` sin errores ni avisos, cargo fmt --check y Clippy con warnings como
   errores; claves es/en, tokens, reglas mecanizables de apariencia y uso exclusivo del logger.
3. Solo después de `pnpm check` correcto: unitarios, integración y cobertura con umbrales
   independientes del principio VIII; incluir contratos runtime, entorno y política de logs.
4. Build frontend y Rust release x64; versiones sincronizadas entre package.json, Tauri
   y Cargo. package.json es la fuente de la versión de la app.
5. Informes disponibles al revisor; tests fallidos, omitidos o intermitentes no cuentan
   como aprobados. Reintentar no oculta la primera incidencia.

Los E2E dependen del check correcto y del build necesario. Ningún job paralelo puede
eludir esa dependencia. Si aún no existe `pnpm check`, crearlo es requisito previo a cerrar
la primera implementación afectada; su ausencia no se registra como validación aprobada.

Cerrar un hito exige Historias.md §26 y pruebas reales aplicables. Publicar v1 exige dos equipos
Windows, TCP/UDP, IPv4/IPv6, LAN/routing/VPN según alcance, UAC aceptado/rechazado,
instalación/actualización/desinstalación, migración, caída/cancelación, revisión manual
de accesibilidad y V-01 a V-12 cerradas en VALIDACION.md.
Matriz mínima propuesta: Windows 10 1809 y 22H2, Windows 11; DPI 100/150/200 %,
monitor desconectado y GPU integrada. Registrar build exacta de cada imagen.

La release genera networkbench-setup.exe, firma y latest.json del mismo binario.
El manifiesto apunta a `releases/download/vX.Y.Z/networkbench-setup.exe`,
no a un binario mutable bajo latest. El enlace público estable
`releases/latest/download/networkbench-setup.exe` se conserva para descarga humana estable.

Las versiones 0.x son pre-releases: pruebas con URL explícita por etiqueta.
No exigir que releases/latest las exponga. Canal de producción solo consume estables;
un canal preview automático requiere endpoint separado y elección explícita, con contrato
fijado antes de implementarlo. Un 404 antes de la primera estable no impide usar el benchmark.

### Discrepancias resueltas por esta propuesta

Trasladar estas decisiones a los artefactos afectados en la siguiente planificación.
En esta redacción solo se modifica la constitución.

| Conflicto | Regla propuesta |
|---|---|
| Bootstrapper online §3.1 / offline §3.3 | WebView2 offline |
| Persistencia H1 / historial H3 | Escritura H1; pantalla y gestión H3 |
| 64 streams y segundo sentido en base+32 | Simultáneo máximo 32 por sentido; secuencial hasta 64; validar todos los puertos |
| Helper «solo rangos de 64» / control y mDNS | Lista blanca separa control, 5353 y bloque de datos de 64 |
| Sondeo de la app / regla por programa NTTTCP | El sondeo no demuestra permiso para el otro ejecutable; validar tráfico real |
| Diálogo combinado / REQUEST posterior | Plan disponible antes de aceptar conjuntamente; G2 fija contrato |
| Instancia única / integración de dos instancias | Harness aislado, sin bypass de producción |
| Terminar procesos por nombre/ruta | Demostrar propiedad de la instancia |
| latest y pre-release / URL mutable | URL por etiqueta para pruebas y binarios; latest solo estable |
| Accesibilidad H2 / zoom bloqueado y máximo 130 % | Accesibilidad desde H1 y ampliación propia al 200 % |
| Design Sistema / Historias Oscuro | Oscuro inicial, sujeto a Q4 |
| Tipografía embebida / fuentes Windows de Design | Fuentes del SO locales; adicionales empaquetadas y licenciadas |
| «Todo local» / updater y Evergreen | Funcionalidad local y actualizaciones externas documentadas |
| u64 / números JavaScript | Codificación sin pérdida en contratos |
| Sin métricas / verdict solo ok-warn-problem | Estado no evaluable antes de congelar schemaVersion 1 |
| Historias §22: sessionId, IP, huella y comandos en logs | Principio XIII: diagnosticId independiente y campos mínimos saneados; adaptar §22 al planificar |
| Historias §22: reenvío de console.error | Wrapper y captura global controlada; sin console directo en código de funcionalidad |

### Puertas técnicas antes de implementar lo afectado

El responsable de implementación resuelve con evidencia en el plan, sin trasladar al usuario
preguntas que requieren investigación técnica:

- **G1 — Motor:** V-01/V-02/V-03 en NTTTCP real: puertos y canales auxiliares, flags,
  readiness TCP/UDP, XML, métricas y control de tasa. Sin limitador, variar streams NO
  se presenta como tasa fija. Capacidad inviable exige revisar alcance, no fingir cumplimiento.
- **G2 — Protocolo:** orden HELLO/plan/emparejamiento/aceptación, validación provisional
  de certificados nuevos, serialización del código/huella, mensajes de firewall,
  heartbeat sin eco infinito, reconexión y ACK idempotentes. Diagrama y contratos probados.
- **G3 — Resultados:** reconciliar frame de 1 MiB con sesiones largas, muestras y XML;
  segmentar con límites totales o ajustar payload respetando lo exigido. Fijar temporización,
  huecos y conciliación de resultados locales; no prometer igualdad tras intercambio fallido.
- **G4 — Interpretación:** fronteras inclusivas/exclusivas, percentiles, ceros, NIC y cohortes
  comparables de historial (peer, adaptadores, protocolo, direcciones y parámetros).
  Tablas de casos antes de implementar reglas.
- **G5 — Entorno:** resolver lockfiles y probar toolchain/instalador/runtime en Windows mínimo.
  Registrar MSVC, WebView2, Windows y acciones CI exactas. Versiones consultadas no equivalen
  a compatibilidad demostrada.
- **G6 — Requisitos:** recuperar o reemplazar referencias al antiguo Historias.md §§29–54
  y Especificacion.md ausente; reconciliar Design. No cerrar requisitos de contenido desconocido.

## Governance

### Autoridad y enmiendas

Tras ratificarla, esta constitución prevalece sobre especificaciones, planes, tareas,
maquetas y código. Historias.md define producto dentro de este marco; Design define
presentación donde no contradiga producto, seguridad o accesibilidad.
Una instrucción explícita posterior del propietario puede enmendar el marco con registro.

Propietario: Daniel Diez Mardomingo. Enmiendas: documentar motivo, reglas, alternativas,
compatibilidad/migración y pruebas; obtener su decisión; actualizar versión, fecha y
artefactos derivados antes de implementar desviaciones. Sin excepciones tácitas por plazo,
comodidad, herramienta o código heredado.

Semver: MAJOR para retirada/redefinición incompatible; MINOR para obligaciones compatibles
adicionales; PATCH para aclaraciones sin cambio normativo y parches tecnológicos que preserven
contratos. Un cambio incompatible no es PATCH por el número que use el proveedor.
Conservar motivos/evidencia en Git.

### Decisiones pendientes de respuesta

- **Q1:** conservar Windows 10 1809+ y Windows 11 x64 (propuesta) o reducir matriz.
- **Q2:** 80 % general por capa y 90 % de líneas en módulos críticos Rust (propuesta).
- **Q3:** adoptar offline, persistencia H1, máximo simultáneo de 32 streams por sentido,
  actualización por versión y accesibilidad desde H1 (propuesta).
  Mantener 64 simultáneos exige rediseñar rangos y reglas.
- **Q4:** Graphite Violet con Oscuro inicial (propuesta) o Sistema inicial.

TODO(RATIFICATION_DATE): registrar fecha de adopción cuando el propietario resuelva Q1-Q4.
Esta 0.4.0 conserva propuestas revisables; Tailwind, la referencia a Design, la validación
transversal, el control Svelte y las políticas de entorno y logging están solicitados expresamente
por el propietario. Primera versión ratificada: 1.0.0.
Ratificar el marco no equivale a aprobar builds todavía no probadas.

**Version**: 0.4.0 | **Ratified**: pendiente — TODO(RATIFICATION_DATE) | **Last Amended**: 2026-09-21
