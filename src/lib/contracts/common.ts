import { z } from "zod";

/**
 * UUID v4 canónico para identidades y sesiones.
 */
export const uuidSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    "Debe ser un UUID v4 canónico",
  );

export const instanceIdSchema = uuidSchema;
export type InstanceId = z.infer<typeof instanceIdSchema>;

export const sessionIdSchema = uuidSchema;
export type SessionId = z.infer<typeof sessionIdSchema>;

/**
 * Cadena decimal para representar enteros de 64 bits sin pérdida de precisión.
 * Acepta string de dígitos decimales o enteros seguros (Number.isSafeInteger) y normaliza a string.
 */
const MAX_U64 = 18446744073709551615n;

export const decimalU64Schema = z.union([
  z
    .string()
    .regex(/^\d+$/, "Debe ser una representación decimal entera no negativa")
    .refine((s) => {
      try {
        const b = BigInt(s);
        return b >= 0n && b <= MAX_U64;
      } catch {
        return false;
      }
    }, "Debe estar dentro del rango de entero de 64 bits sin signo (0..18446744073709551615)"),
  z
    .number()
    .int()
    .nonnegative()
    .refine((n) => Number.isSafeInteger(n), {
      message: "El entero excede Number.MAX_SAFE_INTEGER y debe transferirse como cadena decimal",
    })
    .transform((n) => n.toString()),
  z
    .bigint()
    .nonnegative()
    .refine((b) => b <= MAX_U64, "Debe ser menor o igual a 18446744073709551615")
    .transform((b) => b.toString()),
]);

export type DecimalU64 = z.infer<typeof decimalU64Schema>;

/**
 * Estado discriminado para métricas y magnitudes:
 * Distingue explícitamente entre valor disponible, ausencia, imposibilidad de evaluación e invalidez.
 */
export function createMetricValueSchema<T extends z.ZodTypeAny>(valueSchema: T) {
  return z.discriminatedUnion("status", [
    z.object({
      status: z.literal("available"),
      value: valueSchema,
    }),
    z.object({
      status: z.literal("notAvailable"),
    }),
    z.object({
      status: z.literal("notEvaluable"),
      reason: z.string().min(1, "Se requiere motivo de no evaluación"),
    }),
    z.object({
      status: z.literal("invalid"),
      reason: z.string().min(1, "Se requiere motivo de invalidez"),
    }),
  ]);
}

export const throughputBpsSchema = decimalU64Schema;
export type ThroughputBps = DecimalU64;

export const byteCountSchema = decimalU64Schema;
export type ByteCount = DecimalU64;

export const durationMsSchema = z.number().int().nonnegative();
export type DurationMs = z.infer<typeof durationMsSchema>;
