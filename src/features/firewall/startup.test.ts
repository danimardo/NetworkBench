import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { FirewallStartupCheck } from "./startup.svelte";
import FirewallStartupNotice from "./FirewallStartupNotice.svelte";
import {
  getFirewallRulesStatus,
  createMissingFirewallRules,
  openNetworkSettings,
  type FirewallRulesReport,
  type FirewallRuleState,
} from "../../lib/api/firewall";
import { getSettings, updateSettings } from "../../lib/api/settings";
import { IpcError } from "../../lib/api/transport";
import { setLocale } from "../../lib/i18n";

// Al arrancar se lee el estado del cortafuegos y, si algo falta, se deja un aviso con una
// acción. Un usuario nuevo no debe descubrir la falta de reglas cuando otro equipo no le
// conecta. La comprobación nunca eleva por su cuenta: el UAC solo sale al pulsar «Crear ahora».

vi.mock("../../lib/api/firewall", async (original) => ({
  ...(await original<typeof import("../../lib/api/firewall")>()),
  getFirewallRulesStatus: vi.fn(),
  createMissingFirewallRules: vi.fn(),
  openNetworkSettings: vi.fn(),
}));
vi.mock("../../lib/api/settings", () => ({ getSettings: vi.fn(), updateSettings: vi.fn() }));

function regla(nombre: string, extra: Partial<FirewallRuleState> = {}): FirewallRuleState {
  return {
    nombre,
    protocolo: "TCP",
    puertos: "7411",
    programa: "C:\\Apps\\NetworkBench\\NetworkBench.exe",
    perfiles: ["Domain", "Private"],
    grupo: "NetworkBench",
    estado: "present",
    detalle: "",
    programaExiste: true,
    comandoAgregar: "New-NetFirewallRule",
    ...extra,
  };
}

function informe(
  extra: Partial<FirewallRulesReport> & { estado?: FirewallRuleState["estado"] } = {},
) {
  const { estado = "present", ...resto } = extra;
  return {
    reglas: ["Control", "NTTTCP TCP", "NTTTCP UDP", "Descubrimiento"].map((n) =>
      regla(`NetworkBench - ${n}`, { estado }),
    ),
    redes: [{ nombre: "Red", interfaz: "Ethernet", categoria: "Private" }],
    permitirPublico: false,
    puertoControl: 7411,
    ayudanteDisponible: true,
    ...resto,
  } satisfies FirewallRulesReport;
}

const publica = { nombre: "Red", interfaz: "Ethernet", categoria: "Public" };

beforeEach(() => {
  vi.mocked(getFirewallRulesStatus).mockReset();
  vi.mocked(createMissingFirewallRules).mockReset();
  vi.mocked(openNetworkSettings).mockReset();
  vi.mocked(getSettings).mockReset();
  vi.mocked(updateSettings).mockReset();
});
afterEach(() => setLocale("es"));

