import { z } from "zod";
import { appErrorSchema, type AppError } from "./errors";

export const oneTimeTokenSchema = z.string().uuid();
export type OneTimeToken = z.infer<typeof oneTimeTokenSchema>;

export function createIpcSuccessSchema<T extends z.ZodTypeAny>(valueSchema: T) {
  return z.object({
    ok: z.literal(true),
    value: valueSchema,
  });
}

export const ipcFailureSchema = z.object({
  ok: z.literal(false),
  error: appErrorSchema,
});

export function createIpcResultSchema<T extends z.ZodTypeAny>(valueSchema: T) {
  return z.discriminatedUnion("ok", [createIpcSuccessSchema(valueSchema), ipcFailureSchema]);
}

export type IpcSuccess<T> = {
  ok: true;
  value: T;
};

export type IpcFailure = {
  ok: false;
  error: AppError;
};

export type IpcResult<T> = IpcSuccess<T> | IpcFailure;
