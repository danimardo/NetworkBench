import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import SettingsScreen from "./SettingsScreen.svelte";
import { updateSettings } from "../../lib/api/settings";
import { setLocale } from "../../lib/i18n";

// Los controles de Ajustes tenían un aspecto inexistente: `nb-switch` no estaba definido en
// ningún CSS, así que los interruptores eran botones invisibles; el selector del nivel de
// registro usaba estilos que solo existían dentro de otro componente; y el idioma se
// cambiaba en pantalla pero nunca se guardaba. Estas pruebas fijan que cada control existe,
// dice en qué estado está y hace lo que dice.

const PREFS = {
  schemaVersion: 1,
  theme: "dark",
  locale: "es",
  reduceMotion: false,
  logLevel: "warn",
  autoAcceptTrusted: false,
  customControlPort: null,
  autostart: false,
  minimizeToTray: false,
  mdnsEnabled: true,
};

vi.mock("../../lib/api/settings", () => ({
  getSettings: vi.fn().mockImplementation(() => Promise.resolve({ ...PREFS })),
  updateSettings: vi.fn().mockImplementation((prefs) => Promise.resolve(prefs)),
  getAutostart: vi.fn().mockResolvedValue(false),
  setAutostart: vi.fn().mockResolvedValue(undefined),
  getDataInfo: vi.fn().mockResolvedValue(null),
  purgeData: vi.fn().mockResolvedValue(undefined),
  getAboutInfo: vi.fn().mockResolvedValue({
    appName: "NetworkBench",
    appVersion: "0.2.0",
    engineName: "Microsoft NTTTCP",
    engineVersion: "5.35",
    protocolVersion: "v1",
    license: "GPL-3.0-or-later",
    copyright: "© 2026 Daniel Díez Mardomingo y colaboradores",
  }),
  evaluateAppClose: vi.fn().mockResolvedValue("allowExit"),
}));
vi.mock("../../lib/api/updater", () => ({
  checkUpdate: vi.fn().mockResolvedValue({ status: "upToDate" }),
  evaluateUpdate: vi.fn().mockResolvedValue({ status: "upToDate" }),
}));
vi.mock("../../lib/api/firewall", () => ({ inspectFirewall: vi.fn() }));

async function irAPestana(nombre: RegExp) {
  await fireEvent.click(await screen.findByRole("tab", { name: nombre }));
}

/** Espera a que los ajustes carguen: hasta entonces los controles están desactivados. */
async function cargados() {
  await waitFor(() => expect(vi.mocked(updateSettings)).toBeDefined());
  await new Promise((r) => setTimeout(r, 0));
}

describe("Ajustes: los controles existen, indican su estado y funcionan", () => {
  beforeEach(() => vi.mocked(updateSettings).mockClear());
  afterEach(() => setLocale("es"));

  it("Confianza: auto-aceptar es un interruptor visible con su estado y lo guarda", async () => {
    render(SettingsScreen);
    await irAPestana(/Trust|Confianza/i);
    await cargados();

    const sw = await screen.findByRole("switch", { name: "Auto-aceptar equipos de confianza" });
    expect(sw.getAttribute("aria-checked")).toBe("false");
    expect(screen.getByText("Desactivado")).toBeTruthy();

    await fireEvent.click(sw);
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(
        expect.objectContaining({ autoAcceptTrusted: true }),
      ),
    );
    await waitFor(() => expect(sw.getAttribute("aria-checked")).toBe("true"));
    expect(screen.getByText("Activado")).toBeTruthy();
  });

  it("Ciclo de vida: bandeja e inicio con Windows son interruptores", async () => {
    render(SettingsScreen);
    await irAPestana(/Lifecycle|Ciclo/i);
    await cargados();

    const bandeja = await screen.findByRole("switch", {
      name: "Minimizar a la bandeja del sistema",
    });
    await fireEvent.click(bandeja);
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(
        expect.objectContaining({ minimizeToTray: true }),
      ),
    );
    expect(screen.getByRole("switch", { name: "Iniciar con Windows" })).toBeTruthy();
  });

  it("Red: el descubrimiento mDNS se puede apagar y el campo del puerto existe", async () => {
    render(SettingsScreen);
    await irAPestana(/Network|Red/i);
    await cargados();

    const mdns = await screen.findByRole("switch", { name: /mDNS|descubrimiento/i });
    expect(mdns.getAttribute("aria-checked")).toBe("true");
    await fireEvent.click(mdns);
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(expect.objectContaining({ mdnsEnabled: false })),
    );
    expect(screen.getByPlaceholderText("7411")).toBeTruthy();
  });

  it("Diagnóstico: el nivel de registro marca el actual y se puede cambiar", async () => {
    render(SettingsScreen);
    await irAPestana(/Diagnostics|Diagnóstico/i);
    await cargados();

    const actual = await screen.findByRole("radio", { name: /Warn/ });
    await waitFor(() => expect(actual.getAttribute("aria-checked")).toBe("true"));
    expect(screen.getByRole("radio", { name: "Info" }).getAttribute("aria-checked")).toBe("false");

    await fireEvent.click(screen.getByRole("radio", { name: "Debug" }));
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(expect.objectContaining({ logLevel: "debug" })),
    );
    await waitFor(() =>
      expect(screen.getByRole("radio", { name: "Debug" }).getAttribute("aria-checked")).toBe(
        "true",
      ),
    );
  });

  it("Apariencia: cambiar el idioma traduce la pantalla al instante y se guarda", async () => {
    render(SettingsScreen);
    await cargados();

    expect(screen.getByRole("heading", { level: 1 }).textContent).toContain("Ajustes");
    await fireEvent.click(await screen.findByRole("radio", { name: "English" }));

    // Al instante, sin salir de la pantalla ni recargar.
    await waitFor(() =>
      expect(screen.getByRole("heading", { level: 1 }).textContent).toContain("Settings"),
    );
    // Y guardado: antes solo cambiaba en pantalla y se perdía al reabrir la app.
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(expect.objectContaining({ locale: "en" })),
    );
  });

  it("Acerca de: autoría, versión dinámica del backend e icono de la aplicación", async () => {
    render(SettingsScreen);
    await irAPestana(/About|Acerca/i);
    await cargados();

    expect(await screen.findByText("Desarrollada por Daniel Díez Mardomingo")).toBeTruthy();
    // La versión sale del backend (CARGO_PKG_VERSION), no de un texto fijo en la pantalla.
    expect(await screen.findByText("0.2.0")).toBeTruthy();
    expect(screen.queryByText("NB")).toBeNull();
    expect(document.querySelector(".nb-appicon")).not.toBeNull();

    setLocale("en");
    await waitFor(() =>
      expect(screen.getByText("Developed by Daniel Díez Mardomingo")).toBeTruthy(),
    );
  });
});
