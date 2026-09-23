import { invokeCommand } from "./transport";
import type { SessionResult } from "../contracts/session-result";
import { z } from "zod";

export async function getDiagnosticReport(result: SessionResult): Promise<string> {
  return invokeCommand("diagnostics_get_report", { result }, z.string());
}
