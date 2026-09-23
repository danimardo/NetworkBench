# Guía de Entorno y Desarrollo: NetworkBench

Esta guía detalla los requisitos, convenciones y comandos canónicos para desarrollar y validar NetworkBench.

---

## 1. Requisitos de la Estación de Desarrollo

- **Sistema Operativo**: Windows 11 x64 o Windows 10 22H2 x64.
- **Node.js**: v20+ LTS o v24+ (con `pnpm 10+`).
- **Rust**: 1.84+ stable con target `x86_64-pc-windows-msvc`.
- **Compilador C++**: Visual Studio 2022 con herramientas C++ MSVC x86/x64 y Windows 10/11 SDK.
- **Microsoft Edge WebView2**: Runtime Evergreen (instalado por defecto en Windows 10/11).

---

## 2. Configuración Inicial del Repositorio

Clonar el repositorio e inicializar dependencias y hooks de Git:

```powershell
# Instalar dependencias de Node
pnpm install --frozen-lockfile

# Configurar hook de Git obligatorio del proyecto
git config core.hooksPath scripts/git-hooks
```

---

## 3. Estructura y Arquitectura del Código

El proyecto sigue una estricta separación de responsabilidades y fronteras arquitectónicas (`ARCHITECTURE.md`):

- **Frontend (`src/`)**:
  - Implementado en **Svelte 5** (utilizando runes `$state`, `$derived`, `$props`) y **TypeScript**.
  - Estructurado en features autocontenidas: `features/peers/`, `features/session/`, `features/results/`, `features/history/`, `features/settings/`, `features/export/`.
  - Prohibidos los imports profundos entre features. La comunicación se realiza mediante APIs públicas (`index.ts`) y servicios transversales en `src/lib/`.
- **Backend (`src-tauri/`)**:
  - Implementado en **Rust** sobre Tauri 2.
  - Módulos canónicos:
    - `control/`: Coordinador de sesión, protocolo peer-to-peer y máquina de estados.
    - `engine/`: Integración con NTTTCP mediante Windows Job Objects y parsers XML.
    - `diagnostic/`: Motor de evaluación puro e inmutable basado en `thresholds.json`.
    - `history/`: Persistencia SQLite en modo WAL con migraciones idempotentes.
    - `export/`: Motor de exportación JSON, CSV y PDF (WebView2 `PrintToPdf`).
    - `firewall/`: Inspección de reglas WFP sin elevación y helper de configuración UAC.
    - `updater/`: Gestor de actualizaciones firmado con minisign y URLs inmutables.

---

## 4. Comandos Canónicos de Verificación

Para mantener la integridad continua del repositorio, ejecute los comandos estándar:

```powershell
# Chequeo estático de tipos Svelte y TypeScript
pnpm check

# Comprobación de linters y formato de código
pnpm lint
pnpm format:check

# Verificación de arquitectura y fronteras de imports
pnpm check:architecture

# Pruebas unitarias de frontend
pnpm test:unit

# Pruebas unitarias y de integración de backend Rust
pnpm test:rust

# Pipeline canónico unificado (ejecuta todo lo anterior)
pnpm verify
```

---

## 5. Arneses de Pruebas en Windows

Ubicados en `tests/windows/`, validan la integración profunda con la plataforma Windows sin dependencias externas:

- `tests/windows/e2e/e2e_two_peers_harness.ps1`: Ciclo E2E completo entre dos peers.
- `tests/windows/firewall/firewall_harness.ps1`: Inspección de reglas del Firewall de Windows.
- `tests/windows/ntttcp/ntttcp_harness.ps1`: Ejecución controlada y parsing XML de NTTTCP.
- `tests/windows/window/window_harness.ps1`: Geometría de ventana, Snap Assist y soporte multi-monitor.
- `tests/windows/export_pdf/pdf_harness.ps1`: Validación de renderizado e integridad de PDFs.
