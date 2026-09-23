import { z } from "zod";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const samplePointSchema = z.object({
  tMs: z.number(),
  direction: z.string(),
  bps: z.number(),
  cpuPercent: z.number().nullable().optional(),
  gap: z.boolean(),
});
export type SamplePoint = z.infer<typeof samplePointSchema>;

export const sampleBatchSchema = z.object({
  sessionId: z.string(),
  direction: z.string(),
  samples: z.array(samplePointSchema),
  latestBps: z.number(),
  latestCpuPercent: z.number().nullable().optional(),
  hasGaps: z.boolean(),
});
export type SampleBatch = z.infer<typeof sampleBatchSchema>;

export async function onSampleBatch(callback: (batch: SampleBatch) => void): Promise<UnlistenFn> {
  return listen<SampleBatch>("session://sample-batch", (event) => {
    const parsed = sampleBatchSchema.safeParse(event.payload);
    if (parsed.success) {
      callback(parsed.data);
    }
  });
}
