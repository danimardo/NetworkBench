import { describe, it, expect, vi } from "vitest";

describe("logFrontendEvent: puente hacia el registro del backend", () => {
  it("invoca diagnostics_log_frontend_event con las claves en camelCase que espera Rust", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    vi.doMock("@tauri-apps/api/core", () => ({ invoke }));

    const { logFrontendEvent } = await import("./logging");
    await logFrontendEvent({
      level: "warn",
      module: "transport",
      eventCode: "IPC_ERROR",
      message: "algo falló",
      errorCode: "NB-FW-003",
      diagnosticId: "detalle",
      safeParams: { cmd: "x" },
    });

    expect(invoke).toHaveBeenCalledWith("diagnostics_log_frontend_event", {
      evento: {
        level: "warn",
        module: "transport",
        eventCode: "IPC_ERROR",
        message: "algo falló",
        errorCode: "NB-FW-003",
        diagnosticId: "detalle",
        safeParams: { cmd: "x" },
      },
    });
    vi.doUnmock("@tauri-apps/api/core");
  });

  it("un fallo al enviar (sin backend real) no lanza ni rompe nada", async () => {
    vi.doMock("@tauri-apps/api/core", () => ({
      invoke: vi.fn().mockRejectedValue(new Error("sin Tauri")),
    }));

    const { logFrontendEvent } = await import("./logging");
    await expect(
      logFrontendEvent({ level: "info", module: "x", eventCode: "y", message: "z" }),
    ).resolves.toBeUndefined();
    vi.doUnmock("@tauri-apps/api/core");
  });
});
