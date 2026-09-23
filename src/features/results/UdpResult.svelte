<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import { t } from "../../lib/i18n";
  import type { UdpDiagnosticResult } from "../../lib/contracts/plan";

  interface Props {
    result: UdpDiagnosticResult;
  }

  let { result }: Props = $props();

  function formatBps(bps: number | undefined | null): string {
    if (!bps || bps === 0) return "--";
    if (bps >= 1_000_000_000) {
      return `${(bps / 1_000_000_000).toFixed(2)} Gbit/s`;
    }
    return `${(bps / 1_000_000).toFixed(1)} Mbit/s`;
  }

  let tone = $derived.by<"success" | "warning" | "danger" | "neutral">(() => {
    switch (result.lossLevel) {
      case "low":
        return "success";
      case "moderate":
        return "warning";
      case "high":
        return "danger";
      default:
        return "neutral";
    }
  });

  let lossTitle = $derived.by<string>(() => {
    switch (result.lossLevel) {
      case "low":
        return t("udp.loss_low");
      case "moderate":
        return t("udp.loss_moderate");
      case "high":
        return t("udp.loss_high");
      default:
        return "No evaluable";
    }
  });
</script>

<div class="udp-result-section" role="region" aria-label={t("udp.title")}>
  <Card>
    <div class="udp-header">
      <div class="header-left">
        <h3 class="card-title">{t("udp.title")}</h3>
        <span class="loss-summary">
          {t("udp.loss")}: <strong>{result.lossPercent.toFixed(2)} %</strong>
        </span>
      </div>
      <StatusPill {tone}>
        {lossTitle}
      </StatusPill>
    </div>

    <!-- Observaciones específicas de UDP -->
    {#if result.isTargetExceeded || result.observations.length > 0}
      <div class="udp-observations" role="note">
        <Icon name="info" size={20} />
        <div class="obs-list">
          {#if result.isTargetExceeded}
            <p>{t("udp.target_exceeded")}</p>
          {/if}
          {#each result.observations as obs (obs)}
            {#if obs !== "diagnostics.udp.target_exceeded_capacity"}
              <p>{obs}</p>
            {/if}
          {/each}
        </div>
      </div>
    {/if}

    <!-- Grid de Métricas UDP -->
    <div class="metrics-grid">
      <div class="metric-card">
        <span class="metric-label">{t("udp.throughput_received")}</span>
        <strong class="metric-value">{formatBps(result.receivedRateBps)}</strong>
      </div>

      <div class="metric-card">
        <span class="metric-label">{t("udp.emitted_rate")}</span>
        <strong class="metric-value">{formatBps(result.emittedRateBps)}</strong>
      </div>

      {#if result.targetRateBps}
        <div class="metric-card">
          <span class="metric-label">{t("udp.target_rate")}</span>
          <strong class="metric-value">{formatBps(result.targetRateBps)}</strong>
        </div>
      {/if}

      <div class="metric-card">
        <span class="metric-label">{t("udp.packets_sent")}</span>
        <strong class="metric-value">{result.packetsSent.toLocaleString()}</strong>
      </div>

      <div class="metric-card">
        <span class="metric-label">{t("udp.packets_received")}</span>
        <strong class="metric-value">{result.packetsReceived.toLocaleString()}</strong>
      </div>

      <div class="metric-card">
        <span class="metric-label">{t("udp.packets_lost")}</span>
        <strong class="metric-value">{result.packetsLost.toLocaleString()}</strong>
      </div>
    </div>

    <!-- Nota informativa: sin asimetría secuencial (§18) -->
    <div class="no-asymmetry-note">
      <small>{t("udp.no_asymmetry_notice")}</small>
    </div>
  </Card>
</div>

<style>
  .udp-result-section {
    margin-top: var(--nb-space-4, 1rem);
  }

  .udp-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--nb-space-4, 1rem);
    padding-bottom: var(--nb-space-3, 0.75rem);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .card-title {
    font-size: var(--nb-font-size-base, 1rem);
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 var(--nb-space-1, 0.25rem) 0;
  }

  .loss-summary {
    font-size: var(--nb-font-size-sm, 0.875rem);
    color: var(--color-text-secondary);
  }

  .loss-summary strong {
    color: var(--color-text-primary);
  }

  .udp-observations {
    display: flex;
    align-items: flex-start;
    gap: var(--nb-space-3, 0.75rem);
    padding: var(--nb-space-3, 0.75rem) var(--nb-space-4, 1rem);
    border-radius: var(--radius-card, 8px);
    background: var(--color-surface-sunken, rgba(255, 255, 255, 0.05));
    border-left: 4px solid var(--color-info, #3b82f6);
    margin-bottom: var(--nb-space-4, 1rem);
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-primary);
  }

  .obs-list p {
    margin: 2px 0;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--nb-space-3, 0.75rem);
  }

  .metric-card {
    display: flex;
    flex-direction: column;
    padding: var(--nb-space-3, 0.75rem);
    background: var(--color-surface-sunken, rgba(0, 0, 0, 0.15));
    border-radius: var(--radius-sm, 6px);
  }

  .metric-label {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-secondary);
  }

  .metric-value {
    font-size: var(--nb-font-size-base, 1rem);
    color: var(--color-text-primary);
    margin-top: 4px;
  }

  .no-asymmetry-note {
    margin-top: var(--nb-space-3, 0.75rem);
    padding-top: var(--nb-space-2, 0.5rem);
    border-top: 1px solid var(--color-border-subtle);
    color: var(--color-text-tertiary);
    font-size: var(--nb-font-size-xs, 0.75rem);
  }
</style>
