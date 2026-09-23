# Third-Party Notices

NetworkBench incorporates or interacts with third-party software components.
This document contains the notices, licenses, and acknowledgments for those components.

---

## 1. Microsoft NTTTCP for Windows (v5.40 x64)

- **Vendor**: Microsoft Corporation
- **License**: Microsoft Software License Terms (included in `engine/LICENSE`)
- **Official SHA-256**: `f66561d09af91305412fd60ca4b28d57c7b650035d3c1edcc00a57b079e2247e`
- **Notice**: NTTTCP is executed as a standalone subprocess engine. NetworkBench does not link statically or dynamically against NTTTCP source code.

---

## 2. Rust Dependencies

NetworkBench Rust backend uses open-source libraries licensed under MIT or Apache-2.0 licenses:

- **Tauri** (`tauri`, `tauri-build`, `tauri-plugin-updater`) — MIT / Apache-2.0
- **Tokio** (`tokio`, `tokio-rustls`, `tokio-util`) — MIT
- **Rustls** (`rustls`, `rustls-pki-types`) — Apache-2.0 / ISC / MIT
- **Rusqlite** (`rusqlite`, `libsqlite3-sys`) — MIT
- **Serde** (`serde`, `serde_json`) — MIT / Apache-2.0
- **Quick-xml** (`quick-xml`) — MIT
- **Tracing** (`tracing`, `tracing-subscriber`) — MIT
- **rcgen** (`rcgen`) — MIT / Apache-2.0
- **sha2** (`sha2`) — MIT / Apache-2.0
- **uuid** (`uuid`) — Apache-2.0 / MIT
- **mdns-sd** (`mdns-sd`) — MIT / Apache-2.0
- **windows** (`windows`, `windows-core`, `windows-sys`) — MIT / Apache-2.0
- **webview2-com** (`webview2-com`) — MIT / Apache-2.0
- **csv** (`csv`) — Unlicense / MIT
- **unicode-normalization** (`unicode-normalization`) — MIT / Apache-2.0

---

## 3. Frontend & Build Dependencies

NetworkBench frontend uses open-source libraries:

- **Svelte** (`svelte`, `@sveltejs/vite-plugin-svelte`) — MIT
- **Vite** (`vite`) — MIT
- **Tailwind CSS** (`tailwindcss`, `@tailwindcss/vite`) — MIT
- **Zod** (`zod`) — MIT
- **TypeScript** (`typescript`, `typescript-eslint`) — Apache-2.0 / MIT
- **ESLint** (`eslint`, `eslint-plugin-svelte`) — MIT
- **Prettier** (`prettier`, `prettier-plugin-svelte`) — MIT
- **Vitest** (`vitest`, `@vitest/coverage-v8`) — MIT
- **Playwright** (`@playwright/test`, `axe-core`) — Apache-2.0 / MPL-2.0
- **loglevel** (`loglevel`) — MIT
