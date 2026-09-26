import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import SettingsScreen from "./SettingsScreen.svelte";
import { updateSettings } from "../../lib/api/settings";
import { IpcError } from "../../lib/api/transport";
import { setLocale } from "../../lib/i18n";

// «Ajustes guardados correctamente» era un banner dentro de la página: al aparecer empujaba
// hacia abajo todos los controles. Ahora es un aviso flotante (ToastLayer) que no ocupa
// sitio en el flujo. Un error no se cierra solo: se queda hasta que se pulsa la X.

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
  updateSettings: vi.fn(),
  getAutostart: vi.fn().mockResolvedValue(false),
  setAutostart: vi.fn().mockResolvedValue(undefined),
  getDataInfo: vi.fn().mockResolvedValue(null),
  purgeData: vi.fn().mockResolvedValue(undefined),
  getAboutInfo: vi.fn().mockResolvedValue(null),
  evaluateAppClose: vi.fn().mockResolvedValue("allowExit"),
}));
vi.mock("../../lib/api/updater", () => ({ checkUpdate: vi.fn(), evaluateUpdate: vi.fn() }));
vi.mock("../../lib/api/firewall", () => ({ inspectFirewall: vi.fn() }));

function errorInterno(): IpcError {
  return new IpcError({
    code: "NB-INTERNAL-001",
    severity: "fatal",
    messageKey: "errors.NB-INTERNAL-001",
    issues: [],
    actions: [],
  });
}

async function alternarBandeja() {
  await fireEvent.click(await screen.findByRole("tab", { name: /Ciclo de vida/ }));
  const sw = await screen.findByRole("switch", { name: "Minimizar a la bandeja del sistema" });
  await waitFor(() => expect((sw as HTMLButtonElement).disabled).toBe(false));
  await fireEvent.click(sw);
}

describe("Ajustes: avisos flotantes que no desplazan los controles", () => {
  beforeEach(() => {
    vi.mocked(updateSettings).mockReset();
  });
  afterEach(() => {
    vi.useRealTimers();
    setLocale("es");
  });

  it("«guardado» aparece en la capa flotante y no entre el título y las pestañas", async () => {
    vi.mocked(updateSettings).mockImplementation((p) => Promise.resolve(p));
    const { container } = render(SettingsScreen);
    await alternarBandeja();

    const aviso = await screen.findByText("Ajustes guardados correctamente");
    expect(aviso.closest(".nb-toastlayer")).not.toBeNull();

    // Nada en el flujo entre la cabecera y las pestañas: es lo que empujaba los controles.
    const pestanas = screen.getByRole("tablist");
    const cabecera = container.querySelector("h1")!.parentElement!;
    expect(pestanas.previousElementSibling).toBe(cabecera);
  });

  it("el aviso de éxito se cierra solo a los 3 segundos", async () => {
    vi.mocked(updateSettings).mockImplementation((p) => Promise.resolve(p));
    render(SettingsScreen);
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    await alternarBandeja();

    await vi.waitFor(() =>
      expect(screen.queryByText("Ajustes guardados correctamente")).not.toBeNull(),
    );
    await vi.advanceTimersByTimeAsync(3100);
    await waitFor(() => expect(screen.queryByText("Ajustes guardados correctamente")).toBeNull());
  });

  it("un error sale traducido, como alerta, y se queda hasta que se cierra", async () => {
    vi.mocked(updateSettings).mockRejectedValue(errorInterno());
    render(SettingsScreen);
    await alternarBandeja();

    const alerta = await screen.findByRole("alert");
    // El texto del error, no «[NB-INTERNAL-001] errors.NB-INTERNAL-001».
    expect(alerta.textContent).toContain("Se produjo un error interno inesperado");
    expect(alerta.textContent).not.toContain("errors.NB-INTERNAL-001");
    expect(alerta.closest(".nb-toastlayer")).not.toBeNull();

    await new Promise((r) => setTimeout(r, 3300));
    expect(screen.queryByRole("alert")).not.toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Cerrar aviso" }));
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("dos guardados seguidos: el segundo aviso no lo corta el temporizador del primero", async () => {
    vi.mocked(updateSettings).mockImplementation((p) => Promise.resolve(p));
    render(SettingsScreen);
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    await alternarBandeja();
    await vi.waitFor(() =>
      expect(screen.queryByText("Ajustes guardados correctamente")).not.toBeNull(),
    );

    await vi.advanceTimersByTimeAsync(2000);
    const sw = screen.getByRole("switch", { name: "Minimizar a la bandeja del sistema" });
    await fireEvent.click(sw);
    await vi.advanceTimersByTimeAsync(2000);

    // Han pasado 4 s desde el primero, pero solo 2 desde el segundo: sigue a la vista.
    expect(screen.queryByText("Ajustes guardados correctamente")).not.toBeNull();
    await vi.advanceTimersByTimeAsync(1200);
    await waitFor(() => expect(screen.queryByText("Ajustes guardados correctamente")).toBeNull());
  });
});
