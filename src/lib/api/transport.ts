import type { z } from "zod";
import { type AppError, appErrorSchema } from "../contracts/errors";
import type { IpcResult } from "../contracts/ipc";
import { logger } from "../logging";

export class IpcError extends Error {
  readonly appError: AppError;

  constructor(appError: AppError) {
    super(`[${appError.code}] ${appError.messageKey}`);
    this.name = "IpcError";
    this.appError = appError;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

let customInvoke: InvokeFn | null = null;

export function setTransportMock(mock: InvokeFn | null): void {
  customInvoke = mock;
}

async function callNativeInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
  if (customInvoke) {
    return customInvoke(cmd, args);
  }

  try {
    // Import dinámico para que no falle en entornos de prueba o Node puro sin Tauri
    const tauri = await import("@tauri-apps/api/core");
    return await tauri.invoke(cmd, args);
  } catch (err) {
    logger.error({
      module: "transport",
      eventCode: "TAURI_INVOKE_FAILED",
      message: `Fallo al invocar comando nativo ${cmd}`,
      safeParams: { cmd },
    });
    throw err;
  }
}

export async function invokeCommand<
  TOutput,
  TInput extends Record<string, unknown> = Record<string, unknown>,
>(command: string, input?: TInput, outputSchema?: z.ZodType<TOutput>): Promise<TOutput> {
  try {
    const rawResult = await callNativeInvoke(command, input ? { request: input } : undefined);

    // Si viene en formato IpcResult nativo { ok: true, value } | { ok: false, error }
    if (rawResult && typeof rawResult === "object" && "ok" in rawResult) {
      const result = rawResult as IpcResult<TOutput>;
      if (result.ok) {
        if (outputSchema) {
          return outputSchema.parse(result.value);
        }
        return result.value;
      } else {
        const validatedError = appErrorSchema.parse(result.error);
        throw new IpcError(validatedError);
      }
    }

    // Si no está envuelto (compatibilidad con comandos que retornan directamente TOutput)
    if (outputSchema) {
      return outputSchema.parse(rawResult);
    }
    return rawResult as TOutput;
  } catch (err: unknown) {
    if (err instanceof IpcError) {
      throw err;
    }

    // Transforma cualquier fallo inesperado de transporte a AppError seguro tipado
    const fallbackError: AppError = {
      code: "NB-INTERNAL-001",
      severity: "fatal",
      messageKey: "errors.NB-INTERNAL-001",
      issues: [
        {
          path: command,
          code: "TRANSPORT_EXCEPTION",
          messageKey: "errors.NB-INTERNAL-001",
          safeParams: { cmd: command },
        },
      ],
      actions: ["retry", "copy_diagnostic"],
    };

    throw new IpcError(fallbackError);
  }
}
