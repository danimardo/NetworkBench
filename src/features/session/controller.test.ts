import { describe, it, expect, afterEach } from "vitest";
import { waitFor } from "@testing-library/svelte";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { setTransportMock } from "../../lib/api/transport";
import { SessionController, phaseOf } from "./controller.svelte";
import type { Peer } from "../../lib/contracts/peer";

// El resultado real que Rust serializa (T160): más fiable que reconstruirlo a mano.
const RESULT_FIXTURE = readFileSync(
  join(__dirname, "../../../tests/contracts/fixtures/rust/session-result-tcp.json"),
  "utf-8",
);

function peer(): Peer {
  return {
    instanceId: "11111111-1111-4111-8111-111111111111",
    displayName: "Equipo",
    fingerprint: "a".repeat(64),
    addresses: ["192.168.1.10:7411"],
    trustState: "trusted",
    autoAccept: false,
    lastSeen: "2026-09-25T10:00:00.000Z",
  };
}

describe("phaseOf", () => {
  it("traduce cada estado de la máquina de Rust a su fase de pantalla", () => {
    expect(phaseOf("PREPARING")).toBe("preparing");
    expect(phaseOf("RUNNING_SEND")).toBe("runningSend");
    expect(phaseOf("RUNNING_RECEIVE")).toBe("runningReceive");
    expect(phaseOf("COMPLETED")).toBe("completed");
    expect(phaseOf("CANCELLED")).toBe("cancelled");
    expect(phaseOf("FAILED")).toBe("failed");
  });

  it("un estado desconocido o Idle se trata como sin sesión", () => {
    expect(phaseOf("IDLE")).toBe("idle");
    expect(phaseOf("ALGO_INVENTADO")).toBe("idle");
  });
});

describe("SessionController", () => {
  afterEach(() => {
    setTransportMock(null);
  });

  function mockCommands(handlers: Record<string, (args?: unknown) => unknown>) {
    setTransportMock(async (cmd, args) => {
      const handler = handlers[cmd];
      if (!handler) throw new Error(`comando no simulado: ${cmd}`);
      return { ok: true, value: handler(args) };
    });
  }

  it("un fallo al arrancar la sesión se refleja como fase failed", async () => {
    setTransportMock(async () => {
      throw new Error("sin conexión");
    });
    const controller = new SessionController();
    const ok = await controller.begin(peer());

    expect(ok).toBe(false);
    expect(controller.phase).toBe("failed");
    expect(controller.errorMessage).toBeTruthy();
  });

  it("sondea el estado hasta que la sesión termina y entonces carga el resultado guardado", async () => {
    let estado = "PREPARING";
    mockCommands({
      session_start: () => ({ sessionId: "11111111-1111-4111-8111-111111111111" }),
      session_get_state: () => estado,
      history_get: () => ({
        id: "11111111-1111-4111-8111-111111111111",
        createdAt: "2026-09-25T10:00:00.000Z",
        status: "completed",
        protocol: "tcp",
        streams: 1,
        durationSeconds: 10,
        isPartial: false,
        samples: [],
        resultJson: RESULT_FIXTURE,
      }),
    });

    const controller = new SessionController();
    await controller.begin(peer());
    expect(controller.phase).toBe("preparing");

    estado = "COMPLETED";
    await waitFor(
      () => {
        expect(controller.phase).toBe("completed");
        expect(controller.result?.plan.protocol).toBe("tcp");
      },
      { timeout: 3000 },
    );
  });

  it("cancelar antes de terminar pasa a la fase cancelling", async () => {
    mockCommands({
      session_start: () => ({ sessionId: "22222222-2222-4222-8222-222222222222" }),
      session_get_state: () => "RUNNING_SEND",
      session_cancel: () => null,
    });
    const controller = new SessionController();
    await controller.begin(peer());
    await controller.cancel();
    expect(controller.phase).toBe("cancelling");
  });

  it("reset() vuelve a idle sin sesión activa", async () => {
    mockCommands({
      session_start: () => ({ sessionId: "33333333-3333-4333-8333-333333333333" }),
      session_get_state: () => "PREPARING",
    });
    const controller = new SessionController();
    await controller.begin(peer());
    controller.reset();
    expect(controller.phase).toBe("idle");
    expect(controller.peer).toBeNull();
    expect(controller.sessionId).toBeNull();
  });
});
