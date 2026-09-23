import { z } from "zod";
import { uuidSchema } from "./common";
import { peerSchema } from "./peer";
import { benchmarkPlanSchema } from "./plan";

export const historyFilterSchema = z.object({
  peerId: uuidSchema.optional().nullable(),
  searchQuery: z.string().optional().nullable(),
  verdict: z.string().optional().nullable(),
  protocol: z.string().optional().nullable(),
  status: z.string().optional().nullable(),
  fromDate: z.string().optional().nullable(),
  toDate: z.string().optional().nullable(),
});
export type HistoryFilter = z.infer<typeof historyFilterSchema>;

export const paginationSchema = z.object({
  limit: z.number().int().min(1).max(100),
  offset: z.number().int().min(0),
});
export type Pagination = z.infer<typeof paginationSchema>;

export const sampleRecordSchema = z.object({
  tMs: z.number().int(),
  direction: z.string(),
  bps: z.string(),
  cpuPercent: z.number().nullable().optional(),
  gap: z.boolean(),
});
export type SampleRecord = z.infer<typeof sampleRecordSchema>;

export const sessionRecordSchema = z.object({
  id: uuidSchema,
  createdAt: z.string(),
  peerId: uuidSchema.nullable().optional(),
  status: z.string(),
  protocol: z.string(),
  streams: z.number().int(),
  durationSeconds: z.number().int(),
  forwardBps: z.string().nullable().optional(),
  reverseBps: z.string().nullable().optional(),
  isPartial: z.boolean(),
  diagnosticVerdict: z.string().nullable().optional(),
  clientInterface: z.string().nullable().optional(),
  serverInterface: z.string().nullable().optional(),
  resultJson: z.string().nullable().optional(),
  planJson: z.string().nullable().optional(),
  samples: z.array(sampleRecordSchema),
});
export type SessionRecord = z.infer<typeof sessionRecordSchema>;

export const historyItemSummarySchema = z.object({
  id: uuidSchema,
  createdAt: z.string(),
  peerId: uuidSchema.optional().nullable(),
  peerName: z.string(),
  peerFingerprint: z.string().optional().nullable(),
  status: z.string(),
  protocol: z.string(),
  streams: z.number().int(),
  durationSeconds: z.number().int(),
  forwardBps: z.string().optional().nullable(),
  reverseBps: z.string().optional().nullable(),
  isPartial: z.boolean(),
  diagnosticVerdict: z.string().optional().nullable(),
  clientInterface: z.string().optional().nullable(),
  serverInterface: z.string().optional().nullable(),
});
export type HistoryItemSummary = z.infer<typeof historyItemSummarySchema>;

export const historyPageSchema = z.object({
  items: z.array(historyItemSummarySchema),
  totalCount: z.number().int(),
  hasMore: z.boolean(),
});
export type HistoryPage = z.infer<typeof historyPageSchema>;

export const deleteTargetSchema = z.discriminatedUnion("type", [
  z.object({ type: z.literal("all") }),
  z.object({ type: z.literal("byPeer"), value: uuidSchema }),
  z.object({ type: z.literal("olderThan"), value: z.string() }),
  z.object({ type: z.literal("single"), value: uuidSchema }),
]);
export type DeleteTarget = z.infer<typeof deleteTargetSchema>;

export const deletePreviewSchema = z.object({
  target: deleteTargetSchema,
  affectedSessions: z.number().int(),
  affectedSamples: z.number().int(),
  confirmationToken: z.string(),
  expiresInSeconds: z.number().int(),
});
export type DeletePreview = z.infer<typeof deletePreviewSchema>;

export const deleteResultSchema = z.object({
  deletedSessions: z.number().int(),
  deletedSamples: z.number().int(),
});
export type DeleteResult = z.infer<typeof deleteResultSchema>;

export const cohortComparisonResultSchema = z.object({
  sampleCount: z.number().int(),
  averageForwardBps: z.number(),
  currentForwardBps: z.number(),
  differencePercent: z.number(),
  isSignificantlyLower: z.boolean(),
  isSignificantlyHigher: z.boolean(),
  observationText: z.string().optional().nullable(),
});
export type CohortComparisonResult = z.infer<typeof cohortComparisonResultSchema>;

export const peerTrendPointSchema = z.object({
  sessionId: uuidSchema,
  createdAt: z.string(),
  forwardBps: z.string().optional().nullable(),
  reverseBps: z.string().optional().nullable(),
  verdict: z.string().optional().nullable(),
  protocol: z.string(),
  clientInterface: z.string().optional().nullable(),
  serverInterface: z.string().optional().nullable(),
});
export type PeerTrendPoint = z.infer<typeof peerTrendPointSchema>;

export const repeatPlanConfigSchema = z.object({
  peer: peerSchema.optional().nullable(),
  plan: benchmarkPlanSchema,
  originalSessionId: uuidSchema,
});
export type RepeatPlanConfig = z.infer<typeof repeatPlanConfigSchema>;
