import { invokeCommand } from "./transport";
import {
  exportPreviewResponseSchema,
  exportExecuteResponseSchema,
  type ExportPreviewRequest,
  type ExportPreviewResponse,
  type ExportExecuteRequest,
  type ExportExecuteResponse,
} from "../contracts/export";

export async function previewExport(request: ExportPreviewRequest): Promise<ExportPreviewResponse> {
  return await invokeCommand("export_preview", { request }, exportPreviewResponseSchema);
}

export async function executeExport(request: ExportExecuteRequest): Promise<ExportExecuteResponse> {
  return await invokeCommand("export_execute", { request }, exportExecuteResponseSchema);
}
