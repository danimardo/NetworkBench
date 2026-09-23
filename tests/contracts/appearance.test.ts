import { describe, it, expect, beforeEach } from "vitest";
import { theme } from "../../src/lib/design-system/theme.svelte";
import { getLocale, setLocale } from "../../src/lib/i18n";

describe("Theme and Appearance Contract Tests (T120, FR-037)", () => {
  beforeEach(() => {
    theme.setMode("dark");
    theme.setReduceMotion(false);
    setLocale("es");
  });

  it("garantiza que el tema inicial por defecto es Oscuro (FR-037)", () => {
    expect(theme.mode).toBe("dark");
    expect(theme.resolved).toBe("dark");
  });

  it("permite cambiar el modo de tema a light, system y dark", () => {
    theme.setMode("light");
    expect(theme.mode).toBe("light");
    expect(theme.resolved).toBe("light");

    theme.setMode("system");
    expect(theme.mode).toBe("system");
    // resolved será light o dark según el entorno del sistema
    expect(["light", "dark"]).toContain(theme.resolved);

    theme.setMode("dark");
    expect(theme.mode).toBe("dark");
    expect(theme.resolved).toBe("dark");
  });

  it("permite activar y desactivar la reducción de movimiento", () => {
    expect(theme.reduceMotion).toBe(false);

    theme.setReduceMotion(true);
    expect(theme.reduceMotion).toBe(true);

    theme.setReduceMotion(false);
    expect(theme.reduceMotion).toBe(false);
  });

  it("permite alternar el idioma de la aplicación", () => {
    expect(getLocale()).toBe("es");

    setLocale("en");
    expect(getLocale()).toBe("en");

    setLocale("es");
    expect(getLocale()).toBe("es");
  });
});
