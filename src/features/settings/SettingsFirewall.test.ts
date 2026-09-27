import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import SettingsScreen from "./SettingsScreen.svelte";
import { updateSettings } from "../../lib/api/settings";
import {
  getFirewallRulesStatus,
  createMissingFirewallRules,
  removeFirewallRules,
  openNetworkSettings,
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
  closeAction: "ask",
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
  getDiagnosticPaths: vi.fn().mockResolvedValue(null),
  openLogFolder: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("../../lib/api/updater", () => ({ checkUpdate: vi.fn(), evaluateUpdate: vi.fn() }));
vi.mock("../../lib/api/firewall", async (original) => ({
  ...(await original<typeof import("../../lib/api/firewall")>()),
  getFirewallRulesStatus: vi.fn(),
  createMissingFirewallRules: vi.fn(),
  removeFirewallRules: vi.fn(),
  openNetworkSettings: vi.fn(),
}));

function regla(nombre: string, extra: Partial<FirewallRuleState> = {}): FirewallRuleState {
  return {
    nombre,
    protocolo: "TCP",
    puertos: "7411",
    programa: "C:\\Apps\\NetworkBench\\NetworkBench.exe",
    perfiles: ["Domain", "Private"],
    grupo: "NetworkBench",
    estado: "present",
    detalle: "Regla presente y activa",
    programaExiste: true,
    comandoAgregar: `New-NetFirewallRule -DisplayName '${nombre}' -Group 'NetworkBench'`,
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
  return {
    reglas,
    redes: [{ nombre: "Red", interfaz: "Ethernet", categoria: "Private" }],
    permitirPublico: false,
    puertoControl: 7411,
    ayudanteDisponible: true,
    ...extra,
  };
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

  it("cada regla muestra sus perfiles, para ver de un vistazo si vale en redes públicas", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe([{ perfiles: ["Domain", "Private"] }, { perfiles: ["Domain", "Private", "Public"] }]),
    );
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    const filas = await screen.findAllByTestId("fw-rule");
    expect(filas[0]!.textContent).toContain("Dominio, Privado");
    expect(filas[0]!.textContent).not.toContain("Público");
    expect(filas[1]!.textContent).toContain("Dominio, Privado, Público");
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

  it("las instrucciones manuales son PowerShell, con grupo, y eliminan por grupo", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe("ninguna"));
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);

    await fireEvent.click(await screen.findByTestId("fw-manual"));
    const texto = (await screen.findByTestId("fw-instructions")).textContent ?? "";
    expect(texto.match(/New-NetFirewallRule/g)).toHaveLength(4);
    expect(texto).toContain("-Group 'NetworkBench'");
    expect(texto).toContain("Remove-NetFirewallRule -Group 'NetworkBench'");
    expect(texto).not.toContain("netsh");
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

describe("Ajustes → Cortafuegos: redes públicas (§14.5)", () => {
  const publica = { nombre: "Red", interfaz: "Ethernet", categoria: "Public" };

  beforeEach(() => {
    vi.mocked(getFirewallRulesStatus).mockReset();
    vi.mocked(createMissingFirewallRules).mockReset();
    vi.mocked(openNetworkSettings).mockReset();
    vi.mocked(updateSettings).mockReset();
    vi.mocked(updateSettings).mockImplementation((p) => Promise.resolve(p));
  });

  async function abrir(inf: FirewallRulesReport) {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(inf);
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);
    await screen.findAllByTestId("fw-rule");
  }

  it("con una red pública y sin permiso, avisa y nombra la red", async () => {
    await abrir(informe("todas", { redes: [publica] }));
    const aviso = await screen.findByTestId("fw-public-notice");
    expect(aviso.textContent).toContain("Windows considera pública la red «Red (Ethernet)»");
    expect(aviso.textContent).toContain("cambiar su tipo a «Privada»");
  });

  it("con las redes privadas, o si ya se permitió, no avisa", async () => {
    await abrir(informe("todas", { redes: [{ ...publica, categoria: "Private" }] }));
    expect(screen.queryByTestId("fw-public-notice")).toBeNull();
  });

  it("«Abrir configuración de red» abre Configuración de Windows", async () => {
    vi.mocked(openNetworkSettings).mockResolvedValue(undefined);
    await abrir(informe("todas", { redes: [publica] }));
    await fireEvent.click(await screen.findByTestId("fw-public-settings"));
    expect(openNetworkSettings).toHaveBeenCalledTimes(1);
  });

  it("«Permitir en redes públicas» guarda el permiso, relee y recrea las reglas (UAC)", async () => {
    vi.mocked(getFirewallRulesStatus)
      .mockResolvedValueOnce(informe("todas", { redes: [publica] }))
      // Tras dar el permiso, las reglas actuales no lo incluyen: quedan desactualizadas.
      .mockResolvedValueOnce(
        informe(
          [
            { estado: "modified" },
            { estado: "modified" },
            { estado: "modified" },
            { estado: "modified" },
          ],
          {
            redes: [publica],
            permitirPublico: true,
          },
        ),
      );
    vi.mocked(createMissingFirewallRules).mockResolvedValue(
      informe("todas", { redes: [publica], permitirPublico: true }),
    );
    render(SettingsScreen);
    await abrirPestana(/Cortafuegos/);
    await screen.findAllByTestId("fw-rule");

    await fireEvent.click(await screen.findByTestId("fw-public-allow"));
    // Da el permiso solo tras explicar la exposición y confirmar.
    await fireEvent.click(await screen.findByTestId("fw-public-confirm-accept"));

    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(
        expect.objectContaining({ firewallAllowPublic: true }),
      ),
    );
    await waitFor(() => expect(createMissingFirewallRules).toHaveBeenCalledTimes(1));
    // Ya permitido: el aviso desaparece y el interruptor queda activado con su advertencia.
    await waitFor(() => expect(screen.queryByTestId("fw-public-notice")).toBeNull());
    const sw = screen.getByRole("switch", { name: "Permitir también en redes públicas" });
    expect(sw.getAttribute("aria-checked")).toBe("true");
    expect(screen.getByText(/podrán llegar a los puertos de NetworkBench/)).toBeTruthy();
  });

  it("encender el interruptor pide confirmación; cancelar no guarda nada", async () => {
    await abrir(informe("todas"));
    const sw = screen.getByRole("switch", { name: "Permitir también en redes públicas" });
    await waitFor(() => expect((sw as HTMLButtonElement).disabled).toBe(false));

    await fireEvent.click(sw);
    expect(await screen.findByText("¿Permitir NetworkBench en redes públicas?")).toBeTruthy();
    await fireEvent.click(screen.getByTestId("fw-public-confirm-cancel"));

    expect(updateSettings).not.toHaveBeenCalled();
    expect(sw.getAttribute("aria-checked")).toBe("false");
  });

  it("encender el interruptor y confirmar guarda el permiso sin pedir UAC todavía", async () => {
    await abrir(informe("todas"));
    const sw = screen.getByRole("switch", { name: "Permitir también en redes públicas" });
    await waitFor(() => expect((sw as HTMLButtonElement).disabled).toBe(false));

    await fireEvent.click(sw);
    await fireEvent.click(await screen.findByTestId("fw-public-confirm-accept"));

    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(
        expect.objectContaining({ firewallAllowPublic: true }),
      ),
    );
    // Las reglas se recrean después, con «Crear las que faltan» (UAC): aquí no.
    expect(createMissingFirewallRules).not.toHaveBeenCalled();
  });

  it("el interruptor lo apaga: guarda la preferencia y vuelve a leer las reglas", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe("todas", { permitirPublico: true }),
    );
    await abrir(informe("todas", { permitirPublico: true }));

    const sw = screen.getByRole("switch", { name: "Permitir también en redes públicas" });
    await waitFor(() => expect((sw as HTMLButtonElement).disabled).toBe(false));
    vi.mocked(getFirewallRulesStatus).mockClear();
    await fireEvent.click(sw);

    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith(
        expect.objectContaining({ firewallAllowPublic: false }),
      ),
    );
    await waitFor(() => expect(getFirewallRulesStatus).toHaveBeenCalledTimes(1));
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
