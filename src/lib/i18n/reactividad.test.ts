import { describe, it, expect, afterEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { tick } from "svelte";
import Sidebar from "../components/Sidebar.svelte";
import TitleBar from "../components/TitleBar.svelte";
import Switch from "../components/Switch.svelte";
import Segmented from "../components/Segmented.svelte";
import DeviceCard from "../components/DeviceCard.svelte";
import { setLocale, getLocale, t } from "./index";

// El idioma es un estado global del módulo: cada prueba lo devuelve a español.
afterEach(() => setLocale("es"));

describe("i18n reactivo: el idioma cambia sin recargar ni salir de la pantalla", () => {
  it("t() devuelve el texto del idioma activo", () => {
    expect(t("nav.home")).toBe("Inicio");
    setLocale("en");
    expect(t("nav.home")).toBe("Home");
    expect(getLocale()).toBe("en");
  });

  it("el Sidebar ya montado se redibuja en inglés al cambiar el idioma", async () => {
    render(Sidebar, { props: { activeId: "inicio", onNavigate: () => {} } });
    expect(screen.getByText("Inicio")).toBeTruthy();
    expect(screen.getByText("Historial")).toBeTruthy();
    expect(screen.getByText("Ajustes")).toBeTruthy();

    setLocale("en");
    await tick();

    // Es el fallo original: antes «Inicio», «Historial» y «Ajustes» seguían en español.
    expect(screen.getByText("Home")).toBeTruthy();
    expect(screen.getByText("History")).toBeTruthy();
    expect(screen.getByText("Settings")).toBeTruthy();
    expect(screen.queryByText("Inicio")).toBeNull();
  });

  it("los botones de la barra de título tienen nombre accesible en el idioma activo", async () => {
    render(TitleBar);
    expect(screen.getByRole("button", { name: "Minimizar" })).toBeTruthy();

    setLocale("en");
    await tick();

    expect(screen.getByRole("button", { name: "Minimize" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Close" })).toBeTruthy();
  });

  it("las etiquetas de estado de DeviceCard siguen al idioma (no son una constante de módulo)", async () => {
    render(DeviceCard, {
      props: {
        name: "PC",
        ip: "10.0.0.2",
        adapterType: "ethernet",
        trust: "unknown",
        availability: "available",
      },
    });
    expect(screen.getByText("Disponible")).toBeTruthy();

    setLocale("en");
    await tick();

    expect(screen.getByText("Available")).toBeTruthy();
    expect(screen.queryByText("Disponible")).toBeNull();
  });
});

describe("Switch", () => {
  it("es un role=switch con su estado accesible y un texto Activado/Desactivado", () => {
    render(Switch, { props: { checked: false, onchange: () => {}, label: "Iniciar con Windows" } });
    const sw = screen.getByRole("switch", { name: "Iniciar con Windows" });
    expect(sw.getAttribute("aria-checked")).toBe("false");
    expect(screen.getByText("Desactivado")).toBeTruthy();
  });

  it("al pulsarlo pide el valor contrario y el texto refleja el estado", async () => {
    const cambios: boolean[] = [];
    const { rerender } = render(Switch, {
      props: { checked: false, onchange: (v: boolean) => cambios.push(v), label: "x" },
    });
    await fireEvent.click(screen.getByRole("switch"));
    expect(cambios).toEqual([true]);

    await rerender({ checked: true, onchange: (v: boolean) => cambios.push(v), label: "x" });
    expect(screen.getByRole("switch").getAttribute("aria-checked")).toBe("true");
    expect(screen.getByText("Activado")).toBeTruthy();
  });

  it("desactivado no responde al clic", async () => {
    const cambios: boolean[] = [];
    render(Switch, {
      props: {
        checked: false,
        disabled: true,
        onchange: (v: boolean) => cambios.push(v),
        label: "x",
      },
    });
    await fireEvent.click(screen.getByRole("switch"));
    expect(cambios).toEqual([]);
  });
});

describe("Segmented", () => {
  const opciones = [
    { id: "warn", label: "Warn" },
    { id: "info", label: "Info" },
    { id: "debug", label: "Debug" },
  ] as const;

  it("marca la opción seleccionada con aria-checked y solo ella está en el orden de Tab", () => {
    render(Segmented, {
      props: { options: opciones, value: "info", onchange: () => {}, label: "Nivel" },
    });
    const radios = screen.getAllByRole("radio");
    expect(radios.map((r) => r.getAttribute("aria-checked"))).toEqual(["false", "true", "false"]);
    expect(radios.map((r) => r.getAttribute("tabindex"))).toEqual(["-1", "0", "-1"]);
  });

  it("las flechas mueven y seleccionan; da la vuelta por los extremos", async () => {
    const elegidos: string[] = [];
    render(Segmented, {
      props: {
        options: opciones,
        value: "debug",
        onchange: (v: string) => elegidos.push(v),
        label: "Nivel",
      },
    });
    const grupo = screen.getByRole("radiogroup");
    await fireEvent.keyDown(grupo, { key: "ArrowRight" });
    await fireEvent.keyDown(grupo, { key: "ArrowLeft" });
    expect(elegidos).toEqual(["warn", "info"]);
  });

  it("con un valor que no es ninguna opción, no marca nada y deja entrar por la primera", () => {
    render(Segmented, {
      props: { options: opciones, value: "error", onchange: () => {}, label: "Nivel" },
    });
    const radios = screen.getAllByRole("radio");
    expect(radios.every((r) => r.getAttribute("aria-checked") === "false")).toBe(true);
    expect(radios[0]!.getAttribute("tabindex")).toBe("0");
  });
});
