import { describe, it, expect, vi, afterEach } from "vitest";
import { setTransportMock, invokeCommand, IpcError } from "./transport";

// Hallazgo real de la auditoría de logging (2026-09-28): casi ningún AppError que veía la
// persona en pantalla llegaba nunca a un fichero de registro — se mostraba en un aviso
// flotante y desaparecía sin dejar rastro. Estos tests fijan que `invokeCommand` registra
// todo error, tanto el que devuelve el backend como el de un fallo de transporte puro.

vi.mock("../logging", () => ({
  logger: {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}));

afterEach(() => {
  setTransportMock(null);
  vi.clearAllMocks();
});

describe("transporte IPC: todo AppError que ve la persona queda registrado", () => {
  it("un error devuelto por el backend se registra con su código y detalle", async () => {
    const { logger } = await import("../logging");
    setTransportMock(async () => ({
      ok: false,
      error: {
        code: "NB-FW-003",
        severity: "error",
        messageKey: "errors.NB-FW-003",
        issues: [],
        actions: [],
        diagnosticId: "detalle de prueba",
      },
    }));

    await expect(invokeCommand("firewall_rules_status", {})).rejects.toBeInstanceOf(IpcError);

    expect(logger.error).toHaveBeenCalledTimes(1);
    const evento = vi.mocked(logger.error).mock.calls[0]![0];
    expect(evento.errorCode).toBe("NB-FW-003");
    expect(evento.diagnosticId).toBe("detalle de prueba");
    expect(evento.message).toContain("firewall_rules_status");
  });

  it("la severidad decide el nivel: warning registra con logger.warn, no logger.error", async () => {
    const { logger } = await import("../logging");
    setTransportMock(async () => ({
      ok: false,
      error: {
        code: "NB-FW-004",
        severity: "warning",
        messageKey: "errors.NB-FW-004",
        issues: [],
        actions: [],
      },
    }));

    await expect(invokeCommand("firewall_rules_create", {})).rejects.toBeInstanceOf(IpcError);

    expect(logger.warn).toHaveBeenCalledTimes(1);
    expect(logger.error).not.toHaveBeenCalled();
  });

  it("un fallo de transporte puro (sin AppError del backend) también se registra", async () => {
    const { logger } = await import("../logging");
    setTransportMock(async () => {
      throw new Error("conexión perdida");
    });

    await expect(invokeCommand("peers_list", {})).rejects.toBeInstanceOf(IpcError);

    expect(logger.error).toHaveBeenCalledTimes(1);
    const evento = vi.mocked(logger.error).mock.calls[0]![0];
    expect(evento.errorCode).toBe("NB-INTERNAL-001");
  });
});
