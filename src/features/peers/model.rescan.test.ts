import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { setTransportMock } from "../../lib/api/transport";
import { PeersModel } from "./model.svelte";

describe("PeersModel.rescan (Buscar de nuevo)", () => {
  const llamadas: string[] = [];

  beforeEach(() => {
    vi.useFakeTimers();
    llamadas.length = 0;
    setTransportMock(async (cmd) => {
      llamadas.push(cmd);
      if (cmd === "peers_rescan") return { ok: true, value: null };
      return { ok: true, value: [] };
    });
  });

  afterEach(() => {
    setTransportMock(null);
    vi.useRealTimers();
  });

  it("reinicia el descubrimiento en el backend y vuelve a mostrar «Buscando…» hasta el tope", async () => {
    const model = new PeersModel();
    model.start();
    await vi.advanceTimersByTimeAsync(5000);
    expect(model.scanning).toBe(false);

    const lanzada = model.rescan();
    expect(model.scanning).toBe(true);
    await lanzada;
    expect(llamadas).toContain("peers_rescan");

    await vi.advanceTimersByTimeAsync(5000);
    expect(model.scanning).toBe(false);
    model.stop();
  });

  it("no lanza otra búsqueda mientras ya está buscando", async () => {
    const model = new PeersModel();
    model.start(); // arranca en «buscando»
    await model.rescan();
    expect(llamadas.filter((c) => c === "peers_rescan")).toHaveLength(0);
    model.stop();
  });

  it("si el backend falla, muestra el error y no se queda buscando para siempre", async () => {
    setTransportMock(async (cmd) => {
      if (cmd === "peers_rescan") throw new Error("mdns caído");
      return { ok: true, value: [] };
    });
    const model = new PeersModel();
    model.start();
    await vi.advanceTimersByTimeAsync(5000);

    await model.rescan();
    expect(model.error).not.toBeNull();
    await vi.advanceTimersByTimeAsync(5000);
    expect(model.scanning).toBe(false);
    model.stop();
  });
});
