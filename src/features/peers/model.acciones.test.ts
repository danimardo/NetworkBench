import { describe, it, expect, afterEach } from "vitest";
import { setTransportMock } from "../../lib/api/transport";
import { PeersModel } from "./model.svelte";
import type { Peer } from "../../lib/contracts/peer";

const guardado: Peer = {
  instanceId: "11111111-1111-4111-8111-111111111111",
  displayName: "WIN11D",
  fingerprint: "a".repeat(64),
  addresses: [],
  trustState: "trusted",
  autoAccept: false,
  favorite: false,
  lastSeen: "2026-09-30T10:00:00.000Z",
};

afterEach(() => setTransportMock(null));

/** Registra cada comando recibido y responde a `peers_list` con `lista`. */
function simular(lista: Peer[], alcance: { fingerprint: string; estado: string }[] = []) {
  const llamadas: { cmd: string; args: unknown }[] = [];
  setTransportMock(async (cmd, args) => {
    llamadas.push({ cmd, args });
    if (cmd === "peers_list") return { ok: true, value: lista };
    if (cmd === "peers_reachability_list") return { ok: true, value: alcance };
    if (cmd === "peers_discovered_list") return { ok: true, value: [] };
    return { ok: true, value: null };
  });
  return llamadas;
}

describe("PeersModel: acciones del menú contextual", () => {
  it("un equipo guardado y no anunciado se muestra sin direcciones (la pantalla no inventa una)", async () => {
    simular([guardado]);
    const model = new PeersModel();
    await model.refresh();

    expect(model.peers[0]!.addresses).toEqual([]);
  });

  it("quitar la confianza pide al backend confianza y autoaceptación apagadas", async () => {
    const llamadas = simular([guardado]);
    const model = new PeersModel();
    await model.refresh();

    await model.revokeTrust(model.peers[0]!);

    expect(llamadas.find((l) => l.cmd === "peers_set_trust")?.args).toEqual({
      fingerprint: guardado.fingerprint,
      isTrusted: false,
      autoAccept: false,
    });
  });

  it("el favorito se invierte respecto al valor actual", async () => {
    const llamadas = simular([guardado]);
    const model = new PeersModel();
    await model.refresh();

    await model.toggleFavorite(model.peers[0]!);

    expect(llamadas.find((l) => l.cmd === "peers_set_favorite")?.args).toEqual({
      fingerprint: guardado.fingerprint,
      favorite: true,
    });
  });

  it("eliminar olvida el equipo por su huella y vuelve a leer la lista", async () => {
    const llamadas = simular([guardado]);
    const model = new PeersModel();
    await model.refresh();
    const antes = llamadas.length;

    await model.forget(model.peers[0]!);

    expect(llamadas.find((l) => l.cmd === "peers_forget")?.args).toEqual({
      fingerprint: guardado.fingerprint,
    });
    expect(llamadas.slice(antes).some((l) => l.cmd === "peers_list")).toBe(true);
  });

  it("un equipo que solo se ve por mDNS (sin registro) no dispara ninguna acción", async () => {
    const llamadas = simular([]);
    const model = new PeersModel();
    await model.refresh();
    const solo = { ...guardado, trustState: "unknown" as const };

    await model.revokeTrust(solo);
    await model.forget(solo);
    await model.toggleFavorite(solo);

    expect(llamadas.filter((l) => /set_trust|forget|favorite/.test(l.cmd))).toEqual([]);
  });

  it("un equipo guardado que no se anuncia y aún no tiene comprobación se muestra «comprobando», no «disponible»", async () => {
    simular([guardado]);
    const model = new PeersModel();
    await model.refresh();

    expect(model.statusOf(model.peers[0]!).availability).toBe("checking");
  });

  it("refleja el resultado de la comprobación: alcanzable = disponible, inalcanzable = no accesible", async () => {
    const otro: Peer = {
      ...guardado,
      instanceId: "22222222-2222-4222-8222-222222222222",
      fingerprint: "b".repeat(64),
    };
    simular(
      [guardado, otro],
      [
        { fingerprint: guardado.fingerprint, estado: "reachable" },
        { fingerprint: otro.fingerprint, estado: "unreachable" },
      ],
    );
    const model = new PeersModel();
    await model.refresh();

    const estado = (fp: string) => model.statusOf(model.peers.find((p) => p.fingerprint === fp)!);
    expect(estado(guardado.fingerprint).availability).toBe("available");
    expect(estado(otro.fingerprint).availability).toBe("unreachable");
  });

  it("los equipos inalcanzables van al final de la lista", async () => {
    const caido: Peer = {
      ...guardado,
      instanceId: "22222222-2222-4222-8222-222222222222",
      fingerprint: "b".repeat(64),
      displayName: "Caído",
    };
    const vivo: Peer = {
      ...guardado,
      instanceId: "33333333-3333-4333-8333-333333333333",
      fingerprint: "c".repeat(64),
      displayName: "Vivo",
    };
    simular(
      [caido, vivo],
      [
        { fingerprint: caido.fingerprint, estado: "unreachable" },
        { fingerprint: vivo.fingerprint, estado: "reachable" },
      ],
    );
    const model = new PeersModel();
    await model.refresh();

    expect(model.peers.map((p) => p.displayName)).toEqual(["Vivo", "Caído"]);
  });

  it("si falla la lectura del alcance, la lista de equipos sigue apareciendo", async () => {
    setTransportMock(async (cmd) => {
      if (cmd === "peers_list") return { ok: true, value: [guardado] };
      if (cmd === "peers_discovered_list") return { ok: true, value: [] };
      throw new Error("sin comando");
    });
    const model = new PeersModel();
    await model.refresh();

    expect(model.peers).toHaveLength(1);
    expect(model.error).toBeNull();
  });

  it("«Comprobar ahora» pide al backend comprobar esa huella", async () => {
    const llamadas = simular([guardado]);
    const model = new PeersModel();
    await model.refresh();

    await model.checkNow(model.peers[0]!);

    expect(llamadas.find((l) => l.cmd === "peers_check_now")?.args).toEqual({
      fingerprint: guardado.fingerprint,
    });
  });
});
