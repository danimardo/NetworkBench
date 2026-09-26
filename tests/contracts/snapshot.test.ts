import { describe, it, expect, beforeEach, vi } from "vitest";
import { SnapshotStore, appSnapshotSchema } from "../../src/lib/api/snapshot.svelte";
import { setTransportMock } from "../../src/lib/api/transport";

describe("Atomic Subscription & Snapshot Contract Tests (contracts/ipc.md)", () => {
  beforeEach(() => {
    setTransportMock(null);
    vi.restoreAllMocks();
  });

  it("valida el esquema de AppSnapshot", () => {
    const valid = {
      revision: 10,
      appVersion: "0.1.0",
      locale: "es",
      theme: "dark",
      instanceId: "inst-123",
      instanceName: "Local PC",
      isSessionActive: false,
      peersCount: 3,
    };
    expect(appSnapshotSchema.parse(valid)).toEqual(valid);

    // Revisión negativa debe fallar
    expect(() => appSnapshotSchema.parse({ ...valid, revision: -1 })).toThrow();
  });

  it("descarta eventos obsoletos con revision <= snapshot.revision", async () => {
    const currentRev = 10;
    setTransportMock(async () => ({
      ok: true,
      value: {
        revision: currentRev,
        appVersion: "0.1.0",
        locale: "es",
        theme: "dark",
        instanceId: "inst-1",
        instanceName: "PC",
        isSessionActive: false,
        peersCount: 1,
      },
    }));

    const store = new SnapshotStore();
    await store.init();

    expect(store.snapshot?.revision).toBe(10);

    // Evento anterior (rev 8) debe descartarse
    const resOld = await store.handleIncomingRevision(8);
    expect(resOld).toBe("discarded");
    expect(store.snapshot?.revision).toBe(10);

    // Evento con misma revisión (rev 10) debe descartarse
    const resSame = await store.handleIncomingRevision(10);
    expect(resSame).toBe("discarded");
    expect(store.snapshot?.revision).toBe(10);
  });

  it("aplica eventos secuenciales en orden estricto (rev + 1)", async () => {
    setTransportMock(async () => ({
      ok: true,
      value: {
        revision: 10,
        appVersion: "0.1.0",
        locale: "es",
        theme: "dark",
        instanceId: "inst-1",
        instanceName: "PC",
        isSessionActive: false,
        peersCount: 1,
      },
    }));

    const store = new SnapshotStore();
    await store.init();

    const resNext = await store.handleIncomingRevision(11, (curr) => ({
      ...curr,
      peersCount: 2,
    }));

    expect(resNext).toBe("applied");
    expect(store.snapshot?.revision).toBe(11);
    expect(store.snapshot?.peersCount).toBe(2);
  });

  it("detecta huecos de revisión y solicita recuperación de snapshot completo", async () => {
    let mockRevision = 10;
    setTransportMock(async (cmd) => {
      if (cmd === "app_get_snapshot") {
        return {
          ok: true,
          value: {
            revision: mockRevision,
            appVersion: "0.1.0",
            locale: "es",
            theme: "dark",
            instanceId: "inst-1",
            instanceName: "PC",
            isSessionActive: true,
            peersCount: 4,
          },
        };
      }
      return { ok: true, value: null };
    });

    const store = new SnapshotStore();
    await store.init();
    expect(store.snapshot?.revision).toBe(10);

    // Simulamos que el backend avanzó a revisión 15 (hueco irrecuperable de eventos 11, 12, 13, 14)
    mockRevision = 15;
    const resGap = await store.handleIncomingRevision(15);

    expect(resGap).toBe("refreshed");
    // Snapshot recuperado con datos autoritativos íntegros
    expect(store.snapshot?.revision).toBe(15);
    expect(store.snapshot?.isSessionActive).toBe(true);
    expect(store.snapshot?.peersCount).toBe(4);
  });

  describe("evento app://snapshot-changed (T150)", () => {
    const base = {
      revision: 10,
      appVersion: "0.1.0",
      locale: "es",
      theme: "dark",
      instanceId: "inst-1",
      instanceName: "PC",
      isSessionActive: false,
      peersCount: 1,
    };

    async function storeConSnapshot() {
      setTransportMock(async () => ({ ok: true, value: base }));
      const store = new SnapshotStore();
      await store.init();
      return store;
    }

    it("sustituye el snapshot si la revisión es más nueva", async () => {
      const store = await storeConSnapshot();
      const resultado = store.aplicarEvento({
        ...base,
        revision: 11,
        isSessionActive: true,
        activeSessionId: "s-1",
      });
      expect(resultado).toBe("applied");
      expect(store.snapshot?.revision).toBe(11);
      expect(store.snapshot?.isSessionActive).toBe(true);
      expect(store.snapshot?.activeSessionId).toBe("s-1");
    });

    it("no salta un hueco: el evento lleva el estado completo, así que basta aplicarlo", async () => {
      const store = await storeConSnapshot();
      expect(store.aplicarEvento({ ...base, revision: 15, peersCount: 4 })).toBe("applied");
      expect(store.snapshot?.peersCount).toBe(4);
    });

    it("descarta un evento obsoleto o repetido sin rebobinar la interfaz", async () => {
      const store = await storeConSnapshot();
      expect(store.aplicarEvento({ ...base, revision: 9, peersCount: 99 })).toBe("discarded");
      expect(store.aplicarEvento({ ...base, revision: 10, peersCount: 99 })).toBe("discarded");
      expect(store.snapshot?.peersCount).toBe(1);
    });

    it("ignora un payload malformado", async () => {
      const store = await storeConSnapshot();
      expect(store.aplicarEvento({ revision: "11" })).toBe("invalid");
      expect(store.aplicarEvento(null)).toBe("invalid");
      expect(store.snapshot?.revision).toBe(10);
    });

    it("un evento previo al snapshot inicial se acepta y el inicial más viejo no lo pisa", async () => {
      setTransportMock(async () => ({ ok: true, value: base }));
      const store = new SnapshotStore();
      expect(store.aplicarEvento({ ...base, revision: 12 })).toBe("applied");
      await store.init();
      expect(store.snapshot?.revision).toBe(12);
    });
  });
});
