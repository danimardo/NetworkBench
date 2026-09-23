<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import HelpTooltip from "../../lib/components/HelpTooltip.svelte";
  import type { DirectionResult } from "../../lib/contracts/session-result";
  import { formatThroughput } from "./model";

  interface Props {
    title: string;
    helpText?: string;
    forwardDir?: DirectionResult | null;
    reverseDir?: DirectionResult | null;
    type: "directions" | "stability" | "retransmission" | "cpu";
  }

  let { title, helpText, forwardDir, reverseDir, type }: Props = $props();

  const forwardSpeed = $derived(formatThroughput(forwardDir?.officialBps));
  const reverseSpeed = $derived(formatThroughput(reverseDir?.officialBps));

  const forwardStability = $derived(forwardDir?.stability);
  const reverseStability = $derived(reverseDir?.stability);

  const forwardRetrans = $derived(forwardDir?.retransmission);
  const reverseRetrans = $derived(reverseDir?.retransmission);

  const forwardCpu = $derived(forwardDir?.cpuReceiver);
  const reverseCpu = $derived(reverseDir?.cpuReceiver);
</script>

<div class="metric-card" data-testid="metric-card-{type}">
  <Card variant="default">
    <div class="card-header">
      <h3 class="card-title">{title}</h3>
      {#if helpText}
        <HelpTooltip text={helpText} label="Información sobre {title}" />
      {/if}
    </div>

    <div class="card-body">
      {#if type === "directions"}
        <div class="directions-grid">
          <div class="dir-item">
            <div class="dir-label">
              <Icon name="arrow-right" size={14} /> Subida (A → B)
            </div>
            <div class="dir-speed">{forwardSpeed}</div>
          </div>
          <div class="dir-item">
            <div class="dir-label">
              <Icon name="arrow-left" size={14} /> Bajada (B → A)
            </div>
            <div class="dir-speed">{reverseSpeed}</div>
          </div>
        </div>
      {:else if type === "stability"}
        <div class="stats-list">
          <div class="stat-row">
            <span class="stat-name">Regularidad A → B:</span>
            <span class="stat-val"
              >{forwardStability ? forwardStability.level : "No evaluable"}</span
            >
          </div>
          <div class="stat-row">
            <span class="stat-name">Regularidad B → A:</span>
            <span class="stat-val"
              >{reverseStability ? reverseStability.level : "No evaluable"}</span
            >
          </div>
          {#if forwardStability?.dropsCount || reverseStability?.dropsCount}
            <div class="stat-row warning-text">
              <span class="stat-name">Caídas momentáneas:</span>
              <span class="stat-val"
                >{(forwardStability?.dropsCount ?? 0) + (reverseStability?.dropsCount ?? 0)} detectadas</span
              >
            </div>
          {/if}
        </div>
      {:else if type === "retransmission"}
        <div class="stats-list">
          <div class="stat-row">
            <span class="stat-name">Pérdida/Retransmisión A → B:</span>
            <span class="stat-val">{forwardRetrans ? forwardRetrans.level : "N/A"}</span>
          </div>
          <div class="stat-row">
            <span class="stat-name">Pérdida/Retransmisión B → A:</span>
            <span class="stat-val">{reverseRetrans ? reverseRetrans.level : "N/A"}</span>
          </div>
        </div>
      {:else if type === "cpu"}
        <div class="stats-list">
          <div class="stat-row">
            <span class="stat-name">Uso receptor A → B:</span>
            <span class="stat-val">{forwardCpu != null ? `${forwardCpu.toFixed(1)}%` : "N/A"}</span>
          </div>
          <div class="stat-row">
            <span class="stat-name">Uso receptor B → A:</span>
            <span class="stat-val">{reverseCpu != null ? `${reverseCpu.toFixed(1)}%` : "N/A"}</span>
          </div>
        </div>
      {/if}
    </div>
  </Card>
</div>

<style>
  .metric-card {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-2, 8px);
  }

  .card-title {
    font-size: var(--font-size-sm, 0.875rem);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--text-secondary, #94a3b8);
    margin: 0;
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2, 8px);
  }

  .directions-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-2, 8px);
  }

  .dir-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .dir-label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: var(--font-size-xs, 0.75rem);
    color: var(--text-muted, #64748b);
  }

  .dir-speed {
    font-size: var(--font-size-base, 1rem);
    font-weight: var(--font-weight-bold, 700);
    color: var(--text-primary, #ffffff);
  }

  .stats-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1, 4px);
    font-size: var(--font-size-xs, 0.75rem);
  }

  .stat-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .stat-name {
    color: var(--text-secondary, #94a3b8);
  }

  .stat-val {
    font-weight: var(--font-weight-medium, 500);
    color: var(--text-primary, #ffffff);
  }

  .warning-text {
    color: var(--tone-warning, #f59e0b);
  }
</style>