describe("FirewallStartupCheck", () => {
  it("con las reglas presentes y la red privada no avisa", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe());
    const c = new FirewallStartupCheck();
    await c.comprobar();
    expect(c.motivo).toBeNull();
    expect(c.visible).toBe(false);
  });

  it("si faltan reglas avisa con motivo «faltan»", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ estado: "missing" }));
    const c = new FirewallStartupCheck();
    await c.comprobar();
    expect(c.motivo).toBe("faltan");
    expect(c.visible).toBe(true);
    expect(c.porCrear).toHaveLength(4);
  });

  it("una regla desactualizada o deshabilitada también cuenta", async () => {
    for (const estado of ["modified", "disabled"] as const) {
      vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ estado }));
      const c = new FirewallStartupCheck();
      await c.comprobar();
      expect(c.motivo).toBe("faltan");
    }
  });

  it("si lo único que falta no se puede crear (programa ausente), no avisa de algo irresoluble", async () => {
    const inf = informe({ estado: "missing" });
    inf.reglas = inf.reglas.map((r) => ({ ...r, programaExiste: false }));
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(inf);
    const c = new FirewallStartupCheck();
    await c.comprobar();
    expect(c.motivo).toBeNull();
  });

  it("con las reglas bien pero la red pública sin permiso, avisa con motivo «redPublica»", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ redes: [publica] }));
    const c = new FirewallStartupCheck();
    await c.comprobar();
    expect(c.motivo).toBe("redPublica");
    expect(c.redesPublicas).toHaveLength(1);
  });

  it("si ya se permitieron las redes públicas, una red pública no avisa", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe({ redes: [publica], permitirPublico: true }),
    );
    const c = new FirewallStartupCheck();
    await c.comprobar();
    expect(c.motivo).toBeNull();
  });

  it("si no se puede leer el cortafuegos, no afirma nada ni rompe el arranque", async () => {
    vi.mocked(getFirewallRulesStatus).mockRejectedValue(new Error("PowerShell no disponible"));
    const c = new FirewallStartupCheck();
    await expect(c.comprobar()).resolves.toBeUndefined();
    expect(c.visible).toBe(false);
    expect(c.report).toBeNull();
  });

  it("«Ahora no» lo oculta aunque el problema siga", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ estado: "missing" }));
    const c = new FirewallStartupCheck();
    await c.comprobar();
    c.descartar();
    expect(c.visible).toBe(false);
    await c.comprobar();
    expect(c.visible).toBe(false);
  });

  it("«Crear ahora» crea, actualiza el informe, avisa de éxito y el aviso desaparece", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ estado: "missing" }));
    vi.mocked(createMissingFirewallRules).mockResolvedValue(informe());
    const c = new FirewallStartupCheck();
    await c.comprobar();
    await c.crearAhora();
    expect(createMissingFirewallRules).toHaveBeenCalledTimes(1);
    expect(c.visible).toBe(false);
    expect(c.error).toBeNull();
    // Antes solo desaparecía el aviso; sin confirmación positiva no se distinguía de un
    // fallo silencioso.
    expect(c.exito).toBe("Reglas del cortafuegos creadas correctamente");
  });

  it("el aviso de éxito se cierra solo, como en Ajustes → Cortafuegos", async () => {
    vi.useFakeTimers();
    try {
      vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ estado: "missing" }));
      vi.mocked(createMissingFirewallRules).mockResolvedValue(informe());
      const c = new FirewallStartupCheck();
      await c.comprobar();
      await c.crearAhora();
      expect(c.exito).not.toBeNull();
      await vi.advanceTimersByTimeAsync(3100);
      expect(c.exito).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });

  it("si el UAC se rechaza, no hay ningún aviso de éxito", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ estado: "missing" }));
    vi.mocked(createMissingFirewallRules).mockRejectedValue(
      new IpcError({
        code: "NB-FW-004",
        severity: "error",
        messageKey: "errors.NB-FW-004",
        issues: [],
        actions: [],
      } as never),
    );
    const c = new FirewallStartupCheck();
    await c.comprobar();
    await c.crearAhora();
    expect(c.exito).toBeNull();
  });

  it("si el UAC se rechaza, sigue avisando y muestra el error traducido", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ estado: "missing" }));
    vi.mocked(createMissingFirewallRules).mockRejectedValue(
      new IpcError({
        code: "NB-FW-004",
        severity: "error",
        messageKey: "errors.NB-FW-004",
        issues: [],
        actions: [],
      } as never),
    );
    const c = new FirewallStartupCheck();
    await c.comprobar();
    await c.crearAhora();
    expect(c.visible).toBe(true);
    expect(c.error).toContain("Permiso de elevación de administrador (UAC) no concedido");
    expect(c.creando).toBe(false);
  });

  it("no eleva por su cuenta: comprobar nunca llama a crear", async () => {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(informe({ estado: "missing" }));
    const c = new FirewallStartupCheck();
    await c.comprobar();
    expect(createMissingFirewallRules).not.toHaveBeenCalled();
  });
});

