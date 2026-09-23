import { z } from "zod";
import { uuidSchema } from "./common";
import { benchmarkPlanSchema } from "./plan";

export const MAX_FRAME_SIZE_BYTES = 1048576; // 1 MiB

export const protocolMessageTypeSchema = z.enum([
  "HELLO",
  "PAIR_REQUEST",
  "PAIR_RESULT",
  "REQUEST",
  "RESPONSE",
  "PREPARE",
  "PREPARE_RESULT",
  "READY",
  "START",
  "STARTED",
  "SAMPLE",
  "ENGINE_DONE",
  "ENGINE_FAILED",
  "SESSION_RESULT",
  "SESSION_ACK",
  "CANCEL",
  "CANCEL_ACK",
  "HEARTBEAT",
  "ERROR",
  "BYE",
]);
export type ProtocolMessageType = z.infer<typeof protocolMessageTypeSchema>;

export const protocolEnvelopeSchema = z.object({
  type: protocolMessageTypeSchema,
  id: uuidSchema,
  sessionId: uuidSchema.nullable(),
  ts: z.string().datetime({ message: "ts debe ser ISO 8601 UTC" }),
  inReplyTo: uuidSchema.nullable().optional(),
  payload: z.record(z.string(), z.unknown()),
});

export type ProtocolEnvelope = z.infer<typeof protocolEnvelopeSchema>;

// Payloads específicos
export const helloPayloadSchema = z.object({
  protocolVersion: z.number().int().min(1),
  protocolMin: z.number().int().min(1),
  appVersion: z.string().min(1),
  instanceId: uuidSchema,
  displayName: z.string().min(1).max(48),
  platform: z.string(),
  isBusy: z.boolean(),
});
export type HelloPayload = z.infer<typeof helloPayloadSchema>;

export const pairRequestPayloadSchema = z.object({
  pairingCode: z
    .string()
    .regex(/^\d{6}$/, "El código de emparejamiento debe tener exactamente 6 dígitos"),
});
export type PairRequestPayload = z.infer<typeof pairRequestPayloadSchema>;

export const pairResultPayloadSchema = z.object({
  accepted: z.boolean(),
  reason: z.string().nullable().optional(),
});
export type PairResultPayload = z.infer<typeof pairResultPayloadSchema>;

export const requestPayloadSchema = z.object({
  plan: benchmarkPlanSchema,
  estimateSeconds: z.number().int().positive(),
  suggestedInterface: z.string().nullable().optional(),
});
export type RequestPayload = z.infer<typeof requestPayloadSchema>;

export const responsePayloadSchema = z.object({
  accepted: z.boolean(),
  reason: z.string().nullable().optional(),
  interfaceName: z.string().nullable().optional(),
});
export type ResponsePayload = z.infer<typeof responsePayloadSchema>;

export const cancelPayloadSchema = z.object({
  reason: z.string().min(1),
  code: z.string().nullable().optional(),
});
export type CancelPayload = z.infer<typeof cancelPayloadSchema>;

export const heartbeatPayloadSchema = z.object({
  nonce: z.number().int().nonnegative(),
});
export type HeartbeatPayload = z.infer<typeof heartbeatPayloadSchema>;
