import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { setTransportMock } from "../../lib/api/transport";
import { PeersModel, splitAddress } from "./model.svelte";
import type { Peer } from "../../lib/contracts/peer";

function peer(overrides: Partial<Peer> = {}): Peer {
  return {
    instanceId: "11111111-1111-4111-8111-111111111111",
    displayName: "Equipo guardado",
    fingerprint: "a".repeat(64),
    addresses: ["192.168.1.10:7411"],
    trustState: "trusted",
    autoAccept: false,
    lastSeen: "2026-09-25T10:00:00.000Z",
    ...overrides,
  };
}

describe("splitAddress", () => {
  it("separa host y puerto de una dirección IPv4", () => {
    expect(splitAddress("192.168.1.10:7411")).toEqual({ host: "192.168.1.10", port: 7411 });
  });

  it("separa host y puerto de una dirección IPv6 entre corchetes", () => {
    expect(splitAddress("[fd00::1]:7411")).toEqual({ host: "fd00::1", port: 7411 });
  });

  it("rechaza un puerto fuera de rango o una dirección sin puerto", () => {
    expect(splitAddress("192.168.1.10:70000")).toBeNull();
    expect(splitAddress("192.168.1.10")).toBeNull();
  });
});

describe("PeersModel", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    setTransportMock(null);
    vi.useRealTimers();
  });

  function mockCommands(handlers: Record<string, (args?: unknown) => unknown>) {
    setTransportMock(async (cmd, args) => {
      const handler = handlers[cmd];
      if (!handler) throw new Error(`comando no simulado: ${cmd}`);
      return { ok: true, value: handler(args) };
    });
  }

  it("fusiona un equipo descubierto con su confianza guardada por id + huella", async () => {
    const guardado = peer({ trustState: "trustedAutoAccept", alias: "Portátil" });
    mockCommands({
      peers_list: () => [guardado],
      peers_discovered_list: () => [
        {
          instanceId: guardado.instanceId,
          fingerprintDeclarada: guardado.fingerprint.toUpperCase(),
          displayName: "Nombre anunciado distinto",
          addresses: ["192.168.1.10:7411"],
          protocolVersion: 1,
          appVersion: "0.1.0",
          linkMbps: 1000,
          busy: false,
        },
      ],
    });

    const model = new PeersModel();
    await model.refresh();

    expect(model.peers).toHaveLength(1);
    const fusionado = model.peers[0]!;
    expect(fusionado.trustState).toBe("trustedAutoAccept");
    expect(fusionado.alias).toBe("Portátil");
    expect(model.statusOf(fusionado)).toEqual({ availability: "available", compatible: true });
  });

  it("un anuncio con huella distinta a la guardada no hereda la confianza (FR-011)", async () => {
    const guardado = peer({ trustState: "trusted" });
    mockCommands({
      peers_list: () => [guardado],
      peers_discovered_list: () => [
        {
          instanceId: guardado.instanceId,
          fingerprintDeclarada: "b".repeat(64),
          displayName: "Impostor",
          addresses: ["192.168.1.99:7411"],
          protocolVersion: 1,
          appVersion: "0.1.0",
          linkMbps: 100,
          busy: false,
        },
      ],
    });

    const model = new PeersModel();
    await model.refresh();

    const impostor = model.peers.find((p) => p.displayName === "Impostor");
    expect(impostor?.trustState).toBe("unknown");
    // El equipo guardado, al no anunciarse con su huella real, sigue apareciendo aparte.
    expect(model.peers).toHaveLength(2);
  });

  it("marca ocupado o incompatible según lo que declara el anuncio", async () => {
    const guardado = peer();
    mockCommands({
      peers_list: () => [guardado],
      peers_discovered_list: () => [
        {
          instanceId: guardado.instanceId,
          fingerprintDeclarada: guardado.fingerprint,
          displayName: guardado.displayName,
          addresses: guardado.addresses,
          protocolVersion: 99,
          appVersion: "9.9.9",
          linkMbps: 0,
          busy: true,
        },
      ],
    });

    const model = new PeersModel();
    await model.refresh();

    expect(model.statusOf(model.peers[0]!)).toEqual({ availability: "busy", compatible: false });
  });

  it("un error de refresco se traduce y no rompe el modelo", async () => {
    setTransportMock(async () => {
      throw new Error("sin red");
    });
    const model = new PeersModel();
    await model.refresh();
    expect(model.error).toBeTruthy();
    model.clearError();
    expect(model.error).toBeNull();
  });

  it("iniciar el emparejamiento arranca el reloj de caducidad", async () => {
    mockCommands({
      peers_pairing_start: () => ({
        pairingId: "22222222-2222-4222-8222-222222222222",
        verificationCode: "123456",
        peer: peer({ trustState: "unknown" }),
      }),
    });

    const model = new PeersModel();
    await model.beginPairing(peer({ trustState: "unknown" }));

    expect(model.pairing?.secondsLeft).toBe(60);
    await vi.advanceTimersByTimeAsync(1000);
    expect(model.pairing?.secondsLeft).toBe(59);
  });

  it("un equipo sin dirección válida no intenta emparejar", async () => {
    const model = new PeersModel();
    await model.beginPairing(peer({ addresses: [] }));
    expect(model.pairing).toBeNull();
    expect(model.error).toBeTruthy();
  });
});
