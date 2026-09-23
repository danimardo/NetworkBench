# Guía rápida de validación: NetworkBench v1

**Fecha**: 2026-09-21  
**Constitución**: 0.7.0, no ratificada

Esta guía define cómo validar el producto a medida que exista. Hoy la aplicación, los manifiestos,
los scripts de producto y CI están **NO PRESENTES**. Los comandos futuros se identifican como
contrato del plan; no deben ejecutarse ni declararse aprobados hasta que L00 los cree y compruebe.

## 1. Estado actual comprobable

Desde la raíz del repositorio:

```powershell
node scripts/agent/verify.mjs
node Design/scripts/verify-tokens.mjs
```

El primero valida el sistema de instrucciones; el segundo, la regla actual de colores del sistema
de diseño. Ninguno compila ni prueba NetworkBench.

## 2. Prerrequisitos para L00

Antes del primer build, registrar evidencia de:

- Windows 10 22H2 y Windows 11 x64 con builds exactas disponibles;
- toolchain Rust/MSVC/Windows SDK, Node y pnpm de la línea base o una enmienda aprobada;
- WebView2 offline exacto y versión mínima realmente probada;
- NTTTCP 5.40 x64 con versión, licencia y SHA-256 recalculado;
- ADR-001/ADR-002 aceptados y Q2 resuelta para el gate de cobertura;
- resolución G5 con manifiestos/lockfiles y primer arranque instalado;
- dos perfiles/instancias aislables y, para los cierres nativos, dos equipos o VMs adecuados.

No usar credenciales, redes o historiales reales en fixtures. Reservar puertos, carpetas, BD,
identidades y reglas de firewall por worker/ejecución.

## 3. Contrato de comandos después de L00

Estos comandos son **planificados; NO PRESENTES hoy**:

```powershell
pnpm install --frozen-lockfile
pnpm check
pnpm verify
pnpm build
pnpm test:unit
pnpm test:integration
pnpm test:ui
pnpm test:windows
```

L00 debe crear una única definición canónica y documentar qué agrega cada script. `pnpm verify`
debe ordenar checks estáticos antes de builds/tests, fallar ante cualquier subcomando fallido y no
marcar pruebas Windows/laboratorio como aprobadas cuando el entorno no exista.

## 4. Checkpoint estático y de contratos

**Propósito**: demostrar que el grafo, tipos y contratos básicos son coherentes.

**Ejecución futura**:

```powershell
pnpm check
pnpm verify
```

**Evidencia esperada**:

- formato, ESLint y `svelte-check` sin errores/avisos;
- `cargo fmt --check` y Clippy con warnings como errores;
- imports respetan `ARCHITECTURE.md` y no existen imports profundos entre features;
- claves es/en equivalentes, tokens y logger único;
- fixtures IPC/protocolo/resultado aceptados/rechazados igual por Zod y Rust;
- builds frontend/Rust release x64;
- cobertura medida por lenguaje; gate aplicado solo tras decisión Q2.

## 5. H1 — Prueba estándar bidireccional

**Prerrequisitos**: L00–L04; G1–G3 cerradas para el escenario; dos identidades y recursos aislados.

**Escenario**:

1. Instalar/abrir dos instancias limpias A y B.
2. Descubrir B y repetir por conexión manual IP/DNS.
3. Comparar el código, aceptar el plan estándar y confirmar que el plan visto es el ejecutado.
4. Ejecutar TCP A→B y B→A.
5. Comprobar el mismo `sessionId`, plan y cifras en ambos extremos, o degradación explícita.
6. Reiniciar ambas instancias y reabrir el resultado persistido.

**Resultado esperado**: recorrido completo en aproximadamente un minuto tras aceptación, resultados
coherentes, español/inglés disponibles y ningún término NTTTCP fuera de detalles/licencias.

**Variantes negativas**:

- código distinto, huella cambiada, rechazo, timeout y peer ocupado;
- plan/rango/estado inválido o mensaje duplicado;
- DNS inválido, IPv4/IPv6 y peer entre subredes con routing;
- motor ausente/hash incorrecto/XML inválido;
- cerrar o matar la app durante ejecución.

**Evidencia necesaria**: tests unitarios/contrato, integración de dos instancias y recorrido Windows
con NTTTCP real. Loopback no prueba LAN, firewall ni dos NIC reales.

## 6. H1 — Cancelación y limpieza

Para cada estado no terminal y desde ambos extremos:

1. solicitar cancelación una vez y repetidamente;
2. cortar control y matar la app/proceso en variantes separadas;
3. observar que no se esperan más de 2 s por ACK;
4. comprobar estado terminal/origen, puertos, temporales y procesos propios;
5. comprobar parcial solo cuando una dirección tiene resultados de ambos extremos.

