import log from "loglevel";

export type LogLevelName = "trace" | "debug" | "info" | "warn" | "error";

export interface LogEventParams {
  module: string;
  eventCode: string;
  message: string;
  errorCode?: string;
  durationMs?: number;
  diagnosticId?: string;
  safeParams?: Record<string, string>;
}

// Precedencia constitucional: Configuración explícita temporal > VITE_LOG_LEVEL > default ("warn")
const initialLevel =
  (import.meta.env.VITE_LOG_LEVEL as LogLevelName) || (import.meta.env.DEV ? "debug" : "warn");

// Configurar loglevel sin persistencia en localStorage/cookies para respetar precedencia
log.setLevel(initialLevel, false);

const madFmt = new Intl.DateTimeFormat("es-ES", {
  timeZone: "Europe/Madrid",
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
  fractionalSecondDigits: 3,
  hour12: false,
});

export function formatMadridTimestamp(d: Date = new Date()): string {
  return madFmt.format(d);
}

export function sanitizeParamKey(key: string): boolean {
  const lower = key.toLowerCase();
  return (
    !lower.includes("secret") &&
    !lower.includes("token") &&
    !lower.includes("password") &&
    !lower.includes("private") &&
    !lower.includes("sessionid") &&
    !lower.includes("fingerprint")
  );
}

export function sanitizeParams(params?: Record<string, string>): Record<string, string> {
  if (!params) return {};
  const cleaned: Record<string, string> = {};
  for (const [k, v] of Object.entries(params)) {
    if (sanitizeParamKey(k)) {
      cleaned[k] = String(v).slice(0, 512);
    } else {
      cleaned[k] = "[REDACTED]";
    }
  }
  return cleaned;
}

let isCapturingGlobals = false;
let bridgeTransport: ((event: LogEventParams & { level: LogLevelName }) => void) | null = null;

export const logger = {
  setLevel(level: LogLevelName) {
    log.setLevel(level, false);
  },

  getLevel(): LogLevelName {
    const num = log.getLevel();
    switch (num) {
      case 0:
        return "trace";
      case 1:
        return "debug";
      case 2:
        return "info";
      case 3:
        return "warn";
      case 4:
        return "error";
      default:
        return "info";
    }
  },

  setBridge(transport: (event: LogEventParams & { level: LogLevelName }) => void) {
    bridgeTransport = transport;
  },

  trace(params: LogEventParams) {
    if (log.getLevel() <= log.levels.TRACE) {
      this._emit("trace", params);
    }
  },

  debug(params: LogEventParams) {
    if (log.getLevel() <= log.levels.DEBUG) {
      this._emit("debug", params);
    }
  },

  info(params: LogEventParams) {
    if (log.getLevel() <= log.levels.INFO) {
      this._emit("info", params);
    }
  },

  warn(params: LogEventParams) {
    if (log.getLevel() <= log.levels.WARN) {
      this._emit("warn", params);
    }
  },

  error(params: LogEventParams) {
    if (log.getLevel() <= log.levels.ERROR) {
      this._emit("error", params);
    }
  },

  _emit(level: LogLevelName, params: LogEventParams) {
    const sanitized = {
      ...params,
      safeParams: sanitizeParams(params.safeParams),
      level,
    };

    // Formatear consola según nivel sin usar console.* en código cliente
    const prefix = `[${formatMadridTimestamp()}] [${level.toUpperCase()}] [${params.module}]`;
    log[level](`${prefix} ${params.eventCode}: ${params.message}`, sanitized.safeParams);

    // Reenvío al backend mediante puente acotado si está registrado
    if (bridgeTransport) {
      try {
        bridgeTransport(sanitized);
      } catch {
        // Ignorar fallos de sink para evitar recursión y bucles infinitos
      }
    }
  },

  installGlobalErrorCapture() {
    if (isCapturingGlobals || typeof window === "undefined") return;
    isCapturingGlobals = true;

    window.addEventListener("error", (ev) => {
      this.error({
        module: "frontend.global",
        eventCode: "unhandled_error",
        message: ev.message || "Error no controlado en frontend",
      });
    });

    window.addEventListener("unhandledrejection", (ev) => {
      this.error({
        module: "frontend.global",
        eventCode: "unhandled_rejection",
        message: String(ev.reason) || "Promesa rechazada no controlada",
      });
    });
  },
};
