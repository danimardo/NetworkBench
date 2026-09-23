import { z } from "zod";
import { benchmarkPlanSchema } from "./plan";
import { uuidSchema, decimalU64Schema } from "./common";

export const capacitySourceSchema = z.enum(["manual", "negotiated", "wifi", "unknown"]);
export type CapacitySource = z.infer<typeof capacitySourceSchema>;

export const capacityReferenceSchema = z.object({
  refBps: z.number().nullable().optional(),
  refSource: capacitySourceSchema,
  capABps: z.number().nullable().optional(),
  capBBps: z.number().nullable().optional(),
});
export type CapacityReference = z.infer<typeof capacityReferenceSchema>;

export const verdictLevelSchema = z.enum(["ok", "warn", "problem", "notEvaluable"]);
export type VerdictLevel = z.infer<typeof verdictLevelSchema>;

export const stabilityLevelSchema = z.enum([
  "veryStable",
  "stable",
  "variable",
  "veryVariable",
  "notEvaluable",
]);
export type StabilityLevel = z.infer<typeof stabilityLevelSchema>;

export const retransmissionLevelSchema = z.enum(["normal", "elevated", "high", "notAvailable"]);
export type RetransmissionLevel = z.infer<typeof retransmissionLevelSchema>;

export const cpuLevelSchema = z.enum(["low", "moderate", "high", "notAvailable"]);
export type CpuLevel = z.infer<typeof cpuLevelSchema>;

export const stabilityStatsSchema = z.object({
  level: stabilityLevelSchema,
  meanBps: z.number(),
  stdDevBps: z.number(),
  cv: z.number(),
  minBps: z.number(),
  maxBps: z.number(),
  p5Bps: z.number(),
  p50Bps: z.number(),
  p95Bps: z.number(),
  dropsCount: z.number(),
  samplesCount: z.number(),
  gapsCount: z.number(),
});
export type StabilityStats = z.infer<typeof stabilityStatsSchema>;

export const retransmissionStatsSchema = z.object({
  level: retransmissionLevelSchema,
  ratio: z.number().nullable().optional(),
  packetsSent: z.number().nullable().optional(),
  packetsRetransmitted: z.number().nullable().optional(),
});
export type RetransmissionStats = z.infer<typeof retransmissionStatsSchema>;

export const asymmetryStatsSchema = z.object({
  ratio: z.number(),
  isAsymmetric: z.boolean(),
});
export type AsymmetryStats = z.infer<typeof asymmetryStatsSchema>;

export const diagnosticItemSchema = z.object({
  ruleId: z.string(),
  messageKey: z.string(),
  safeParams: z.record(z.string(), z.unknown()).nullable().optional(),
  evidenceRefs: z.array(z.string()).default([]),
});
export type DiagnosticItem = z.infer<typeof diagnosticItemSchema>;

export const sessionVerdictSchema = z.object({
  rulesVersion: z.string(),
  level: verdictLevelSchema,
  titleKey: z.string(),
  performanceLevel: verdictLevelSchema.nullable().optional(),
  stabilityLevel: stabilityLevelSchema,
  asymmetryLevel: verdictLevelSchema.nullable().optional(),
  retransmissionLevel: retransmissionLevelSchema,
  cpuLevel: cpuLevelSchema,
  facts: z.array(diagnosticItemSchema).default([]),
  observations: z.array(diagnosticItemSchema).default([]),
  possibleCauses: z.array(diagnosticItemSchema).default([]),
  actions: z.array(diagnosticItemSchema).default([]),
});
export type SessionVerdict = z.infer<typeof sessionVerdictSchema>;

export const peerSnapshotSchema = z.object({
  instanceId: uuidSchema,
  displayName: z.string(),
  fingerprint: z.string(),
  address: z.string(),
});
export type PeerSnapshot = z.infer<typeof peerSnapshotSchema>;

export const engineResultSchema = z.object({
  role: z.enum(["sender", "receiver"]),
  totalBytes: decimalU64Schema,
  realtimeSeconds: z.number(),
  throughputBps: decimalU64Schema,
  cpuPercent: z.number().nullable().optional(),
  buffersCount: z.number().nullable().optional(),
  errorsCount: z.number(),
});
export type EngineResult = z.infer<typeof engineResultSchema>;

export const directionResultSchema = z.object({
  direction: z.enum(["forward", "reverse"]),
  status: z.enum(["completed", "incomplete", "notStarted"]),
  sender: engineResultSchema.nullable().optional(),
  receiver: engineResultSchema.nullable().optional(),
  officialBps: decimalU64Schema.nullable().optional(),
  utilization: z.number().nullable().optional(),
  stability: stabilityStatsSchema.nullable().optional(),
  retransmission: retransmissionStatsSchema.nullable().optional(),
  cpuSender: z.number().nullable().optional(),
  cpuReceiver: z.number().nullable().optional(),
  samplesCount: z.number(),
  gapsCount: z.number(),
});
export type DirectionResult = z.infer<typeof directionResultSchema>;

export const resultVersionsSchema = z.object({
  appVersion: z.string(),
  protocolVersion: z.number(),
  engineVersion: z.string(),
  schemaVersion: z.number(),
  thresholdsHash: z.string(),
});
export type ResultVersions = z.infer<typeof resultVersionsSchema>;

export const sessionResultSchema = z.object({
  schemaVersion: z.number(),
  sessionId: uuidSchema,
  startedAt: z.string(),
  finishedAt: z.string(),
  status: z.enum(["completed", "incomplete"]),
  initiator: peerSnapshotSchema,
  responder: peerSnapshotSchema,
  plan: benchmarkPlanSchema,
  capacity: capacityReferenceSchema.nullable().optional(),
  directions: z.array(directionResultSchema),
  asymmetry: asymmetryStatsSchema.nullable().optional(),
  verdict: sessionVerdictSchema.nullable().optional(),
  resultSource: z.enum(["initiator", "local"]),
  versions: resultVersionsSchema,
});
export type SessionResult = z.infer<typeof sessionResultSchema>;