describe("FirewallStartupCheck: permitir redes públicas", () => {
  const PREFS = { schemaVersion: 1, firewallAllowPublic: false } as never;

  it("guarda el permiso, relee y recrea las reglas, en ese orden", async () => {
    const orden: string[] = [];
    vi.mocked(getSettings).mockImplementation(async () => PREFS);
    vi.mocked(updateSettings).mockImplementation(async (p) => {
      orden.push(`guardar:${p.firewallAllowPublic}`);
      return p;
    });
    vi.mocked(getFirewallRulesStatus).mockImplementation(async () => {
      orden.push("leer");
      return informe({ redes: [publica], estado: "modified" });
    });
    vi.mocked(createMissingFirewallRules).mockImplementation(async () => {
      orden.push("crear");
      return informe({ redes: [publica], permitirPublico: true });
    });

    const c = new FirewallStartupCheck();
    await c.permitirPublico();

    expect(orden).toEqual(["guardar:true", "leer", "crear"]);
    expect(c.visible).toBe(false);
    expect(c.error).toBeNull();
    expect(c.exito).toBe("Reglas del cortafuegos creadas correctamente");
  });

  it("si el UAC se rechaza, el permiso queda guardado y el aviso pasa a ofrecer crear", async () => {
    vi.mocked(getSettings).mockResolvedValue(PREFS);
    vi.mocked(updateSettings).mockImplementation(async (p) => p);
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe({ redes: [publica], estado: "modified", permitirPublico: true }),
    );
    vi.mocked(createMissingFirewallRules).mockRejectedValue(
      new IpcError({
        code: "NB-FW-004",
        severity: "error",
        messageKey: "errors.NB-FW-004",
        issues: [],
        actions: [],
      } as never),
    );
    const c = new FirewallStartupCheck();
    await c.permitirPublico();

    expect(updateSettings).toHaveBeenCalledWith(
      expect.objectContaining({ firewallAllowPublic: true }),
    );
    expect(c.error).toContain("UAC");
    expect(c.motivo).toBe("faltan");
    expect(c.visible).toBe(true);
    expect(c.exito).toBeNull();
  });

  it("si no se puede guardar el permiso, no llega a tocar las reglas", async () => {
    vi.mocked(getSettings).mockResolvedValue(PREFS);
    vi.mocked(updateSettings).mockRejectedValue(new Error("disco lleno"));
    const c = new FirewallStartupCheck();
    await c.permitirPublico();
    expect(createMissingFirewallRules).not.toHaveBeenCalled();
    expect(c.error).toContain("disco lleno");
  });

  it("abre la configuración de red de Windows", async () => {
    vi.mocked(openNetworkSettings).mockResolvedValue(undefined);
    await new FirewallStartupCheck().abrirConfiguracionDeRed();
    expect(openNetworkSettings).toHaveBeenCalledTimes(1);
  });
});

