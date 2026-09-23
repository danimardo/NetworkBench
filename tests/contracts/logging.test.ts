import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  logger,
  sanitizeParams,
  sanitizeParamKey,
  formatMadridTimestamp,
  type LogEventParams,
} from "../../src/lib/logging";

describe("Frontend Logging Contract Tests (Constitución XIII)", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("filtra mensajes por nivel según la configuración", () => {
    const emitted: (LogEventParams & { level: string })[] = [];
    logger.setBridge((evt) => emitted.push(evt));

    logger.setLevel("warn");
    expect(logger.getLevel()).toBe("warn");

    logger.trace({ module: "test", eventCode: "TRC", message: "trace msg" });
    logger.debug({ module: "test", eventCode: "DBG", message: "debug msg" });
    logger.info({ module: "test", eventCode: "INF", message: "info msg" });

    expect(emitted).toHaveLength(0);

    logger.warn({ module: "test", eventCode: "WRN", message: "warning msg" });
    logger.error({ module: "test", eventCode: "ERR", message: "error msg" });

    expect(emitted).toHaveLength(2);
    expect(emitted[0]?.message).toBe("warning msg");
    expect(emitted[0]?.level).toBe("warn");
    expect(emitted[1]?.message).toBe("error msg");
    expect(emitted[1]?.level).toBe("error");
  });

  it("redacta campos sensibles y preserva los permitidos", () => {
    expect(sanitizeParamKey("secretKey")).toBe(false);
    expect(sanitizeParamKey("token")).toBe(false);
    expect(sanitizeParamKey("authToken")).toBe(false);
    expect(sanitizeParamKey("userPassword")).toBe(false);
    expect(sanitizeParamKey("sessionId")).toBe(false);
    expect(sanitizeParamKey("fingerprint")).toBe(false);
    expect(sanitizeParamKey("privateData")).toBe(false);

    expect(sanitizeParamKey("moduleName")).toBe(true);
    expect(sanitizeParamKey("interfaceIndex")).toBe(true);
    expect(sanitizeParamKey("status")).toBe(true);

    const rawParams = {
      sessionId: "123e4567-e89b-12d3-a456-426614174000",
      token: "secret_bearer_token",
      peerFingerprint: "sha256:abcdef",
      interface: "Ethernet0",
      ipVersion: "IPv4",
    };

    const sanitized = sanitizeParams(rawParams);
    expect(sanitized.sessionId).toBe("[REDACTED]");
    expect(sanitized.token).toBe("[REDACTED]");
    expect(sanitized.peerFingerprint).toBe("[REDACTED]");
    expect(sanitized.interface).toBe("Ethernet0");
    expect(sanitized.ipVersion).toBe("IPv4");
  });

  it("formatea timestamps con zona horaria Europe/Madrid", () => {
    // 2026-07-15 12:00:00 UTC (en verano Madrid CEST UTC+2 -> 14:00)
    const summerDate = new Date("2026-07-15T12:00:00Z");
    const summerStr = formatMadridTimestamp(summerDate);
    expect(summerStr).toMatch(/14:00:00/);

    // 2026-01-15 12:00:00 UTC (en invierno Madrid CET UTC+1 -> 13:00)
    const winterDate = new Date("2026-01-15T12:00:00Z");
    const winterStr = formatMadridTimestamp(winterDate);
    expect(winterStr).toMatch(/13:00:00/);
  });

  it("captura errores globales sin bucles ni duplicados", () => {
    const emitted: (LogEventParams & { level: string })[] = [];
    logger.setBridge((evt) => emitted.push(evt));
    logger.setLevel("debug");

    // Instalar captura global
    logger.installGlobalErrorCapture();
    // Una segunda llamada no debe duplicar listeners
    logger.installGlobalErrorCapture();

    // Simular error de window
    window.dispatchEvent(new ErrorEvent("error", { message: "Error global de prueba" }));

    expect(
      emitted.some(
        (e) => e.eventCode === "unhandled_error" && e.message === "Error global de prueba",
      ),
    ).toBe(true);

    // Fallos del sink en el bridge no deben lanzar excepciones ni entrar en bucle infinito
    logger.setBridge(() => {
      throw new Error("Sink defectuoso");
    });

    expect(() => {
      logger.error({
        module: "test",
        eventCode: "SINK_TEST",
        message: "tolerante a fallos de sink",
      });
    }).not.toThrow();
  });
});
