<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import Card from "../../lib/components/Card.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import HistoryActions from "./HistoryActions.svelte";
  import { t } from "../../lib/i18n";
  import type {
    CohortComparisonResult,
    RepeatPlanConfig,
    SessionRecord,
  } from "../../lib/contracts/history";

  interface Props {
    session: SessionRecord;
    cohortComparison?: CohortComparisonResult | null;
    onBack: () => void;
    onRepeat?: (config: RepeatPlanConfig) => void;
    onDelete?: () => void;
    onExport?: () => void;
  }

  let { session, cohortComparison, onBack, onRepeat, onDelete, onExport }: Props = $props();

  function formatBps(bps: string | null | undefined): string {
    if (!bps) return "--";
    const num = Number(bps);
    if (isNaN(num)) return "--";
    if (num >= 1_000_000_000) {
      return `${(num / 1_000_000_000).toFixed(2)} Gbit/s`;
    }
    return `${(num / 1_000_000).toFixed(1)} Mbit/s`;
  }
</script>

<div class="history-detail" role="region" aria-label="Detalle histórico de prueba">
  <div class="nav-bar">
    <Button variant="ghost" onclick={onBack}>
      ← {t("history.back_to_list")}
    </Button>
  </div>

  <Card>
    <div class="detail-header">
      <div>
        <h2 class="session-title">
          {session.protocol.toUpperCase()} — {session.createdAt.slice(0, 16).replace("T", " ")}
        </h2>
        <span class="session-id">ID: {session.id}</span>
      </div>
      <StatusPill tone={session.isPartial ? "warning" : "success"}>
        {session.isPartial ? t("history.incomplete_tag") : "COMPLETADA"}
      </StatusPill>
    </div>

    <!-- Observación de evolución histórica si aplica (§19.4) -->
    {#if cohortComparison?.observationText}
      <div class="cohort-observation" role="note">
        <Icon name="info" size={20} />
        <span>{cohortComparison.observationText}</span>
      </div>
    {/if}

    <div class="metrics-grid">
      <div class="metric-box">
        <span class="metric-label">Envío (Subida)</span>
        <strong class="metric-val">{formatBps(session.forwardBps)}</strong>
      </div>
      <div class="metric-box">
        <span class="metric-label">Recepción (Bajada)</span>
        <strong class="metric-val">{formatBps(session.reverseBps)}</strong>
      </div>
      <div class="metric-box">
        <span class="metric-label">Streams</span>
        <strong class="metric-val">{session.streams}</strong>
      </div>
      <div class="metric-box">
        <span class="metric-label">Duración</span>
        <strong class="metric-val">{session.durationSeconds} s</strong>
      </div>
    </div>

    {#if session.clientInterface || session.serverInterface}
      <div class="interfaces-box">
        <h4>Adaptadores de red utilizados</h4>
        <p>Cliente local: {session.clientInterface ?? "No especificado"}</p>
        <p>Servidor remoto: {session.serverInterface ?? "No especificado"}</p>
      </div>
    {/if}

    <HistoryActions sessionId={session.id} {onRepeat} {onDelete} {onExport} />
  </Card>
</div>

<style>
  .history-detail {
    max-width: 800px;
    margin: 0 auto;
  }

  .nav-bar {
    margin-bottom: var(--nb-space-4, 1rem);
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--nb-space-4, 1rem);
    padding-bottom: var(--nb-space-3, 0.75rem);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .session-title {
    font-size: var(--nb-font-size-lg, 1.125rem);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .session-id {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-tertiary);
    font-family: var(--font-mono, monospace);
  }

  .cohort-observation {
    display: flex;
    align-items: center;
    gap: var(--nb-space-3, 0.75rem);
    padding: var(--nb-space-3, 0.75rem) var(--nb-space-4, 1rem);
    border-radius: var(--radius-card, 8px);
    background: var(--color-surface-sunken, rgba(255, 255, 255, 0.05));
    border-left: 4px solid var(--color-info, #3b82f6);
    margin-bottom: var(--nb-space-4, 1rem);
    font-size: var(--nb-font-size-sm, 0.875rem);
    color: var(--color-text-primary);
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--nb-space-3, 0.75rem);
    margin-bottom: var(--nb-space-4, 1rem);
  }

  .metric-box {
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

  .metric-val {
    font-size: var(--nb-font-size-base, 1rem);
    color: var(--color-text-primary);
  }

  .interfaces-box {
    margin-top: var(--nb-space-3, 0.75rem);
    padding: var(--nb-space-3, 0.75rem);
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-secondary);
    border-top: 1px solid var(--color-border-subtle);
  }
</style>
