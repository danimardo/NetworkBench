import { describe, it, expect, afterEach } from "vitest";
import { setTransportMock, invokeCommand } from "./transport";
import { updateSettings, setAutostart } from "./settings";
import { manualConnectPeer } from "./peers";
import { PreferencesSchema } from "../contracts/settings";

// Tauri casa las claves del objeto de argumentos con los NOMBRES DE LOS PARÁMETROS de la
// función Rust (en camelCase). El transporte envolvía todo en `{ request: … }`, así que
// cualquier comando con parámetros (guardar un ajuste, conectar por IP…) fallaba en la app
// real con `NB-INTERNAL-001`, y ningún test lo veía: los mocks devolvían lo que se les
// pedía sin mirar la forma de los argumentos. `setTransportMock` recibe aquí exactamente lo
// que recibiría `invoke` de Tauri.

const PREFS = PreferencesSchema.parse({
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
});

afterEach(() => setTransportMock(null));

describe("transporte IPC: los argumentos llegan a Tauri tal cual, sin envoltorio", () => {
  it("no envuelve el objeto de argumentos en { request: … }", async () => {
    let recibido: Record<string, unknown> | undefined;
    setTransportMock(async (_cmd, args) => {
      recibido = args;
      return { ok: true, value: null };
    });

    await invokeCommand("cualquier_comando", { host: "10.0.0.2", port: 7411 });

    expect(recibido).toEqual({ host: "10.0.0.2", port: 7411 });
    expect(recibido).not.toHaveProperty("request");
  });

  it("sin argumentos, no manda nada", async () => {
    let recibido: unknown = "sin llamar";
    setTransportMock(async (_cmd, args) => {
      recibido = args;
      return { ok: true, value: null };
    });
    await invokeCommand("otro_comando");
    expect(recibido).toBeUndefined();
  });

  it("guardar ajustes envía `preferences` (el parámetro de settings_update)", async () => {
    let recibido: Record<string, unknown> | undefined;
    setTransportMock(async (_cmd, args) => {
      recibido = args;
      return { ok: true, value: PREFS };
    });
    await updateSettings({ ...PREFS, autoAcceptTrusted: true });

    expect(Object.keys(recibido ?? {})).toEqual(["preferences"]);
    expect((recibido!.preferences as { autoAcceptTrusted: boolean }).autoAcceptTrusted).toBe(true);
  });

  it("el autoarranque envía `enabled` (el parámetro de settings_autostart_set)", async () => {
    let recibido: Record<string, unknown> | undefined;
    setTransportMock(async (_cmd, args) => {
      recibido = args;
      return { ok: true, value: null };
    });
    await setAutostart(true);
    expect(recibido).toEqual({ enabled: true });
  });

  it("la conexión manual envía `host` y `port` (los parámetros de peers_manual_connect)", async () => {
    let recibido: Record<string, unknown> | undefined;
    setTransportMock(async (_cmd, args) => {
      recibido = args;
      throw new Error("no importa la respuesta");
    });
    await manualConnectPeer("192.168.1.50", 7411).catch(() => {});
    expect(recibido).toEqual({ host: "192.168.1.50", port: 7411 });
  });
});
