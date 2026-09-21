# Versiones

**Ámbito:** este repositorio. **Estado:** VERIFICADO (herramientas locales) /
DOCUMENTADO (línea base). **Fecha:** 2026-09-21.

## La línea base manda

`.specify/memory/constitution.md` §«Línea base propuesta» fija ~50 componentes con versión
exacta. Es la referencia. Dice literalmente: **«No se ha compilado ni validado el conjunto
en Windows 10 1809.»** El primer plan debe resolver dependencias, compilar, empaquetar y
comprobar arranque antes de consolidarla (puerta **G5**).

Versiones consultadas en un registro **no equivalen a compatibilidad demostrada.**

## Las herramientas locales divergen de la línea base (VERIFICADO)

| Componente | Exigido | Instalado | Delta |
|---|---|---|---|
| Node.js | 24.21.0 LTS | **24.4.1** | inferior |
| pnpm | 12.5.1 | **11.6.0** | inferior, cambio mayor |
| Rust / Cargo | 1.98.1 | **1.94.0** | inferior |
| Git | — | 2.55.0 | sin exigencia |

`Historias.md` ya lo advierte: «Herramientas del equipo no equivalen a versiones del
proyecto». **No hagas nada al respecto por tu cuenta.**

NO VERIFICABLE aquí: Visual Studio Build Tools 2022 17.14.41, MSVC v143 14.44,
Windows SDK 10.0.26100.9169, WebView2 runtime, NTTTCP 5.40.

## Reglas

- **No instalar, actualizar ni hacer downgrade de nada** sin autorización específica.
  Eso incluye toolchains, paquetes, plugins y servidores MCP.
- **No hacer downgrade silencioso** para resolver un conflicto de versiones. Un conflicto
  con la línea base exige enmienda explícita de la constitución.
- **No declarar «última versión»** como versión válida. Acciones de CI, plugins y
  herramientas nuevas se incorporan con versión y SHA fijados, actualizando la línea base
  cuando sean dependencias directas.
- **Un cambio incompatible no es PATCH** por el número que use el proveedor.
- `package.json` será la fuente de la versión de la aplicación, sincronizada con Tauri y
  Cargo. La versión de la constitución es independiente de la de la aplicación.

## Excepciones al pin estático (constitución)

WebView2 Evergreen se actualiza fuera de la aplicación: **no declararlo congelado**.
Windows, las fuentes del sistema y el runner alojado también varían. Cada release registra
sus builds exactas. La instalación debe funcionar sin red aunque el runtime se actualice
después.

## Versiones del propio sistema de agentes

Registradas en `.agents/meta/agent-capabilities.yaml`. Al cambiar la versión de una
herramienta: **no regenerar automáticamente**. Comparar capacidades, identificar impacto,
presentar los cambios y pedir aprobación si requieren modificar algo. Procedimiento en
`.agents/skills/verificar-sistema-agentes/SKILL.md`.
