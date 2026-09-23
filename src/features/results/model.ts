import type { Peer } from "../../lib/contracts/peer";
import type { BenchmarkPlan } from "../../lib/contracts/plan";

export interface BasicResultModel {
  sessionId: string;
  peer: Peer;
  plan: BenchmarkPlan;
  forwardOfficialBps: string | null;
  reverseOfficialBps: string | null;
  durationSeconds: number;
  completedAt: string;
}

export function formatThroughput(bpsStr: string | null | undefined): string {
  if (!bpsStr) return "No disponible";
  const num = Number(bpsStr);
  if (isNaN(num) || num <= 0) return "0 Mbit/s";
  const mbps = num / 1_000_000;
  if (mbps < 1000) {
    return `${mbps.toFixed(1)} Mbit/s`;
  }
  return `${(mbps / 1000).toFixed(2)} Gbit/s`;
}
