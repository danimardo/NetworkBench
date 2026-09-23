<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Button from "../../lib/components/Button.svelte";
  import { t } from "../../lib/i18n";
  import { inspectFirewallRule, type FirewallInspection } from "../../lib/api/firewall";

  let inspection = $state<FirewallInspection | null>(null);
  let isInspecting = $state(false);
  let inspectError = $state<string | null>(null);

  async function handleInspect() {
    isInspecting = true;
    inspectError = null;
    try {
      inspection = await inspectFirewallRule("NetworkBench-Control", "7411", "TCP");
    } catch (e: unknown) {
      inspectError = e instanceof Error ? e.message : String(e);
    } finally {
      isInspecting = false;
    }
  }
</script>

<div class="space-y-6">
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold">{t("settings.firewallRules")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.firewallRulesDesc")}
          </p>
        </div>
        <Button variant="secondary" onclick={handleInspect} disabled={isInspecting}>
          {isInspecting ? t("common.checking") : t("settings.checkFirewallNow")}
        </Button>
      </div>

      {#if inspectError}
        <p class="text-xs text-[var(--color-text-danger)]">{inspectError}</p>
      {/if}

      {#if inspection}
        <div class="space-y-2 rounded bg-[var(--color-surface-sunken)] p-3 text-xs">
          <div class="flex justify-between">
            <span class="text-[var(--color-text-secondary)]"
              >{t("settings.firewallControlRule")}:</span
            >
            <span class="font-mono font-medium">{inspection.ruleName} ({inspection.status})</span>
          </div>
          <div class="flex justify-between">
            <span class="text-[var(--color-text-secondary)]">{t("settings.firewallProfile")}:</span>
            <span class="font-mono font-medium"
              >{inspection.isPolicyManaged ? "GPO / Managed" : "Local"}</span
            >
          </div>
          {#if inspection.details}
            <div class="text-[var(--color-text-muted)] pt-1">
              {inspection.details}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </Card>
</div>
