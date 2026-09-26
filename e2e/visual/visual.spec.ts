import { test, expect } from "@playwright/test";

/**
 * Revisión visual real (T147, T133). Los tres nombres de esta describe son los del
 * placeholder original; el tercero se deja `skip` con el motivo, en vez de fabricar una
 * comprobación sobre una función que no existe todavía (ver el `skip` más abajo).
 */

function tokenDeTema(page: import("@playwright/test").Page, variable: string) {
  return page.evaluate(
    (v) => getComputedStyle(document.documentElement).getPropertyValue(v).trim(),
    variable,
  );
}

test.describe("Revisión Visual, DPI y Responsive (T133)", () => {
  test("paleta y tokens de color respetan tokens.css en tema oscuro y claro", async ({ page }) => {
    await page.goto("/");

    // Oscuro es el tema por defecto (theme.svelte.ts, tokens.css §"TEMA OSCURO").
    await expect.poll(() => tokenDeTema(page, "--color-bg-main")).toBe("#191c30");
    await expect(page.locator("html")).not.toHaveAttribute("data-theme", "light");

    await page.getByRole("button", { name: "Ajustes" }).click();
    await page.getByRole("radio", { name: "Claro" }).click();

    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
    await expect.poll(() => tokenDeTema(page, "--color-bg-main")).toBe("#f8f7fb");
    await expect.poll(() => tokenDeTema(page, "--color-text-primary")).toBe("#252331");
  });

  test("escalas de zoom 100%, 150% y 200% mantienen el layout sin desbordar horizontalmente", async ({
    page,
  }) => {
    // Aproximación con zoom CSS, no con el escalado real de DPI de Windows: un navegador
    // de escritorio sin esa plataforma no puede fijar 150/200 DPI del sistema operativo.
    // Lo que sí comprueba de verdad: que el layout responsive no se rompe al agrandar el
    // contenido, que es la parte comprobable de "legibilidad" en este entorno.
    await page.goto("/");

    for (const factor of [1, 1.5, 2]) {
      await page.evaluate((z) => {
        document.documentElement.style.zoom = String(z);
      }, factor);

      const overflow = await page.evaluate(() => {
        return document.documentElement.scrollWidth - document.documentElement.clientWidth;
      });
      expect(overflow, `desbordamiento horizontal a zoom ${factor}`).toBeLessThanOrEqual(1);

      await expect(page.getByRole("heading", { name: "Equipos disponibles" })).toBeVisible();
    }
  });

  test("plantilla de impresión PDF A4 aísla estilos vectoriales SVG", () => {
    test.skip(
      true,
      "PrintReport.svelte no tiene ninguna ruta ni ventana que lo monte (grep confirma que " +
        "solo se importa desde features/export/index.ts sin ningún consumidor) y no existe " +
        "ninguna regla @media print en el proyecto: no hay nada que un E2E pueda navegar o " +
        "activar para ejercitar 'aísla estilos vectoriales SVG'. Fabricar un montaje ad-hoc " +
        "para este test simularía una integración que no existe. Declarado NO PRESENTE en " +
        "VALIDACION.md; corresponde a quien cablee la exportación a PDF (fuera de T147).",
    );
  });
});
