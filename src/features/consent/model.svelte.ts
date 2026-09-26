import {
  listIncomingPairingRequests,
  listIncomingSessionRequests,
  onIncomingPairingRequest,
  onIncomingSessionRequest,
  respondIncomingPairing,
  respondIncomingSession,
  type IncomingPairingRequest,
  type IncomingSessionRequest,
} from "../../lib/api/consent";
import { logger } from "../../lib/logging";

const POLL_MS = 3000;

export type IncomingRequest =
  | { kind: "session"; request: IncomingSessionRequest }
  | { kind: "pairing"; request: IncomingPairingRequest };

/**
 * Solicitudes entrantes que esperan una decisión humana (T177/T182, FR-012/FR-016): otro
 * equipo pide medir o emparejarse con este.
 *
 * El backend avisa por evento en cuanto llega una, pero un evento se pudo perder si la
 * ventana no estaba lista; por eso también se sondea la lista pendiente periódicamente,
 * igual que hace `PeersModel` con los equipos descubiertos.
 */
export class ConsentModel {
  sessionRequests = $state<IncomingSessionRequest[]>([]);
  pairingRequests = $state<IncomingPairingRequest[]>([]);
  busy = $state(false);
  error = $state<string | null>(null);

  private pollTimer: ReturnType<typeof setInterval> | null = null;
  private unlistenSession: Awaited<ReturnType<typeof onIncomingSessionRequest>> | null = null;
  private unlistenPairing: Awaited<ReturnType<typeof onIncomingPairingRequest>> | null = null;

  /** La más antigua primero; una solicitud de sesión se muestra antes que un emparejamiento. */
  current = $derived.by<IncomingRequest | null>(() => {
    if (this.sessionRequests[0]) return { kind: "session", request: this.sessionRequests[0] };
    if (this.pairingRequests[0]) return { kind: "pairing", request: this.pairingRequests[0] };
    return null;
  });

  async start(): Promise<void> {
    if (this.pollTimer) return;
    await this.refresh();
    this.pollTimer = setInterval(() => void this.refresh(), POLL_MS);
    try {
      this.unlistenSession = await onIncomingSessionRequest(() => void this.refresh());
      this.unlistenPairing = await onIncomingPairingRequest(() => void this.refresh());
    } catch {
      // Sin Tauri (pruebas, navegador): el sondeo periódico sigue cubriendo el caso.
    }
  }

  stop(): void {
    if (this.pollTimer) clearInterval(this.pollTimer);
    this.pollTimer = null;
    this.unlistenSession?.();
    this.unlistenPairing?.();
    this.unlistenSession = null;
    this.unlistenPairing = null;
  }

  async refresh(): Promise<void> {
    try {
      const [sesiones, emparejamientos] = await Promise.all([
        listIncomingSessionRequests(),
        listIncomingPairingRequests(),
      ]);
      this.sessionRequests = sesiones;
      this.pairingRequests = emparejamientos;
    } catch (err) {
      logger.warn({
        module: "consent",
        eventCode: "CONSENT_REFRESH_FAILED",
        message: `No se pudo actualizar las solicitudes entrantes: ${String(err)}`,
      });
    }
  }

  async respond(accepted: boolean): Promise<void> {
    const actual = this.current;
    if (!actual) return;
    this.busy = true;
    this.error = null;
    try {
      if (actual.kind === "session") {
        await respondIncomingSession(actual.request.requestId, accepted);
        this.sessionRequests = this.sessionRequests.filter(
          (r) => r.requestId !== actual.request.requestId,
        );
      } else {
        await respondIncomingPairing(actual.request.pairingId, accepted);
        this.pairingRequests = this.pairingRequests.filter(
          (r) => r.pairingId !== actual.request.pairingId,
        );
      }
    } catch (err) {
      this.error = String(err);
    } finally {
      this.busy = false;
    }
  }
}
