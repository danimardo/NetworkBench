import { defineConfig } from "@playwright/test";

const PORT = 4173;

export default defineConfig({
  testDir: "./e2e",
  timeout: 30000,
  fullyParallel: true,
  reporter: "list",
  use: {
    baseURL: `http://localhost:${PORT}`,
    trace: "on-first-retry",
  },
  // Se sirve el build de producción (`vite preview`), no `vite dev`: el dev server
  // compila cada módulo bajo demanda la primera vez que se pide, y la primera navegación
  // contra un proceso recién arrancado tardaba de forma intermitente más de 60 s — a veces
  // pocos segundos, otras veces mucho más — mientras `curl` al mismo servidor respondía en
  // 7-10 s de forma consistente (VALIDACION.md §1.32). `vite preview` sirve los assets ya
  // construidos: sin transformación bajo demanda, sin esa fuente de intermitencia.
  webServer: {
    command: `pnpm exec vite build && pnpm exec vite preview --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}`,
    reuseExistingServer: !process.env.CI,
    timeout: 60000,
  },
});
