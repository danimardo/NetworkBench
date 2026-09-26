import { describe, it, expect, afterEach } from "vitest";
import { setTransportMock } from "../../lib/api/transport";
import { ConsentModel } from "./model.svelte";
import type { Peer } from "../../lib/contracts/peer";

function peer(): Peer {
  return {
    instanceId: "11111111-1111-4111-8111-111111111111",
    displayName: "Equipo remoto",
    fingerprint: "a".repeat(64),
    addresses: ["192.168.1.10:7411"],
    trustState: "trusted",
    autoAccept: false,
    lastSeen: "2026-09-25T10:00:00.000Z",
  };
}

function mockCommands(handlers: Record<string, (args?: unknown) => unknown>) {
  setTransportMock(async (cmd, args) => {
    const handler = handlers[cmd];
    if (!handler) throw new Error(`comando no simulado: ${cmd}`);
    return { ok: true, value: handler(args) };
  });
}

describe("ConsentModel", () => {
  afterEach(() => {
    setTransportMock(null);
  });

  it("sin nada pendiente, current es null", async () => {
    mockCommands({
      session_incoming_list: () => [],
      peers_pairing_incoming_list: () => [],
    });
    const model = new ConsentModel();
    await model.refresh();
    expect(model.current).toBeNull();
  });

  it("una solicitud de sesión se antepone a un emparejamiento pendiente", async () => {
    mockCommands({
      session_incoming_list: () => [
        {
          requestId: "22222222-2222-4222-8222-222222222222",
          peer: peer(),
          plan: {
            protocol: "tcp",
            direction: "forward",
            streams: 1,
            warmupSeconds: 1,
            measureSeconds: 10,
            cooldownSeconds: 1,
            port: 5001,
          },
        },
      ],
      peers_pairing_incoming_list: () => [
        {
          pairingId: "33333333-3333-4333-8333-333333333333",
          verificationCode: "123456",
          peer: peer(),
        },
      ],
    });
    const model = new ConsentModel();
    await model.refresh();
    expect(model.current?.kind).toBe("session");
  });

  it("aceptar una solicitud de sesión la retira de la lista", async () => {
    mockCommands({
      session_incoming_list: () => [
        {
          requestId: "22222222-2222-4222-8222-222222222222",
          peer: peer(),
          plan: {
            protocol: "tcp",
            direction: "forward",
            streams: 1,
            warmupSeconds: 1,
            measureSeconds: 10,
            cooldownSeconds: 1,
            port: 5001,
          },
        },
      ],
      peers_pairing_incoming_list: () => [],
      session_incoming_respond: () => null,
    });
    const model = new ConsentModel();
    await model.refresh();
    expect(model.current).not.toBeNull();

    await model.respond(true);
    expect(model.current).toBeNull();
    expect(model.sessionRequests).toHaveLength(0);
  });

  it("un error al responder se refleja sin vaciar la solicitud pendiente", async () => {
    mockCommands({
      session_incoming_list: () => [
        {
          requestId: "22222222-2222-4222-8222-222222222222",
          peer: peer(),
          plan: {
            protocol: "tcp",
            direction: "forward",
            streams: 1,
            warmupSeconds: 1,
            measureSeconds: 10,
            cooldownSeconds: 1,
            port: 5001,
          },
        },
      ],
      peers_pairing_incoming_list: () => [],
    });
    const model = new ConsentModel();
    await model.refresh();

    setTransportMock(async () => {
      throw new Error("canal perdido");
    });
    await model.respond(true);
    expect(model.error).toBeTruthy();
    expect(model.current).not.toBeNull();
  });
});
