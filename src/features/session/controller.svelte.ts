import { cancelSession, getSessionState, startSession } from "../../lib/api/session";
import { getHistoryDetail } from "../../lib/api/history";
import { IpcError } from "../../lib/api/transport";
import { onSampleBatch, type SamplePoint } from "../../lib/api/samples";
import { benchmarkPlanSchema, type BenchmarkPlan } from "../../lib/contracts/plan";
import type { Peer } from "../../lib/contracts/peer";
import { sessionResultSchema, type SessionResult } from "../../lib/contracts/session-result";
import { logger } from "../../lib/logging";
import { t } from "../../lib/i18n";
import type { ExecutionPhase } from "./SessionScreen.svelte";

const POLL_MS = 500;
const RESULT_RETRIES = 5;

/** Plan estándar: TCP, un flujo, la misma forma que `BenchmarkPlan::new_standard_tcp`. */
export function standardPlan(): BenchmarkPlan {
  return benchmarkPlanSchema.parse({
    protocol: "tcp",
    direction: "forward",
    streams: 1,
    warmupSeconds: 1,
    measureSeconds: 10,
    cooldownSeconds: 1,
    port: 5001,
  });
}

/** Traduce el estado de la máquina de Rust (`SCREAMING_SNAKE_CASE`) a la fase de la pantalla. */
export function phaseOf(state: string): ExecutionPhase | "idle" {
  switch (state) {
    case "CONNECTING":
    case "HELLO_PENDING":
    case "PAIRING":
    case "REQUESTING":
    case "PREPARING":
      return "preparing";
    case "RUNNING_SEND":
    case "RUNNING_BOTH":
      return "runningSend";
    case "RUNNING_RECEIVE":
      return "runningReceive";
    case "ANALYZING":
      return "analyzing";
    case "COMPLETED":
      return "completed";
    case "CANCELLING":
      return "cancelling";
    case "CANCELLED":
      return "cancelled";
    case "FAILED":
      return "failed";
    default:
      return "idle";
  }
}

function esperar(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Lleva una medición de principio a fin desde la interfaz: la lanza, sigue su estado y, al
 * terminar bien, carga el resultado **guardado** por el backend (la interfaz no fabrica
 * ningún resultado).
 *
 * El estado lo decide Rust; aquí solo se consulta cada 500 ms mientras dura la sesión.
 * El porcentaje es una **estimación por tiempo transcurrido**, no un dato del motor.
 */
export class SessionController {
  peer = $state<Peer | null>(null);
  plan = $state<BenchmarkPlan>(standardPlan());
  phase = $state<ExecutionPhase | "idle">("idle");
  sessionId = $state<string | null>(null);
  progressPercent = $state(0);
  currentBps = $state("0");
  errorMessage = $state("");
  result = $state<SessionResult | null>(null);
  samplesForward = $state<SamplePoint[]>([]);
  samplesReverse = $state<SamplePoint[]>([]);

  private timer: ReturnType<typeof setInterval> | null = null;
  private unlisten: Awaited<ReturnType<typeof onSampleBatch>> | null = null;
  private startedAt = 0;

  get active(): boolean {
    return this.phase !== "idle" && !this.finished;
  }

  get finished(): boolean {
    return this.phase === "completed" || this.phase === "cancelled" || this.phase === "failed";
  }

  async begin(peer: Peer): Promise<boolean> {
    this.reset();
    this.peer = peer;
    this.plan = standardPlan();
    this.phase = "preparing";
    try {
      await this.subscribeSamples();
      this.sessionId = await startSession(peer, this.plan);
    } catch (err) {
      this.phase = "failed";
      this.errorMessage =
        err instanceof IpcError ? t(err.appError.messageKey) : t("session.errors.start");
      return false;
    }
    this.startedAt = Date.now();
    this.timer = setInterval(() => void this.poll(), POLL_MS);
    return true;
  }

  async cancel(): Promise<void> {
    if (!this.sessionId || this.finished) return;
    this.phase = "cancelling";
    try {
      await cancelSession(this.sessionId);
    } catch (err) {
      this.errorMessage =
        err instanceof IpcError ? t(err.appError.messageKey) : t("session.errors.cancel");
    }
  }

  /** Vuelve a «sin sesión»; no toca lo guardado. */
  reset(): void {
    this.stopPolling();
    void this.unsubscribeSamples();
    this.peer = null;
    this.phase = "idle";
    this.sessionId = null;
    this.progressPercent = 0;
    this.currentBps = "0";
    this.errorMessage = "";
    this.result = null;
    this.samplesForward = [];
    this.samplesReverse = [];
  }

  private async poll(): Promise<void> {
    let state: string;
    try {
      state = await getSessionState();
    } catch {
      return;
    }
    const fase = phaseOf(state);
    if (fase === "idle") return;
    this.phase = fase;

    const total =
      2 * (this.plan.warmupSeconds + this.plan.measureSeconds + this.plan.cooldownSeconds);
    const transcurrido = (Date.now() - this.startedAt) / 1000;
    this.progressPercent = this.finished
      ? 100
      : Math.min(99, Math.round((transcurrido / total) * 100));

    if (!this.finished) return;
    this.stopPolling();
    void this.unsubscribeSamples();
    if (fase === "completed") {
      await this.loadResult();
    } else if (fase === "failed") {
      this.errorMessage = t("session.errors.failed");
    }
  }

  /** El resultado se guarda al terminar; puede tardar un instante en estar en la base. */
  private async loadResult(): Promise<void> {
    if (!this.sessionId) return;
    for (let intento = 0; intento < RESULT_RETRIES; intento++) {
      try {
        const registro = await getHistoryDetail(this.sessionId);
        if (registro?.resultJson) {
          this.result = sessionResultSchema.parse(JSON.parse(registro.resultJson));
          return;
        }
      } catch (err) {
        logger.warn({
          module: "session",
          eventCode: "RESULT_LOAD_FAILED",
          message: `No se pudo leer el resultado guardado: ${String(err)}`,
        });
      }
      await esperar(300);
    }
    this.errorMessage = t("session.errors.noResult");
  }

  private async subscribeSamples(): Promise<void> {
    try {
      this.unlisten = await onSampleBatch((lote) => {
        if (lote.sessionId !== this.sessionId) return;
        this.currentBps = String(lote.latestBps);
        const destino = lote.direction === "reverse" ? "samplesReverse" : "samplesForward";
        this[destino] = [...this[destino], ...lote.samples];
      });
    } catch {
      // Sin Tauri (pruebas, navegador) no hay muestras en vivo; la medición sigue igual.
    }
  }

  private async unsubscribeSamples(): Promise<void> {
    this.unlisten?.();
    this.unlisten = null;
  }

  private stopPolling(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }
}
