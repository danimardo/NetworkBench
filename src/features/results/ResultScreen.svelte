<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import { t } from "../../lib/i18n";
  import type { SessionResult } from "../../lib/contracts/session-result";
  import type { SamplePoint } from "../../lib/api/samples";
  import VerdictCard from "./VerdictCard.svelte";
  import MetricCard from "./MetricCard.svelte";
  import ThroughputChart from "./ThroughputChart.svelte";
  import TechnicalDetails from "./TechnicalDetails.svelte";
  import { ExportDialog } from "../export";

  interface Props {
    result: SessionResult;
    samplesForward?: SamplePoint[];
    samplesReverse?: SamplePoint[];
    onRetest?: () => void;
    onNewTest?: () => void;
  }

  let { result, samplesForward = [], samplesReverse = [], onRetest, onNewTest }: Props = $props();

  let showDetails = $state(false);
  let showExport = $state(false);

  const forwardDir = $derived(result.directions.find((d) => d.direction === "forward"));
  const reverseDir = $derived(result.directions.find((d) => d.direction === "reverse"));

  const minSpeedBps = $derived(() => {
    const f = forwardDir?.officialBps ? Number(forwardDir.officialBps) : null;
    const r = reverseDir?.officialBps ? Number(reverseDir.officialBps) : null;
    if (f != null && r != null) return Math.min(f, r).toString();
    if (f != null) return f.toString();
    if (r != null) return r.toString();
    return null;
  });

  const verdict = $derived(result.verdict);
</script>

<div class="result-screen" data-testid="result-screen">
  <!-- Nivel 1: Tarjeta principal de veredicto y velocidad -->
  <VerdictCard
    {verdict}
    officialBps={minSpeedBps()}
    capacityBps={result.capacity?.refBps}
    refSource={result.capacity?.refSource}
  />

  <!-- Nivel 2: Métricas estructuradas -->
  <div class="metrics-grid">
    <MetricCard type="directions" title={t("results.details")} {forwardDir} {reverseDir} />
    <MetricCard
      type="stability"
      title="Estabilidad de conexión"
      helpText={t("tooltips.stability")}
      {forwardDir}
      {reverseDir}
    />
    <MetricCard
      type="retransmission"
      title="Pérdidas y retransmisiones"
      helpText={t("tooltips.retransmissions")}
      {forwardDir}
      {reverseDir}
    />
    <MetricCard
      type="cpu"
      title={t("results.cpuUsage")}
      helpText={t("tooltips.cpu_usage")}
      {forwardDir}
      {reverseDir}
    />
  </div>

  <!-- Gráfica SVG accesible -->
  <div class="chart-section">
    <ThroughputChart
      {samplesForward}
      {samplesReverse}
      warmupSeconds={result.plan.warmupSeconds}
      measureSeconds={result.plan.measureSeconds}
      cooldownSeconds={result.plan.cooldownSeconds}
    />
  </div>

  <!-- Conclusión y diagnóstico guiado (Hechos / Observaciones / Causas / Acciones) -->
  {#if verdict}
    <div class="diagnostic-sections" data-testid="diagnostic-conclusion">
      {#if verdict.facts.length > 0}
        <div class="diag-block facts-block" data-testid="facts-block">
          <h4 class="diag-title">
            <Icon name="check" size={16} />
            {t("verdict.facts_title")}
          </h4>
          <ul class="diag-list">
            {#each verdict.facts as f (f.ruleId)}
              <li>{t(f.messageKey) || f.messageKey}</li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if verdict.observations.length > 0}
        <div class="diag-block observations-block" data-testid="observations-block">
          <h4 class="diag-title">
            <Icon name="info" size={16} />
            {t("verdict.observations_title")}
          </h4>
          <ul class="diag-list">
            {#each verdict.observations as o (o.ruleId)}
              <li>{t(o.messageKey) || o.messageKey}</li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if verdict.possibleCauses.length > 0}
        <div class="diag-block causes-block" data-testid="causes-block">
          <h4 class="diag-title">
            <Icon name="alert-triangle" size={16} />
            {t("verdict.causes_title")}
          </h4>
          <ul class="diag-list">
            {#each verdict.possibleCauses as c (c.ruleId)}
              <li>{t(c.messageKey) || c.messageKey}</li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if verdict.actions.length > 0}
        <div class="diag-block actions-block" data-testid="actions-block">
          <h4 class="diag-title">
            <Icon name="arrow-right" size={16} />
            {t("verdict.actions_title")}
          </h4>
          <ul class="diag-list">
            {#each verdict.actions as a (a.ruleId)}
              <li>{t(a.messageKey) || a.messageKey}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Acciones principales de cierre de sesión -->
  <div class="actions-toolbar">
    <Button variant="secondary" onclick={() => (showDetails = true)} data-testid="details-btn">
      <Icon name="file-text" size={16} />
      {t("verdict.technical_details")}
    </Button>

    <Button variant="secondary" onclick={() => (showExport = true)} data-testid="export-btn">
      <Icon name="external-link" size={16} />
      {t("export.action")}
    </Button>

    <div class="right-buttons">
      {#if onRetest}
        <Button variant="secondary" onclick={onRetest} data-testid="retest-btn">
          <Icon name="refresh" size={16} />
          {t("verdict.repeat")}
        </Button>
      {/if}

      {#if onNewTest}
        <Button variant="primary" onclick={onNewTest} data-testid="new-test-btn">
          <Icon name="plus" size={16} />
          {t("verdict.new_test")}
        </Button>
      {/if}
    </div>
  </div>

  <!-- Diálogo modal de detalles técnicos -->
  <TechnicalDetails open={showDetails} {result} onClose={() => (showDetails = false)} />

  <!-- Diálogo de exportación -->
  <ExportDialog
    open={showExport}
    sessionIds={[result.sessionId]}
    peerName={result.responder.displayName}
    onclose={() => (showExport = false)}
  />
</div>

<style>
  .result-screen {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-4, 16px);
    padding: var(--spacing-4, 16px);
    max-width: 1200px;
    margin: 0 auto;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: var(--spacing-3, 12px);
  }

  .chart-section {
    margin-top: var(--spacing-2, 8px);
  }

  .diagnostic-sections {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: var(--spacing-3, 12px);
    margin-top: var(--spacing-2, 8px);
  }

  .diag-block {
    background: var(--bg-card, rgba(30, 41, 59, 0.5));
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-lg, 12px);
    padding: var(--spacing-3, 12px);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2, 8px);
  }

  .diag-title {
    display: flex;
    align-items: center;
    gap: var(--spacing-2, 8px);
    font-size: var(--font-size-sm, 0.875rem);
    font-weight: var(--font-weight-semibold, 600);
    margin: 0;
  }

  .facts-block .diag-title {
    color: var(--color-emerald, #10b981);
  }

  .observations-block .diag-title {
    color: var(--color-cyan, #06b6d4);
  }

  .causes-block .diag-title {
    color: var(--tone-warning, #f59e0b);
  }

  .actions-block .diag-title {
    color: var(--accent, #8b5cf6);
  }

  .diag-list {
    margin: 0;
    padding-left: var(--spacing-4, 16px);
    font-size: var(--font-size-xs, 0.75rem);
    color: var(--text-secondary, #94a3b8);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1, 4px);
  }

  .actions-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: var(--spacing-3, 12px);
    border-top: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    margin-top: var(--spacing-2, 8px);
  }

  .right-buttons {
    display: flex;
    gap: var(--spacing-2, 8px);
  }
</style>
