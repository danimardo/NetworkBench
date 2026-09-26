import {
  confirmPairing as confirmPairingCommand,
  listDiscoveredPeers,
  listPeers,
  manualConnectPeer,
  startPairing,
  type DiscoveredPeer,
} from "../../lib/api/peers";
import { IpcError } from "../../lib/api/transport";
import type { Peer } from "../../lib/contracts/peer";
import { logger } from "../../lib/logging";
import { t } from "../../lib/i18n";

/** Versión del protocolo de control que habla esta aplicación (`control::server`). */
export const PROTOCOL_VERSION = 1;

const POLL_MS = 2000;
/** Sin ninguna respuesta mDNS pasado este tiempo, «Buscando» pasa a «Ninguno encontrado». */
const SCAN_TIMEOUT_MS = 5000;
const PAIRING_SECONDS = 60;

export interface PeerStatus {
  availability: "available" | "busy";
  compatible: boolean;
}

export interface PairingInProgress {
  pairingId: string;
  peer: Peer;
  code: string;
  secondsLeft: number;
}

/** Separa `host` y `puerto` de `ip:puerto` o `[ipv6]:puerto`. */
export function splitAddress(address: string): { host: string; port: number } | null {
  const v6 = /^\[(.+)\]:(\d{1,5})$/.exec(address);
  const v4 = /^([^:]+):(\d{1,5})$/.exec(address);
  const m = v6 ?? v4;
  if (!m?.[1] || !m[2]) return null;
  const port = Number(m[2]);
  return port >= 1 && port <= 65535 ? { host: m[1], port } : null;
}

function messageOf(err: unknown, fallbackKey: string): string {
  return t(err instanceof IpcError ? err.appError.messageKey : fallbackKey);
}

/**
 * Lo que muestra «Equipos disponibles»: los equipos vistos por mDNS junto con los ya
 * guardados.
 *
 * **La confianza que se muestra es una pista, no una garantía.** Un equipo descubierto solo
 * muestra la confianza guardada si coinciden su identificador Y su huella; aun así ambos
 * viajan por mDNS y cualquiera en la red puede anunciarlos. Lo que protege es que al
 * conectar se verifica el certificado TLS contra la huella guardada (FR-013), así que
 * pulsar una tarjeta falsificada acaba en un error, no en una medición contra el intruso.
 */
export class PeersModel {
  known = $state<Peer[]>([]);
  discovered = $state<DiscoveredPeer[]>([]);
  scanning = $state(true);
  error = $state<string | null>(null);
  busy = $state(false);
  pairing = $state<PairingInProgress | null>(null);

  private pollTimer: ReturnType<typeof setInterval> | null = null;
  private scanTimer: ReturnType<typeof setTimeout> | null = null;
  private pairingTimer: ReturnType<typeof setInterval> | null = null;

  /** Equipos a mostrar: primero los vistos ahora, después los guardados sin anuncio. */
  peers = $derived.by<Peer[]>(() => {
    // Marca de tiempo plana, no estado reactivo: un objeto Date no aporta nada aquí.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const now = new Date().toISOString();
    const usadas: string[] = [];

    const vistos = this.discovered.map((d): Peer => {
      const huella = d.fingerprintDeclarada.toLowerCase();
      const guardado = this.known.find(
        (k) => k.instanceId === d.instanceId && k.fingerprint === huella,
      );
      if (guardado) usadas.push(guardado.fingerprint);
      return {
        instanceId: d.instanceId,
        displayName: d.displayName,
        fingerprint: huella,
        addresses: d.addresses,
        trustState: guardado?.trustState ?? "unknown",
        autoAccept: guardado?.autoAccept ?? false,
        lastSeen: now,
        alias: guardado?.alias ?? null,
      };
    });

    // Guardado y sin anuncio ahora: sigue siendo seleccionable, la conexión decide.
    const guardados = this.known.filter((k) => !usadas.includes(k.fingerprint));
    return [...vistos, ...guardados];
  });

  statusOf = (peer: Peer): PeerStatus => {
    const d = this.discovered.find(
      (x) => x.fingerprintDeclarada.toLowerCase() === peer.fingerprint,
    );
    if (!d) return { availability: "available", compatible: true };
    return {
      availability: d.busy ? "busy" : "available",
      compatible: d.protocolVersion === PROTOCOL_VERSION,
    };
  };

