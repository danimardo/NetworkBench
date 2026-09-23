import { z } from "zod";
import { invokeCommand } from "./transport";

export const ruleStatusSchema = z.enum(["present", "missing", "modified", "disabled"]);
export type RuleStatus = z.infer<typeof ruleStatusSchema>;

export const firewallInspectionSchema = z.object({
  ruleName: z.string(),
  status: ruleStatusSchema,
  isPolicyManaged: z.boolean(),
  details: z.string().optional().nullable(),
});
export type FirewallInspection = z.infer<typeof firewallInspectionSchema>;

export const firewallHelperRequestSchema = z.object({
  operation: z.enum(["add", "remove"]),
  ruleName: z.string(),
  protocol: z.enum(["TCP", "UDP"]),
  portRange: z.string(),
  program: z.string(),
  profiles: z.array(z.string()),
});
export type FirewallHelperRequest = z.infer<typeof firewallHelperRequestSchema>;

export async function inspectFirewallRule(
  ruleName: string,
  expectedPortRange: string,
  expectedProto: string = "TCP",
): Promise<FirewallInspection> {
  return await invokeCommand(
    "firewall_inspect",
    {
      request: {
        ruleName,
        expectedPortRange,
        expectedProto,
      },
    },
    firewallInspectionSchema,
  );
}

export async function applyFirewallRules(rules: FirewallHelperRequest[]): Promise<void> {
  await invokeCommand("firewall_apply", { rules }, z.void().nullable());
}
