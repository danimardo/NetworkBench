import { z } from "zod";
import { decimalU64Schema } from "./common";

export const benchmarkProtocolSchema = z.enum(["tcp", "udp"]);
export type BenchmarkProtocol = z.infer<typeof benchmarkProtocolSchema>;

export const benchmarkDirectionSchema = z.enum([
  "forward",
  "reverse",
  "both",
  "both_sequential",
  "both_simultaneous",
]);
export type BenchmarkDirection = z.infer<typeof benchmarkDirectionSchema>;

/**
 * Límites de BenchmarkPlan según FR-042 / Historias.md / Contratos:
 * - streams: 1..64 (1..32 por sentido en simultáneo 'both'/'both_simultaneous')
 * - warmupSeconds: 0..10
 * - measureSeconds: 5..300
 * - cooldownSeconds: 0..10
 * - port: 1024..65000
 * - bufferSizeBytes: 4 KB..4 MB potencias de 2
 * - udpTargetRateBps: 1 Mbit/s..100 000 Mbit/s
 * - udpPacketSizeBytes: 64..65 507 bytes
 * - expectedCapacityBps: 1 Mbit/s..400 000 Mbit/s
 */
export const benchmarkPlanSchema = z
  .object({
    protocol: benchmarkProtocolSchema,
    direction: benchmarkDirectionSchema,
    streams: z.number().int().min(1, "Al menos 1 stream").max(64, "Máximo 64 streams"),
    warmupSeconds: z
      .number()
      .int()
      .min(0, "Calentamiento mínimo 0 s")
      .max(10, "Calentamiento máximo 10 s"),
    measureSeconds: z
      .number()
      .int()
      .min(5, "Medición mínima 5 s")
      .max(300, "Medición máxima 300 s"),
    cooldownSeconds: z
      .number()
      .int()
      .min(0, "Enfriamiento mínimo 0 s")
      .max(10, "Enfriamiento máximo 10 s"),
    port: z.number().int().min(1024, "Puerto reservado").max(65000, "Puerto base máximo 65000"),
    bufferSizeBytes: decimalU64Schema.nullable().optional(),
    udpTargetRateBps: decimalU64Schema.nullable().optional(),
    udpPacketSizeBytes: z.number().int().min(64).max(65507).nullable().optional(),
    expectedCapacityBps: decimalU64Schema.nullable().optional(),
  })
  .refine(
    (plan) => {
      const isSimultaneous = plan.direction === "both" || plan.direction === "both_simultaneous";
      if (isSimultaneous && plan.streams > 32) {
        return false;
      }
      return true;
    },
    {
      message: "En pruebas simultáneas el límite es de 1..32 streams por sentido",
      path: ["streams"],
    },
  );

export type BenchmarkPlan = z.infer<typeof benchmarkPlanSchema>;

export const udpLossLevelSchema = z.enum(["low", "moderate", "high", "not_evaluable"]);
export type UdpLossLevel = z.infer<typeof udpLossLevelSchema>;

export const udpDiagnosticResultSchema = z.object({
  targetRateBps: z.number().nullable().optional(),
  emittedRateBps: z.number(),
  receivedRateBps: z.number(),
  packetsSent: z.number(),
  packetsReceived: z.number(),
  packetsLost: z.number(),
  lossRatio: z.number(),
  lossPercent: z.number(),
  lossLevel: udpLossLevelSchema,
  titleKey: z.string(),
  observations: z.array(z.string()),
  isTargetExceeded: z.boolean(),
});
export type UdpDiagnosticResult = z.infer<typeof udpDiagnosticResultSchema>;
