import { invokeCommand } from "./transport";
import type { Peer } from "../contracts/peer";
import type { BenchmarkPlan } from "../contracts/plan";
import { uuidSchema } from "../contracts/common";
import { z } from "zod";

const startSessionResultSchema = z.object({
  sessionId: uuidSchema,
});

export async function startSession(peer: Peer, plan: BenchmarkPlan): Promise<string> {
  const result = await invokeCommand(
    "session_start",
    {
      payload: {
        peer,
        plan,
      },
    },
    startSessionResultSchema,
  );
  return result.sessionId;
}

export async function cancelSession(sessionId: string, reason?: string): Promise<void> {
  await invokeCommand("session_cancel", { sessionId, reason: reason ?? null }, z.void().nullable());
}

export async function getSessionState(): Promise<string> {
  const result = await invokeCommand("session_get_state", undefined, z.string());
  return result;
}