describe("FirewallStartupNotice", () => {
  async function comprobado(inf: FirewallRulesReport) {
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(inf);
    const c = new FirewallStartupCheck();
    await c.comprobar();
    return c;
  }

  it("flota (no ocupa sitio) y ofrece «Crear ahora» cuando faltan reglas", async () => {
    const check = await comprobado(informe({ estado: "missing" }));
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });

    const aviso = await screen.findByTestId("fw-startup-notice");
    expect(aviso.closest(".nb-toastlayer")).not.toBeNull();
    expect(aviso.textContent).toContain("Faltan reglas del cortafuegos de Windows");
    expect(screen.getByRole("button", { name: "Crear ahora" })).toBeTruthy();
  });

  it("«Crear ahora» lanza la creación y el aviso se va cuando ya están", async () => {
    vi.mocked(createMissingFirewallRules).mockResolvedValue(informe());
    const check = await comprobado(informe({ estado: "missing" }));
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });

    await fireEvent.click(await screen.findByRole("button", { name: "Crear ahora" }));
    expect(createMissingFirewallRules).toHaveBeenCalledTimes(1);
    await waitFor(() => expect(screen.queryByTestId("fw-startup-notice")).toBeNull());
    // El aviso persistente desaparece porque ya no hace falta, pero eso no basta como
    // confirmación: además debe verse un aviso positivo de que la acción funcionó.
    expect(await screen.findByText("Reglas del cortafuegos creadas correctamente")).toBeTruthy();
  });

  it("con una red pública explica cuál y ofrece las dos salidas, sin «Revisar»", async () => {
    const check = await comprobado(informe({ redes: [publica] }));
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });

    const aviso = await screen.findByTestId("fw-startup-notice");
    expect(aviso.textContent).toContain("Windows considera pública tu red «Red (Ethernet)»");
    expect(screen.getByRole("button", { name: "Permitir en redes públicas" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Abrir configuración de red" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Revisar" })).toBeNull();
  });

  it("«Permitir en redes públicas» pide confirmación explicando la exposición, y cancelar no hace nada", async () => {
    const check = await comprobado(informe({ redes: [publica] }));
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });

    await fireEvent.click(
      await screen.findByRole("button", { name: "Permitir en redes públicas" }),
    );
    expect(await screen.findByText("¿Permitir NetworkBench en redes públicas?")).toBeTruthy();
    expect(
      screen.getByText(/Otros equipos de esas redes podrán llegar a los puertos/),
    ).toBeTruthy();

    await fireEvent.click(screen.getByTestId("fw-public-confirm-cancel"));
    expect(updateSettings).not.toHaveBeenCalled();
    expect(createMissingFirewallRules).not.toHaveBeenCalled();
    expect(screen.queryByText("¿Permitir NetworkBench en redes públicas?")).toBeNull();
  });

  it("al confirmar guarda el permiso y crea las reglas (UAC)", async () => {
    vi.mocked(getSettings).mockResolvedValue({ schemaVersion: 1 } as never);
    vi.mocked(updateSettings).mockImplementation(async (p) => p);
    vi.mocked(createMissingFirewallRules).mockResolvedValue(
      informe({ redes: [publica], permitirPublico: true }),
    );
    const check = await comprobado(informe({ redes: [publica] }));
    vi.mocked(getFirewallRulesStatus).mockResolvedValue(
      informe({ redes: [publica], estado: "modified" }),
    );
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });

    await fireEvent.click(
      await screen.findByRole("button", { name: "Permitir en redes públicas" }),
    );
    await fireEvent.click(await screen.findByTestId("fw-public-confirm-accept"));

    await waitFor(() => expect(createMissingFirewallRules).toHaveBeenCalledTimes(1));
    expect(updateSettings).toHaveBeenCalledWith(
      expect.objectContaining({ firewallAllowPublic: true }),
    );
    await waitFor(() => expect(screen.queryByTestId("fw-startup-notice")).toBeNull());
    expect(await screen.findByText("Reglas del cortafuegos creadas correctamente")).toBeTruthy();
  });

  it("«Abrir configuración de red» abre Windows sin pedir confirmación", async () => {
    vi.mocked(openNetworkSettings).mockResolvedValue(undefined);
    const check = await comprobado(informe({ redes: [publica] }));
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });

    await fireEvent.click(
      await screen.findByRole("button", { name: "Abrir configuración de red" }),
    );
    expect(openNetworkSettings).toHaveBeenCalledTimes(1);
  });

  it("sin el ayudante elevado no ofrece crear: remite a Ajustes", async () => {
    const check = await comprobado(informe({ estado: "missing", ayudanteDisponible: false }));
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });

    await screen.findByTestId("fw-startup-notice");
    expect(screen.queryByRole("button", { name: "Crear ahora" })).toBeNull();
    expect(screen.getByRole("button", { name: "Revisar" })).toBeTruthy();
  });

  it("la X («Ahora no») lo cierra", async () => {
    const check = await comprobado(informe({ estado: "missing" }));
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });

    await fireEvent.click(await screen.findByRole("button", { name: "Cerrar aviso" }));
    expect(screen.queryByTestId("fw-startup-notice")).toBeNull();
  });

  it("sale en el idioma activo", async () => {
    setLocale("en");
    const check = await comprobado(informe({ estado: "missing" }));
    render(FirewallStartupNotice, { props: { check, onReview: () => {} } });
    expect((await screen.findByTestId("fw-startup-notice")).textContent).toContain(
      "Some Windows firewall rules are missing",
    );
    expect(screen.getByRole("button", { name: "Create now" })).toBeTruthy();
  });
});