**Resultado esperado**: ningún proceso huérfano, ningún recurso ajeno afectado y ninguna sesión parcial
presentada como éxito. La verificación de Job Object necesita ejecución Windows real.

## 7. H1 — Persistencia y migración

Preparar archivos SQLite reales con WAL para esquema actual, anterior, futuro, corrupto y fallo
inyectado durante migración.

**Resultado esperado**:

- commit/rollback e idempotencia por `sessionId`;
- backup coherente antes de migrar y restauración ante fallo;
- original y backup preservados;
- esquema futuro rechazado sin escritura;
- resultado histórico conserva reglas/versiones y no se recalcula.

Una BD en memoria no satisface WAL, reinicio ni recuperación.

## 8. H2 — Diagnóstico, UI y rendimiento

Con fixtures controlados de capacidad conocida/desconocida, muestras insuficientes, huecos,
asimetría, retransmisiones, CPU y tráfico ajeno:

- verificar hechos, observaciones, posibles causas y acciones por separado;
- confirmar `notEvaluable` en datos insuficientes y exact-boundary cases de G4;
- comprobar teclado, foco, icono+texto, reducción de movimiento y texto/DPI hasta 200 %;
- ejecutar cinco mediciones release por escenario V-04 con entorno registrado y sin cobertura/traces;
- medir app + procesos WebView2 atribuibles: cada ejecución <5 % de un procesador lógico;
- confirmar eventos visuales ≤4 Hz y cancelación responsiva bajo carga visual/logging degradado.

V-12 repite con/sin materiales y GPU integrada. Una captura no prueba accesibilidad ni rendimiento.

## 9. H2 — Firewall y UAC

En un entorno Windows restaurable, cubrir regla ausente, modificada, deshabilitada, red pública y
política corporativa. Probar consentimiento/UAC aceptado y rechazado.

**Resultado esperado**: aplicación principal sin elevación; helper solo opera sobre reglas propias
allowlisted; programa+puerto+perfil mínimos; rechazo deja la aplicación utilizable; desinstalación
no borra recursos ajenos.

## 10. H3 — Historial y exportación

Crear sesiones sintéticas completas/incompletas con peers, adaptadores y planes comparables/no
comparables.

**Resultado esperado**:

- filtros, búsqueda, reapertura, borrado confirmado y cohortes correctas;
- comparación solo de las cinco sesiones compatibles más recientes;
- PDF real de WebView2 validado por V-09;
- JSON preciso y CSV es/en neutralizando fórmulas;
- anonimización elimina todos los canarios identificativos, incluidos bloques crudos y nombre.

## 11. H3 — UDP y planes avanzados

Tras V-01/V-02/V-03/V-05, ejecutar límites mínimo/máximo/ilegal y puertos solapados. Probar UDP
en dos equipos y plan simultáneo.

**Resultado esperado**: ambos extremos validan el mismo plan; máximo 64 streams según la spec; UDP
distingue objetivo, emisión real, recepción y pérdida; una aproximación se etiqueta como tal; no
se aplica asimetría secuencial a un plan simultáneo.

## 12. H3/L10 — Ciclo de vida y release

Con el artefacto exacto candidato:

- instalar sin Internet en cada SO admitido y primer arranque con WebView2 preparado;
- probar cierre, bandeja, autoarranque, monitor retirado, DPI y geometría;
- probar actualización auténtica, inválida, anterior, desactivada y durante sesión activa;
- desinstalar conservando datos por defecto y limpiando solo recursos propios;
- verificar licencias, hashes, SBOM/inventario y ausencia de vulnerabilidades bloqueantes;
- registrar V-01–V-12 en `VALIDACION.md` con hardware, versiones, comandos y artefactos.

La ruta exacta del manifiesto/updater permanece bloqueada hasta alinear la spec y la Constitución.
No publicar una etiqueta o release sin autorización Git/publicación específica.

## 13. Registro de evidencia

Cada ejecución conserva:

- commit/artefacto/hash y versión de contratos;
- comandos exactos y códigos de salida;
- Windows/WebView2/toolchains/hardware relevantes;
- suites ejecutadas, omitidas, fallidas o no verificables;
- fixtures/identidades sintéticas y limpieza realizada;
- logs/diagnósticos saneados y artefactos de fallo;
- Q/G/V cerradas y responsable de las pendientes.

Estados permitidos: `VERIFICADO`, `DOCUMENTADO`, `INFERIDO`, `NO VERIFICABLE`, `NO PRESENTE`.
Una prueba omitida o un reintento no transforma un fallo en aprobación.
