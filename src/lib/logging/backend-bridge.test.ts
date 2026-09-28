import { describe, it, expect, vi } from "vitest";

describe("installBackendLogBridge: conecta logger.setBridge con el backend", () => {
  it("un logger.error llega a logFrontendEvent con el mismo evento", async () => {
    const logFrontendEvent = vi.fn().mockResolvedValue(undefined);
    vi.doMock("../api/logging", () => ({ logFrontendEvent }));

    const { installBackendLogBridge } = await import("./backend-bridge");
    const { logger } = await import("./index");
    installBackendLogBridge();
    logger.error({ module: "x", eventCode: "y", message: "z", errorCode: "NB-INTERNAL-001" });

    expect(logFrontendEvent).toHaveBeenCalledTimes(1);
    expect(logFrontendEvent.mock.calls[0]![0]).toMatchObject({
      level: "error",
      module: "x",
      eventCode: "y",
      message: "z",
      errorCode: "NB-INTERNAL-001",
    });
    vi.doUnmock("../api/logging");
    vi.resetModules();
  });

  it("no lanza si el propio puente falla al enviar", async () => {
    vi.doMock("../api/logging", () => ({
      logFrontendEvent: vi.fn().mockRejectedValue(new Error("sin backend")),
    }));

    const { installBackendLogBridge } = await import("./backend-bridge");
    const { logger } = await import("./index");
    installBackendLogBridge();

    expect(() =>
      logger.warn({ module: "x", eventCode: "y", message: "sin backend" }),
    ).not.toThrow();
    vi.doUnmock("../api/logging");
    vi.resetModules();
  });
});
