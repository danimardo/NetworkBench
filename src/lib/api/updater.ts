import { invokeCommand } from "./transport";
import { UpdateStatusSchema, type UpdateStatus } from "../contracts/settings";

export async function checkUpdate(): Promise<UpdateStatus> {
  return await invokeCommand("updater_check", {}, UpdateStatusSchema);
}

export async function evaluateUpdate(manifestJson: string): Promise<UpdateStatus> {
  return await invokeCommand("updater_evaluate", { manifestJson }, UpdateStatusSchema);
}
