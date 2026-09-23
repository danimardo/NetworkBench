import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    conditions: ["browser"],
  },
  test: {
    globals: true,
    environment: "jsdom",
    include: ["src/**/*.{test,spec}.ts", "tests/**/*.{test,spec}.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      reportsDirectory: "./artifacts/coverage/frontend",
      exclude: ["node_modules", "tests", "**/*.test.ts"],
      // Q2 (constitución 0.7.0): 80 % en cada métrica, sin promediarlas entre sí.
      // Hoy la baseline está por debajo en las cuatro y este gate falla a propósito:
      // ningún porcentaje inferior se interpreta como aprobación. Medido el 2026-09-23:
      // sentencias 67,40 · ramas 49,46 · funciones 68,96 · líneas 68,06.
      thresholds: {
        statements: 80,
        branches: 80,
        functions: 80,
        lines: 80,
      },
    },
  },
});
