import { describe, it, expect, afterEach, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import App from "./App.svelte";
import { setTransportMock } from "../lib/api/transport";
import { setLocale } from "../lib/i18n";
import { router } from "./router.svelte";

// Cableado real: al arrancar la aplicación se comprueba el cortafuegos en segundo plano y,
// si faltan reglas, aparece el aviso. Sin este test, todo lo de `features/firewall` podía
// pasar en verde sin que `App.svelte` lo llamara nunca.

const PREFS = {
  schemaVersion: 1,
  theme: "dark",
  locale: "es",
  reduceMotion: false,
  logLevel: "warn",
  autoAcceptTrusted: false,
  customControlPort: null,
  autostart: false,
  closeAction: "ask",
  mdnsEnabled: true,
  firewallAllowPublic: false,
};

function regla(nombre: string, estado: string) {
  return {
    nombre: `NetworkBench - ${nombre}`,
    protocolo: "TCP",
    puertos: "7411",
    programa: "C:\\Apps\\NetworkBench\\NetworkBench.exe",
    perfiles: ["Domain", "Private"],
    grupo: "NetworkBench",
    estado,
    detalle: "",
    programaExiste: true,
    comandoAgregar: "New-NetFirewallRule",
  };
}

const informe = (
  estado: string,
  redes = [{ nombre: "Red", interfaz: "Ethernet", categoria: "Private" }],
  ayudante = true,
) => ({
  reglas: ["Control", "NTTTCP TCP", "NTTTCP UDP", "Descubrimiento"].map((n) => regla(n, estado)),
  redes,
  permitirPublico: false,
  puertoControl: 7411,
  ayudanteDisponible: ayudante,
});

let llamadas: string[] = [];

function simular(
  estado: string,
  redes?: { nombre: string; interfaz: string; categoria: string }[],
  ayudante = true,
) {
  llamadas = [];
  setTransportMock(async (cmd) => {
    llamadas.push(cmd);
    const ok = (value: unknown) => ({ ok: true, value });
    switch (cmd) {
      case "firewall_rules_status":
        return ok(informe(estado, redes, ayudante));
      case "app_get_snapshot":
        return ok({
          revision: 1,
          appVersion: "0.1.0",
          locale: "es",
          theme: "dark",
          instanceId: "x",
          instanceName: "PC",
          isSessionActive: false,
          activeSessionId: null,
          peersCount: 0,
        });
      case "settings_get":
        return ok(PREFS);
      case "peers_list":
      case "peers_discovered_list":
      case "session_incoming_list":
      case "peers_pairing_incoming_list":
        return ok([]);
      default:
        return ok(null);
    }
  });
}

beforeEach(() => router.navigate("inicio"));
afterEach(() => {
  setTransportMock(null);
  setLocale("es");
});

describe("App: comprobación del cortafuegos al arrancar", () => {
  it("comprueba al arrancar y, si faltan reglas, avisa sin elevar nada", async () => {
    simular("missing");
    render(App);

    const aviso = await screen.findByTestId("fw-startup-notice", {}, { timeout: 4000 });
    expect(aviso.textContent).toContain("Faltan reglas del cortafuegos de Windows");
    expect(llamadas).toContain("firewall_rules_status");
    // Solo se ha leído: crear pide UAC y únicamente sale al pulsar el botón.
    expect(llamadas).not.toContain("firewall_rules_create");
  });

  it("con todo en orden no muestra ningún aviso", async () => {
    simular("present");
    render(App);

    await waitFor(() => expect(llamadas).toContain("firewall_rules_status"));
    await new Promise((r) => setTimeout(r, 200));
    expect(screen.queryByTestId("fw-startup-notice")).toBeNull();
  });

  it("con una red pública ofrece las dos salidas y «Abrir configuración de red» llama a Windows", async () => {
    simular("present", [{ nombre: "Red", interfaz: "Ethernet", categoria: "Public" }]);
    render(App);

    await fireEvent.click(
      await screen.findByRole("button", { name: "Abrir configuración de red" }, { timeout: 4000 }),
    );
    expect(screen.getByRole("button", { name: "Permitir en redes públicas" })).toBeTruthy();
    await waitFor(() => expect(llamadas).toContain("firewall_open_network_settings"));
  });

  it("sin el ayudante elevado, «Revisar» lleva a Ajustes, pestaña Cortafuegos", async () => {
    simular("missing", undefined, false);
    render(App);

    await fireEvent.click(
      await screen.findByRole("button", { name: "Revisar" }, { timeout: 4000 }),
    );

    const pestana = await screen.findByRole("tab", { name: /Cortafuegos/ });
    await waitFor(() => expect(pestana.getAttribute("aria-selected")).toBe("true"));
  });
});
