# NetworkBench

> Aplicación de escritorio moderna y local para Windows que mide el rendimiento real de red entre dos equipos usando Microsoft NTTTCP como motor.

NetworkBench combina un frontend fluido y accesible en Svelte 5 y TypeScript con un backend de alto rendimiento en Rust sobre Tauri 2, renderizado en Microsoft Edge WebView2 Evergreen. Todo el procesamiento y almacenamiento es 100 % local: sin servidores centrales, sin nube, sin cuentas y con telemetría cero.

---

## Características Principales

- **Motor Estándar de Referencia**: Integra y controla Microsoft NTTTCP v5.40 (x64) mediante Windows Job Objects para una ejecución determinista y sin procesos huérfanos.
- **Emparejamiento Simétrico Seguro**: Negociación cifrada y autenticada entre dos peers con identidades Ed25519, huellas SHA-256 y código simétrico de 6 dígitos con expiración.
- **Pruebas Completas TCP y UDP**:
  - Pruebas TCP secuenciales (ida / vuelta) y simultáneas bidireccionales (`RunningBoth`) con reserva disjunta de puertos.
  - Pruebas UDP con métricas de datagramas recibidos, pérdida de paquetes y tasa efectiva calculada.
- **Diagnóstico y Veredicto Automático**:
  - Clasificación en tres niveles de salud de red: *Excelente*, *Bueno con observaciones* o *Problemas detectados*.
  - Hechos objetivos, posibles causas y acciones recomendadas con explicaciones en lenguaje claro.
  - Reglas normativas centralizadas e inmutables calibradas en `thresholds.json`.
- **Historial Local Transaccional**: Base de datos SQLite local en modo WAL con consultas paginadas, comparativa de cohortes idénticas, tendencias históricas y borrado atómico con confirmación.
- **Exportación Multi-Formato**: Generación de informes en PDF (renderizado vectorial A4), JSON (unidades base para automatización) y CSV estructurado (con BOM UTF-8 y protección contra inyección de fórmulas).
- **Accesibilidad y Personalización**: Conforme a WCAG 2.1 AA (navegación completa por teclado, regiones en vivo ARIA, alto contraste, tema oscuro por defecto y modo reducción de movimiento).

---

## Requisitos del Sistema

- **Sistema Operativo**: Windows 11 (build 22000+) o Windows 10 22H2 (x64).
- **Runtime**: Microsoft Edge WebView2 Runtime (versión Evergreen incluida por defecto en Windows 10/11).
- **Permisos**: Ejecución en modo usuario sin elevación forzada para control y pruebas estándar. Un helper UAC opcional y acotado permite configurar reglas de firewall si se requiere.

---

## Documentación

- [Guía de Usuario](file:///docs/user/guide.md): Instrucciones de uso, emparejamiento, configuración de pruebas e interpretación de veredictos.
- [Guía de Desarrollo](file:///docs/development/setup.md): Configuración del entorno, arquitectura del proyecto y comandos de verificación.
- [Registro de Validación Empírica](file:///VALIDACION.md): Evidencias de pruebas de integración, arneses Windows y matriz V-01 a V-12.
- [Historias y Requisitos del Producto](file:///Historias.md): Catálogo exhaustivo de requisitos y criterios de aceptación.

---

## Licencia

Distribuido bajo licencia MIT. Consulte `THIRD_PARTY_NOTICES.md` para ver las licencias de dependencias y componentes de terceros.
