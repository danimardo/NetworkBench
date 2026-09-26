import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import SettingsScreen from "./SettingsScreen.svelte";
import { updateSettings } from "../../lib/api/settings";
import {
  getFirewallRulesStatus,
  createMissingFirewallRules,
  removeFirewallRules,
  type FirewallRulesReport,
  type FirewallRuleState,
} from "../../lib/api/firewall";
import { IpcError } from "../../lib/api/transport";
import { setLocale } from "../../lib/i18n";

// Ajustes → Cortafuegos (Historias.md §14.6) y el aviso de la pestaña Red al cambiar el
// puerto de control (§14.1: «se marcan desactualizadas y se ofrece recrearlas»).

const PREFS = {
  schemaVersion: 1,
  theme: "dark",
  locale: "es",
  reduceMotion: false,
  logLevel: "warn",
  autoAcceptTrusted: false,
  customControlPort: null as number | null,
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
vi.mock("../../lib/api/firewall", async (original) => ({
  ...(await original<typeof import("../../lib/api/firewall")>()),
  getFirewallRulesStatus: vi.fn(),
  createMissingFirewallRules: vi.fn(),
  removeFirewallRules: vi.fn(),
}));

function regla(nombre: string, extra: Partial<FirewallRuleState> = {}): FirewallRuleState {
  return {
    nombre,
    protocolo: "TCP",
    puertos: "7411",
    programa: "C:\\Apps\\NetworkBench\\NetworkBench.exe",
    perfiles: ["Domain", "Private"],
    estado: "present",
    detalle: "Regla presente y activa",
    programaExiste: true,
    netshAgregar: `netsh advfirewall firewall add rule name="${nombre}" dir=in action=allow`,
    ...extra,
  };
}

function informe(
  estados: Partial<FirewallRuleState>[] | "todas" | "ninguna",
  extra: Partial<FirewallRulesReport> = {},
): FirewallRulesReport {
  const nombres = [
    "NetworkBench - Control",
    "NetworkBench - NTTTCP TCP",
    "NetworkBench - NTTTCP UDP",
    "NetworkBench - Descubrimiento",
  ];
  const reglas = nombres.map((n, i) => {
    if (estados === "todas") return regla(n);
    if (estados === "ninguna") return regla(n, { estado: "missing", detalle: "Regla ausente" });
    return regla(n, estados[i] ?? {});
  });
  return { reglas, puertoControl: 7411, ayudanteDisponible: true, ...extra };
}

function errorDe(codigo: string, clave: string): IpcError {
  return new IpcError({
    code: codigo,
    severity: "error",
    messageKey: clave,
    issues: [],
    actions: [],
  } as never);
}

async function abrirPestana(nombre: RegExp) {
  await fireEvent.click(await screen.findByRole("tab", { name: nombre }));
}

describe("Ajustes → Cortafuegos", () => {
  beforeEach(() => {
    vi.mocked(getFirewallRulesStatus).mockReset();
    vi.mocked(createMissingFirewallRules).mockReset();
    vi.mocked(removeFirewallRules).mockReset();
  });
  afterEach(() => setLocale("es"));

  it("lista las reglas con su estado real, no un único «ausente»", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe([
        {},
        { estado: "missing" },
        { estado: "modified", detalle: "puertos 5001" },
        { estado: "disabled" },
      ]),
    );
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    const filas = await screen.findAllByTestId("fw-rule");
    expect(filas).toHaveLength(4);
    expect(filas[0]!.textContent).toContain("NetworkBench - Control");
    expect(filas[0]!.textContent).toContain("Presente");
    expect(filas[1]!.textContent).toContain("Ausente");
    expect(filas[2]!.textContent).toContain("Desactualizada");
    expect(filas[3]!.textContent).toContain("Deshabilitada");
  });

  it("«Crear las que faltan» abre el UAC, actualiza la lista y avisa", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe("ninguna"));
    vi.mocked(createMissingFirewallRules).mockResolvedValue(informe("todas"));
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    const crear = await screen.findByTestId("fw-create");
    await waitFor(() => expect((crear as HTMLButtonElement).disabled).toBe(false));
    await fireEvent.click(crear);

    expect(createMissingFirewallRules).toHaveBeenCalledTimes(1);
    expect(await screen.findByText("Reglas del cortafuegos creadas correctamente")).toBeTruthy();
    await waitFor(() => expect(screen.getAllByText("Presente")).toHaveLength(4));
    // Con todo presente ya no hay nada que crear.
    expect((screen.getByTestId("fw-create") as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText("Todas las reglas están presentes y activas.")).toBeTruthy();
  });

  it("si el UAC se rechaza, el error sale traducido y la lista no cambia", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe("ninguna"));
    vi.mocked(createMissingFirewallRules).mockRejectedValue(
      errorDe("NB-FW-004", "errors.NB-FW-004"),
    );
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    const crear = await screen.findByTestId("fw-create");
    await waitFor(() => expect((crear as HTMLButtonElement).disabled).toBe(false));
    await fireEvent.click(crear);

    const alerta = await screen.findByRole("alert");
    expect(alerta.textContent).toContain(
      "Permiso de elevación de administrador (UAC) no concedido",
    );
    expect(screen.getAllByText("Ausente").length).toBe(4);
  });

  it("«Eliminar todas» pide confirmación y solo elimina si se acepta", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe("todas"));
    vi.mocked(removeFirewallRules).mockResolvedValue(informe("ninguna"));
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    const eliminar = await screen.findByTestId("fw-remove");
    await waitFor(() => expect((eliminar as HTMLButtonElement).disabled).toBe(false));

    await fireEvent.click(eliminar);
    await fireEvent.click(await screen.findByRole("button", { name: "Cancelar" }));
    expect(removeFirewallRules).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByTestId("fw-remove"));
    await fireEvent.click(await screen.findByTestId("fw-remove-confirm"));
    expect(removeFirewallRules).toHaveBeenCalledTimes(1);
    expect(await screen.findByText("Reglas del cortafuegos eliminadas")).toBeTruthy();
  });

  it("«Eliminar todas» está desactivado si no hay ninguna regla", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe("ninguna"));
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);
    await screen.findAllByTestId("fw-rule");
    expect((screen.getByTestId("fw-remove") as HTMLButtonElement).disabled).toBe(true);
  });

  it("las instrucciones manuales traen un netsh por regla y otro para eliminarla", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe("ninguna"));
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    await fireEvent.click(await screen.findByTestId("fw-manual"));
    const texto = (await screen.findByTestId("fw-instructions")).textContent ?? "";
    expect(texto.match(/add rule name=/g)).toHaveLength(4);
    expect(texto).toContain('delete rule name="NetworkBench - Control"');
  });

  it("una regla cuyo programa no existe se avisa y no cuenta como «por crear»", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe([
        {},
        { estado: "missing", programaExiste: false, programa: "C:\\x\\ntttcp.exe" },
        {},
        {},
      ]),
    );
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    expect(
      await screen.findByText("No se encuentra ntttcp.exe: esta regla no se puede crear."),
    ).toBeTruthy();
    // Lo único que falta no se puede crear: no hay nada que ofrecer.
    expect((screen.getByTestId("fw-create") as HTMLButtonElement).disabled).toBe(true);
  });

  it("sin el ayudante elevado se explica y se deshabilitan crear y eliminar", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe("ninguna", { ayudanteDisponible: false }),
    );
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    expect(await screen.findByText(/No se encuentra el ayudante de firewall/)).toBeTruthy();
    expect((screen.getByTestId("fw-create") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByTestId("fw-remove") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByTestId("fw-manual") as HTMLButtonElement).disabled).toBe(false);
  });
});

