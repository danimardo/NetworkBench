import { z } from "zod";
import { invokeCommand } from "./transport";
import {
  cohortComparisonResultSchema,
  deletePreviewSchema,
  deleteResultSchema,
  historyPageSchema,
  peerTrendPointSchema,
  repeatPlanConfigSchema,
  sessionRecordSchema,
  type CohortComparisonResult,
  type DeletePreview,
  type DeleteResult,
  type DeleteTarget,
  type HistoryFilter,
  type HistoryPage,
  type Pagination,
  type PeerTrendPoint,
  type RepeatPlanConfig,
  type SessionRecord,
} from "../contracts/history";

export async function listHistory(
  filter?: HistoryFilter,
  pagination?: Pagination,
): Promise<HistoryPage> {
  return await invokeCommand(
    "history_list",
    {
      filter: filter ?? null,
      pagination: pagination ?? null,
    },
    historyPageSchema,
  );
}

export async function getHistoryDetail(sessionId: string): Promise<SessionRecord | null> {
  const res = await invokeCommand(
    "history_get",
    { sessionId },
    sessionRecordSchema.optional().nullable(),
  );
  return res ?? null;
}

export async function deletePreview(target: DeleteTarget): Promise<DeletePreview> {
  return await invokeCommand("history_delete_preview", { target }, deletePreviewSchema);
}

export async function deleteConfirm(token: string): Promise<DeleteResult | null> {
  const res = await invokeCommand(
    "history_delete_confirm",
    { token },
    deleteResultSchema.nullable().optional(),
  );
  return res ?? null;
}

export async function getPeerTrend(peerId: string, protocol?: string): Promise<PeerTrendPoint[]> {
  return await invokeCommand(
    "history_get_trend",
    { peerId, protocol: protocol ?? null },
    z.array(peerTrendPointSchema),
  );
}

export async function compareCohort(sessionId: string): Promise<CohortComparisonResult | null> {
  const res = await invokeCommand(
    "history_compare_cohort",
    { sessionId },
    cohortComparisonResultSchema.nullable().optional(),
  );
  return res ?? null;
}

export async function repeatPlan(sessionId: string): Promise<RepeatPlanConfig> {
  return await invokeCommand("history_repeat_plan", { sessionId }, repeatPlanConfigSchema);
}
