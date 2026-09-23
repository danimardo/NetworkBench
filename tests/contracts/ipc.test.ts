import { describe, it, expect, beforeEach, vi } from "vitest";
import { z } from "zod";
import {
  createIpcResultSchema,
  oneTimeTokenSchema,
  type IpcResult,
} from "../../src/lib/contracts/ipc";
import { invokeCommand, setTransportMock, IpcError } from "../../src/lib/api/transport";

describe("IPC Envelope and Transport Contract Tests (contracts/ipc.md)", () => {
  beforeEach(() => {
    setTransportMock(null);
    vi.restoreAllMocks();
  });

  it("valida Success<T> y Failure con createIpcResultSchema", () => {
    const stringSchema = createIpcResultSchema(z.string());

    const validSuccess: IpcResult<string> = {
      ok: true,
      value: "hello world",
    };
    expect(stringSchema.parse(validSuccess)).toEqual(validSuccess);

    const validFailure: IpcResult<string> = {
      ok: false,
      error: {
        code: "NB-CONN-001",
        severity: "error",
        messageKey: "errors.NB-CONN-001",
        issues: [],
        actions: ["retry"],
      },
    };
    expect(stringSchema.parse(validFailure)).toEqual(validFailure);

    // Variante inválida: ok falta
    expect(() => stringSchema.parse({ value: "missing ok" })).toThrow();

    // Variante inválida: error sin código válido
    expect(() =>
      stringSchema.parse({
        ok: false,
        error: { code: "INVALID_CODE", severity: "error", messageKey: "msg" },
      }),
    ).toThrow();
  });

  it("valida oneTimeToken como UUID válido", () => {
    const validUuid = "123e4567-e89b-42d3-a456-426614174000";
    expect(oneTimeTokenSchema.parse(validUuid)).toBe(validUuid);

    expect(() => oneTimeTokenSchema.parse("invalid-token-string")).toThrow();
  });

  it("invokeCommand desenvuelve Success<T> validando schema", async () => {
    setTransportMock(async () => {
      return {
        ok: true,
        value: { id: "test-123", active: true },
      };
    });

    const schema = z.object({ id: z.string(), active: z.boolean() });
    const result = await invokeCommand("app.test", {}, schema);

    expect(result).toEqual({ id: "test-123", active: true });
  });

  it("invokeCommand lanza IpcError tipado ante Failure de Rust", async () => {
    setTransportMock(async () => {
      return {
        ok: false,
        error: {
          code: "NB-PEER-001",
          severity: "error",
          messageKey: "errors.NB-PEER-001",
          issues: [],
          actions: ["retry_pairing"],
        },
      };
    });

    await expect(invokeCommand("peers.confirmPairing", {})).rejects.toThrow(IpcError);
    try {
      await invokeCommand("peers.confirmPairing", {});
    } catch (e: unknown) {
      expect(e).toBeInstanceOf(IpcError);
      const ipcErr = e as IpcError;
      expect(ipcErr.appError.code).toBe("NB-PEER-001");
      expect(ipcErr.appError.actions).toContain("retry_pairing");
    }
  });

  it("invokeCommand captura excepciones no controladas y devuelve AppError NB-INTERNAL-001", async () => {
    setTransportMock(async () => {
      throw new Error("Tauri IPC crash simulado");
    });

    try {
      await invokeCommand("session.cancel", {});
      expect.unreachable();
    } catch (e: unknown) {
      expect(e).toBeInstanceOf(IpcError);
      const ipcErr = e as IpcError;
      expect(ipcErr.appError.code).toBe("NB-INTERNAL-001");
      expect(ipcErr.appError.severity).toBe("fatal");
    }
  });
});
