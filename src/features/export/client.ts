import { previewExport, executeExport } from "../../lib/api/export";
import type {
  ExportFormat,
  ExportPreviewResponse,
  ExportExecuteResponse,
} from "../../lib/contracts/export";

export interface ExportSessionIntent {
  sessionIds: string[];
  peerName?: string;
  defaultFormat?: ExportFormat;
}

/**
 * Cliente de conveniencia para la orquestación de exportaciones desde cualquier pantalla
 * (Historial o Resultados) sin acceder a stores privados.
 */
export async function requestExportPreview(
  sessionIds: string[],
  format: ExportFormat = "pdf",
  anonymize: boolean = false,
): Promise<ExportPreviewResponse> {
  return await previewExport({
    sessionIds,
    format,
    anonymize,
  });
}

export async function requestExportExecution(
  token: string,
  sessionIds: string[],
  format: ExportFormat,
  destinationDir: string,
  anonymize: boolean = false,
  delimiter?: string,
  decimalSeparator?: string,
): Promise<ExportExecuteResponse> {
  return await executeExport({
    token,
    sessionIds,
    format,
    anonymize,
    destinationDir,
    delimiter,
    decimalSeparator,
  });
}