describe("Ajustes → Red: cambiar el puerto de control", () => {
  beforeEach(() => {
    vi.mocked(getFirewallRulesStatus).mockReset();
    vi.mocked(createMissingFirewallRules).mockReset();
    vi.mocked(updateSettings).mockReset();
    vi.mocked(updateSettings).mockImplementation((p) => Promise.resolve(p));
  });

  async function escribirPuerto(valor: string) {
    await abrirPestana(/Red/);
    const campo = (await screen.findByPlaceholderText("7411")) as HTMLInputElement;
    await waitFor(() => expect(campo.disabled).toBe(false));
    await fireEvent.input(campo, { target: { value: valor } });
    await fireEvent.blur(campo);
    return campo;
  }

  it("guarda el puerto nuevo y, si la regla no vale, ofrece recrearla", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe([{ estado: "modified", puertos: "7411" }], { puertoControl: 9000 }),
    );
    render(SettingsScreen);
    await escribirPuerto("9000");

    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(
        expect.objectContaining({ customControlPort: 9000 }),
      ),
    );
    const aviso = await screen.findByTestId("fw-port-notice");
    expect(aviso.textContent).toContain("El puerto de control ahora es 9000");
    expect(aviso.textContent).toContain("sigue siendo de otro puerto");
  });

  it("si no hay regla, el aviso lo dice; «Recrear regla» la crea y el aviso desaparece", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe("ninguna", { puertoControl: 9000 }),
    );
    vi.mocked(createMissingFirewallRules).mockResolvedValue(
      informe("todas", { puertoControl: 9000 }),
    );
    render(SettingsScreen);
    await escribirPuerto("9000");

    const aviso = await screen.findByTestId("fw-port-notice");
    expect(aviso.textContent).toContain("no tiene ninguna regla que lo permita");

    await fireEvent.click(screen.getByTestId("fw-recreate"));
    expect(createMissingFirewallRules).toHaveBeenCalledTimes(1);
    await waitFor(() => expect(screen.queryByTestId("fw-port-notice")).toBeNull());
  });

  it("si la regla ya vale para el puerto nuevo, no hay aviso", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe("todas", { puertoControl: 9000 }));
    render(SettingsScreen);
    await escribirPuerto("9000");

    await waitFor(() => expect(getFirewallRulesStatus).toHaveBeenCalled());
    expect(screen.queryByTestId("fw-port-notice")).toBeNull();
  });

  it("un puerto ocupado avisa, no comprueba el cortafuegos y devuelve el campo al puerto anterior", async () => {
    vi.mocked(updateSettings).mockRejectedValue(errorDe("NB-PORT-001", "errors.NB-PORT-001"));
    render(SettingsScreen);
    const campo = await escribirPuerto("9000");

    const alerta = await screen.findByRole("alert");
    expect(alerta.textContent).toContain("ya está en uso por otra aplicación");
    await waitFor(() => expect(campo.value).toBe(""));
    expect(getFirewallRulesStatus).not.toHaveBeenCalled();
    expect(screen.queryByTestId("fw-port-notice")).toBeNull();
  });

  it("salir del campo sin cambiar nada no guarda ni comprueba nada", async () => {
    render(SettingsScreen);
    await abrirPestana(/Red/);
    const campo = (await screen.findByPlaceholderText("7411")) as HTMLInputElement;
    await waitFor(() => expect(campo.disabled).toBe(false));
    await fireEvent.blur(campo);
    expect(updateSettings).not.toHaveBeenCalled();
    expect(getFirewallRulesStatus).not.toHaveBeenCalled();
  });

  it("un puerto fuera de rango se rechaza en la interfaz y no llega al backend", async () => {
    render(SettingsScreen);
    await escribirPuerto("80");
    expect(await screen.findByText(/puerto/i, { selector: "p[role=alert]" })).toBeTruthy();
    expect(updateSettings).not.toHaveBeenCalled();
  });
});