  start(): void {
    if (this.pollTimer) return;
    this.scanning = true;
    void this.refresh();
    this.pollTimer = setInterval(() => void this.refresh(), POLL_MS);
    this.scanTimer = setTimeout(() => {
      this.scanning = false;
    }, SCAN_TIMEOUT_MS);
  }

  stop(): void {
    if (this.pollTimer) clearInterval(this.pollTimer);
    if (this.scanTimer) clearTimeout(this.scanTimer);
    this.pollTimer = null;
    this.scanTimer = null;
    this.stopPairingClock();
  }

  async refresh(): Promise<void> {
    try {
      const [known, discovered] = await Promise.all([listPeers(), listDiscoveredPeers()]);
      this.known = known;
      this.discovered = discovered;
      if (discovered.length > 0) this.scanning = false;
    } catch (err) {
      this.error = messageOf(err, "peers.errors.refresh");
      logger.warn({
        module: "peers",
        eventCode: "PEERS_REFRESH_FAILED",
        message: "No se pudo actualizar la lista de equipos",
      });
    }
  }

  clearError(): void {
    this.error = null;
  }

  async manualConnect(host: string, port: number): Promise<Peer | null> {
    this.error = null;
    this.busy = true;
    try {
      const peer = await manualConnectPeer(host, port);
      await this.refresh();
      return peer;
    } catch (err) {
      this.error = messageOf(err, "peers.errors.connect");
      return null;
    } finally {
      this.busy = false;
    }
  }

  /** ¿Se puede medir ya con este equipo, o hay que emparejarlo antes? */
  isTrusted(peer: Peer): boolean {
    return peer.trustState === "trusted" || peer.trustState === "trustedAutoAccept";
  }

  async beginPairing(peer: Peer): Promise<void> {
    this.error = null;
    const destino = peer.addresses.map(splitAddress).find((a) => a !== null);
    if (!destino) {
      this.error = t("peers.errors.noAddress");
      return;
    }
    this.busy = true;
    try {
      const inicio = await startPairing(destino.host, destino.port);
      this.pairing = {
        pairingId: inicio.pairingId,
        peer: inicio.peer,
        code: inicio.verificationCode,
        secondsLeft: PAIRING_SECONDS,
      };
      this.startPairingClock();
    } catch (err) {
      this.error = messageOf(err, "peers.errors.pairStart");
    } finally {
      this.busy = false;
    }
  }

  /** La persona confirma que los códigos coinciden en las dos pantallas. */
  async confirmPairing(): Promise<boolean> {
    const actual = this.pairing;
    if (!actual) return false;
    this.busy = true;
    try {
      const resultado = await confirmPairingCommand(actual.pairingId, true);
      this.closePairing();
      await this.refresh();
      if (!resultado.accepted) {
        this.error = t("peers.errors.pairRejected");
      }
      return resultado.accepted;
    } catch (err) {
      this.closePairing();
      this.error = messageOf(err, "peers.errors.pairFailed");
      return false;
    } finally {
      this.busy = false;
    }
  }

  async cancelPairing(): Promise<void> {
    const actual = this.pairing;
    if (!actual) return;
    this.closePairing();
    try {
      await confirmPairingCommand(actual.pairingId, false);
    } catch {
      // El canal puede haber caducado ya: cancelar es idempotente para la persona.
    }
  }

  private closePairing(): void {
    this.stopPairingClock();
    this.pairing = null;
  }

  private startPairingClock(): void {
    this.stopPairingClock();
    this.pairingTimer = setInterval(() => {
      const actual = this.pairing;
      if (!actual) return this.stopPairingClock();
      if (actual.secondsLeft <= 1) {
        this.error = t("peers.errors.pairExpired");
        void this.cancelPairing();
        return;
      }
      this.pairing = { ...actual, secondsLeft: actual.secondsLeft - 1 };
    }, 1000);
  }

  private stopPairingClock(): void {
    if (this.pairingTimer) clearInterval(this.pairingTimer);
    this.pairingTimer = null;
  }
}
