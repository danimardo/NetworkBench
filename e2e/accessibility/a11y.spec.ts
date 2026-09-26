import { test, expect } from "@playwright/test";
import { runAxe, describeViolations } from "../axe";

/**
 * Accesibilidad real con axe-core (T147, SC-008). Las tres suites `expect(true)` que
 * sustituye esta no comprobaban nada; los tres nombres de esta describe siguen literalmente
 * los que tenía el placeholder, pero ahora cada uno ejecuta lo que promete.
 */

const RUTAS: { boton: string; encabezado: string | RegExp }[] = [
  { boton: "Inicio", encabezado: "Equipos disponibles" },
  { boton: "Historial", encabezado: "Historial de pruebas" },
  { boton: "Ajustes", encabezado: /./ },
];

test.describe("Accesibilidad y Navegación por Teclado (T133)", () => {
  test("los roles ARIA y la estructura semántica cumplen estándares", async ({ page }) => {
    await page.goto("/");

    for (const ruta of RUTAS) {
      if (ruta.boton !== "Inicio") {
        await page.getByRole("button", { name: ruta.boton }).click();
      }
      const violaciones = await runAxe(page);
      expect(violaciones, describeViolations(violaciones)).toEqual([]);
    }
  });

  test("el foco inicial y atrapado de modales cumple WCAG 2.1 AA", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("manual-connect-btn").click();

    const dialogo = page.getByRole("dialog");
    await expect(dialogo).toBeVisible();

    // El foco entra al diálogo, no se queda en el botón que lo abrió.
    const enfocadoAlAbrir = await page.evaluate(() => document.activeElement?.tagName);
    expect(enfocadoAlAbrir).not.toBe("BODY");

    const focusables = await dialogo.locator(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    );
    const total = await focusables.count();
    expect(total).toBeGreaterThan(1);

    // Shift+Tab desde el primer elemento debe quedarse dentro del diálogo (atrapado), no
    // escapar hacia la barra lateral o la barra de título.
    await page.keyboard.press("Shift+Tab");
    const escapoDelDialogo = await page.evaluate((selectorDialogo) => {
      const activo = document.activeElement;
      const dialogo = document.querySelector(selectorDialogo);
      return !!activo && !!dialogo && !dialogo.contains(activo);
    }, '[role="dialog"]');
    expect(escapoDelDialogo).toBe(false);

    // El resto de la ventana queda `inert`: la barra lateral no es alcanzable.
    const sidebarInert = await page
      .getByRole("navigation", { name: "Navegación principal" })
      .evaluate((el) => el.closest("[inert]") !== null);
    expect(sidebarInert).toBe(true);

    await page.keyboard.press("Escape");
    await expect(dialogo).toBeHidden();
    await expect(page.getByTestId("manual-connect-btn")).toBeFocused();
  });

  test("la navegación por teclado del tablist de Ajustes opera con flechas, Home/End y Tab", async ({
    page,
  }) => {
    await page.goto("/");
    await page.getByRole("button", { name: "Ajustes" }).click();

    const tabs = page.getByRole("tab");
    const primero = tabs.first();
    await primero.click();
    await expect(primero).toBeFocused();

    // Solo la pestaña activa está en el orden de tabulación (roving tabindex, APG).
    const tabindices = await tabs.evaluateAll((els) =>
      els.map((el) => el.getAttribute("tabindex")),
    );
    expect(tabindices.filter((t) => t === "0")).toHaveLength(1);

    await page.keyboard.press("ArrowRight");
    await expect(tabs.nth(1)).toBeFocused();
    await expect(tabs.nth(1)).toHaveAttribute("aria-selected", "true");

    await page.keyboard.press("End");
    await expect(tabs.last()).toBeFocused();
    await expect(tabs.last()).toHaveAttribute("aria-selected", "true");

    await page.keyboard.press("Home");
    await expect(primero).toBeFocused();
    await expect(primero).toHaveAttribute("aria-selected", "true");
  });
});
