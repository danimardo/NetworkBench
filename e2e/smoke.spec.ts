import { test, expect } from "@playwright/test";

/**
 * Humo real (T147): sin backend Tauri detrás (no hay `window.__TAURI__` en un navegador
 * normal), así que cada comando IPC falla y se registra por el logger — eso ya lo verifica
 * `transport.ts` en Vitest. Lo que este fichero comprueba es lo que Vitest no puede: que la
 * aplicación entera arranca en un navegador real, que las tres rutas montadas por
 * `App.svelte` (T181) renderizan sin quedarse en blanco ni lanzar una excepción no
 * capturada, y que se navega entre ellas con la barra lateral.
 */

test("la aplicación arranca y muestra la pantalla de Inicio", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle("NetworkBench");
  await expect(page.getByRole("heading", { name: "Equipos disponibles" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Navegación principal" })).toBeVisible();
});

test("la barra lateral navega a Historial y Ajustes sin romper el renderizado", async ({
  page,
}) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Historial" }).click();
  await expect(page.getByRole("heading", { name: "Historial de pruebas" })).toBeVisible();

  await page.getByRole("button", { name: "Ajustes" }).click();
  await expect(page.getByRole("tablist")).toBeVisible();

  await page.getByRole("button", { name: "Inicio" }).click();
  await expect(page.getByRole("heading", { name: "Equipos disponibles" })).toBeVisible();
});

test("el diálogo de conexión manual se abre y se cierra sin dejar la app inutilizable", async ({
  page,
}) => {
  await page.goto("/");

  await page.getByTestId("manual-connect-btn").click();
  await expect(page.getByRole("dialog")).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect(page.getByTestId("manual-connect-btn")).toBeFocused();
});

test("una llamada IPC sin backend se registra como error visible, no como una página rota", async ({
  page,
}) => {
  await page.goto("/");
  // FR-016/T181: sin `window.__TAURI__`, `peers_list` falla y `PeersModel` lo refleja en un
  // aviso cerrable en vez de dejar la pantalla a medio renderizar o sin reaccionar.
  await expect(page.getByTestId("peers-error")).toBeVisible();
  await page.getByTestId("peers-error").getByRole("button", { name: "Cerrar" }).click();
  await expect(page.getByTestId("peers-error")).toBeHidden();
});
