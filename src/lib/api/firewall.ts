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

/** Nombre de la regla del puerto de control (`firewall::reglas::NOMBRE_CONTROL`). */
export const NOMBRE_REGLA_CONTROL = "NetworkBench - Control";

/**
 * Una de las reglas de NetworkBench (`Historias.md` §14.1) con su estado real en el sistema.
 * Los nombres de campo son los del backend (`firewall::estado::EstadoRegla`).
 */
export const firewallRuleStateSchema = z.object({
  nombre: z.string(),
  protocolo: z.string(),
  /** Un puerto (`7411`) o un rango (`5001-5064`). */
  puertos: z.string(),
  programa: z.string(),
  perfiles: z.array(z.string()),
  /** Grupo con el que se crea; el desinstalador retira las reglas por él. */
  grupo: z.string(),
  estado: ruleStatusSchema,
  detalle: z.string(),
  /** Sin el programa en disco el helper rechaza la petición: la regla no se puede crear. */
  programaExiste: z.boolean(),
  /** Comando de PowerShell equivalente, para copiarlo (`netsh` no puede asignar el grupo). */
  comandoAgregar: z.string(),
});
export type FirewallRuleState = z.infer<typeof firewallRuleStateSchema>;

/** Una red a la que está conectado el equipo y cómo la clasifica Windows. */
export const activeNetworkSchema = z.object({
  nombre: z.string(),
  interfaz: z.string(),
  /** `Public`, `Private` o `DomainAuthenticated`. Una VPN suele salir como `Public`. */
  categoria: z.string(),
});
export type ActiveNetwork = z.infer<typeof activeNetworkSchema>;

export const firewallRulesReportSchema = z.object({
  reglas: z.array(firewallRuleStateSchema),
  redes: z.array(activeNetworkSchema),
  /** La persona ha permitido también las redes públicas (§14.5). */
  permitirPublico: z.boolean(),
  puertoControl: z.number().int(),
  /** Existe el helper elevado; sin él no se pueden crear ni eliminar reglas desde la app. */
  ayudanteDisponible: z.boolean(),
});
export type FirewallRulesReport = z.infer<typeof firewallRulesReportSchema>;

/** Estado de las reglas, sin elevación. Puede tardar unos segundos (lee con PowerShell). */
export async function getFirewallRulesStatus(): Promise<FirewallRulesReport> {
  return invokeCommand("firewall_rules_status", undefined, firewallRulesReportSchema);
}

/** «Crear las que faltan». Abre el UAC de Windows; devuelve el estado tras aplicarlo. */
export async function createMissingFirewallRules(): Promise<FirewallRulesReport> {
  return invokeCommand("firewall_rules_create", undefined, firewallRulesReportSchema);
}

/** «Eliminar todas». Abre el UAC de Windows; devuelve el estado tras aplicarlo. */
export async function removeFirewallRules(): Promise<FirewallRulesReport> {
  return invokeCommand("firewall_rules_remove", undefined, firewallRulesReportSchema);
}

/** Redes que Windows considera públicas: en ellas las reglas por defecto no se aplican. */
export function publicNetworks(informe: FirewallRulesReport): ActiveNetwork[] {
  return informe.redes.filter((r) => r.categoria === "Public");
}

/** Abre la página de estado de red de Configuración de Windows (§14.5). */
export async function openNetworkSettings(): Promise<void> {
  await invokeCommand("firewall_open_network_settings", undefined, z.void().nullable());
}
