import { z } from "zod";

export const errorSeveritySchema = z.enum(["info", "warning", "error", "fatal"]);
export type ErrorSeverity = z.infer<typeof errorSeveritySchema>;

export const errorCodeSchema = z.enum([
  "NB-CONN-001",
  "NB-CONN-002",
  "NB-CONN-003",
  "NB-CONN-004",
  "NB-CONN-005",
  "NB-CONN-006",
  "NB-VERSION-001",
  "NB-PEER-001",
  "NB-PEER-002",
  "NB-PEER-003",
  "NB-PEER-004",
  "NB-PEER-005",
  "NB-PEER-006",
  "NB-PARAM-001",
  "NB-PORT-001",
  "NB-PORT-002",
  "NB-FW-001",
  "NB-FW-002",
  "NB-FW-003",
  "NB-FW-004",
  "NB-FW-005",
  "NB-ENGINE-001",
  "NB-ENGINE-002",
  "NB-ENGINE-003",
  "NB-ENGINE-004",
  "NB-ENGINE-005",
  "NB-ENGINE-006",
  "NB-RESULT-001",
  "NB-RESULT-002",
  "NB-NIC-001",
  "NB-NIC-002",
  "NB-NIC-003",
  "NB-NIC-004",
  "NB-PERM-001",
  "NB-DISK-001",
  "NB-DATA-001",
  "NB-UNEXPECTED-001",
  "NB-INTERNAL-001",
]);
export type ErrorCode = z.infer<typeof errorCodeSchema>;

export const errorActionSchema = z.enum([
  "retry",
  "repeat_test",
  "show_details",
  "change_address",
  "change_port",
  "change_ports",
  "retry_pairing",
  "retry_later",
  "reverify_identity",
  "check_install",
  "reinstall",
  "open_firewall_settings",
  "configure_firewall",
  "show_firewall_instructions",
  "retry_as_admin",
  "choose_interface",
  "open_network_settings",
  "free_space",
  "update_app",
  "copy_diagnostic",
  "copy_diagnostics",
  "check_connection",
]);
export type ErrorAction = z.infer<typeof errorActionSchema>;

export const appIssueSchema = z.object({
  path: z.string(),
  code: z.string(),
  messageKey: z.string(),
  safeParams: z.record(z.string(), z.unknown()).optional(),
});
export type AppIssue = z.infer<typeof appIssueSchema>;

export const appErrorSchema = z.object({
  code: errorCodeSchema,
  severity: errorSeveritySchema,
  messageKey: z.string().min(1),
  issues: z.array(appIssueSchema).default([]),
  actions: z.array(errorActionSchema).default([]),
  diagnosticId: z.string().optional(),
});
export type AppError = z.infer<typeof appErrorSchema>;
