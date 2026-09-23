import { z } from "zod";

export const exportFormatSchema = z.enum(["pdf", "json", "csv"]);
export type ExportFormat = z.infer<typeof exportFormatSchema>;

export const exportPreviewRequestSchema = z.object({
  sessionIds: z.array(z.string().uuid()).min(1),
  format: exportFormatSchema,
  anonymize: z.boolean(),
});
export type ExportPreviewRequest = z.infer<typeof exportPreviewRequestSchema>;

export const exportPreviewResponseSchema = z.object({
  token: z.string().min(1),
  fileNames: z.array(z.string()),
  disclosureKeys: z.array(z.string()),
  totalSessions: z.number().int().nonnegative(),
});
export type ExportPreviewResponse = z.infer<typeof exportPreviewResponseSchema>;

export const exportExecuteRequestSchema = z.object({
  token: z.string().min(1),
  sessionIds: z.array(z.string().uuid()).min(1),
  format: exportFormatSchema,
  anonymize: z.boolean(),
  destinationDir: z.string().min(1),
  delimiter: z.string().length(1).optional(),
  decimalSeparator: z.string().length(1).optional(),
  pdfContentBase64: z.string().optional(),
});
export type ExportExecuteRequest = z.infer<typeof exportExecuteRequestSchema>;

export const exportExecuteResponseSchema = z.object({
  exportedFiles: z.array(z.string()),
  bytesWritten: z.number().int().nonnegative(),
});
export type ExportExecuteResponse = z.infer<typeof exportExecuteResponseSchema>;
