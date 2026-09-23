import { z } from "zod";
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
      // 1. Obtener snapshot inicial
      const data = await invokeCommand<AppSnapshot>(
        "app.getSnapshot",
        undefined,
        appSnapshotSchema,
      );
      this.snapshot = data;
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
        "app.getSnapshot",
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
