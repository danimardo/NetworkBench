import { z } from "zod";
import { listen } from "@tauri-apps/api/event";
import { invokeCommand, IpcError } from "./transport";
import { logger } from "../logging";

export const appSnapshotSchema = z.object({
  revision: z.number().int().nonnegative(),
  appVersion: z.string(),
  locale: z.string(),
  theme: z.string(),
  instanceId: z.string(),
  instanceName: z.string(),
  isSessionActive: z.boolean(),
  activeSessionId: z.string().nullable().optional(),
  peersCount: z.number().int().nonnegative(),
});

export type AppSnapshot = z.infer<typeof appSnapshotSchema>;

export const EVENTO_SNAPSHOT = "app://snapshot-changed";

export class SnapshotStore {
  snapshot = $state<AppSnapshot | null>(null);
  isLoading = $state<boolean>(true);
  error = $state<string | null>(null);

  private isSubscribed = false;
  private eventCleanup: (() => void) | null = null;

  async init(): Promise<void> {
    this.isLoading = true;
    this.error = null;

    try {
      // 1. Suscribirse antes de pedir el snapshot: un cambio entre las dos cosas no se
      // pierde (contracts/ipc.md). Sin `window.__TAURI__` (pruebas, navegador) no hay
      // eventos y el snapshot inicial sigue funcionando.
      await this.suscribir();

      // 2. Obtener snapshot inicial. Si un evento más reciente llegó mientras tanto,
      // se conserva ese.
      const data = await invokeCommand<AppSnapshot>(
        "app_get_snapshot",
        undefined,
        appSnapshotSchema,
      );
      if (!this.snapshot || data.revision > this.snapshot.revision) {
        this.snapshot = data;
      }
      this.isSubscribed = true;
    } catch (err) {
      const msg = err instanceof IpcError ? err.appError.messageKey : "Error al obtener snapshot";
      this.error = msg;
      logger.error({
        module: "snapshot",
        eventCode: "SNAPSHOT_FETCH_FAILED",
        message: msg,
      });
    } finally {
      this.isLoading = false;
    }
  }

  private async suscribir(): Promise<void> {
    if (this.eventCleanup) return;
    try {
      this.eventCleanup = await listen<unknown>(EVENTO_SNAPSHOT, (evento) => {
        this.aplicarEvento(evento.payload);
      });
    } catch {
      logger.warn({
        module: "snapshot",
        eventCode: "SNAPSHOT_SUBSCRIBE_FAILED",
        message: "No se pudo suscribir a los cambios del snapshot; solo habrá lectura inicial",
      });
    }
  }

  /**
   * El evento lleva el snapshot completo, así que es autosuficiente: se descarta si no es
   * más nuevo que el actual y, si lo es, lo sustituye sin reconstruir nada por partes.
   */
  aplicarEvento(payload: unknown): "applied" | "discarded" | "invalid" {
    const parsed = appSnapshotSchema.safeParse(payload);
    if (!parsed.success) {
      logger.warn({
        module: "snapshot",
        eventCode: "SNAPSHOT_EVENT_INVALID",
        message: "Evento de snapshot con forma inválida; se ignora",
      });
      return "invalid";
    }
    if (this.snapshot && parsed.data.revision <= this.snapshot.revision) {
      return "discarded";
    }
    this.snapshot = parsed.data;
    return "applied";
  }

  /**
   * Procesa la llegada de un evento con revisión monotónica.
   * - Si revision <= actual: descarte de evento obsoleto sin efectos.
   * - Si revision == actual + 1: evento secuencial en orden, se aplica mutate.
   * - Si revision > actual + 1: detección de hueco, solicita snapshot completo de recuperación.
   */
  async handleIncomingRevision(
    incomingRevision: number,
    mutate?: (current: AppSnapshot) => AppSnapshot,
  ): Promise<"applied" | "discarded" | "refreshed"> {
    if (!this.snapshot) {
      await this.refresh();
      return "refreshed";
    }

    if (incomingRevision <= this.snapshot.revision) {
      logger.debug({
        module: "snapshot",
        eventCode: "OBSOLETE_EVENT_DISCARDED",
        message: `Descarte de evento obsoleto con revisión ${incomingRevision} (actual ${this.snapshot.revision})`,
        safeParams: {
          incomingRev: String(incomingRevision),
          currentRev: String(this.snapshot.revision),
        },
      });
      return "discarded";
    }

    if (incomingRevision === this.snapshot.revision + 1) {
      if (mutate) {
        this.snapshot = mutate(this.snapshot);
      }
      this.snapshot.revision = incomingRevision;
      return "applied";
    }

    // Hueco irrecuperable: solicitar snapshot fresco para restaurar integridad
    logger.warn({
      module: "snapshot",
      eventCode: "REVISION_GAP_DETECTED",
      message: `Hueco detectado: revisión entrante ${incomingRevision}, actual ${this.snapshot.revision}. Refrescando snapshot.`,
      safeParams: {
        incomingRev: String(incomingRevision),
        currentRev: String(this.snapshot.revision),
      },
    });

    await this.refresh();
    return "refreshed";
  }

  async refresh(): Promise<void> {
    try {
      const fresh = await invokeCommand<AppSnapshot>(
        "app_get_snapshot",
        undefined,
        appSnapshotSchema,
      );
      this.snapshot = fresh;
      this.error = null;
    } catch {
      logger.error({
        module: "snapshot",
        eventCode: "SNAPSHOT_REFRESH_FAILED",
        message: "No se pudo refrescar el snapshot tras hueco de revisión",
      });
    }
  }

  destroy(): void {
    if (this.eventCleanup) {
      this.eventCleanup();
      this.eventCleanup = null;
    }
    this.isSubscribed = false;
  }
}

export const snapshotStore = new SnapshotStore();
