<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import HelpTooltip from "../../lib/components/HelpTooltip.svelte";
  import { t } from "../../lib/i18n";
  import type { SessionVerdict } from "../../lib/contracts/session-result";
  import { formatThroughput } from "./model";

  interface Props {
    verdict?: SessionVerdict | null;
    officialBps?: string | null;
    capacityBps?: number | null;
    refSource?: string | null;
  }

  let { verdict, officialBps, capacityBps, refSource }: Props = $props();

  const level = $derived(verdict?.level ?? "notEvaluable");

  const titleText = $derived(() => {
    if (!verdict) return t("verdict.not_evaluable_title");
    return t(verdict.titleKey) || verdict.titleKey;
  });

  const speedFormatted = $derived(formatThroughput(officialBps));

  const capacityText = $derived(() => {
    if (!capacityBps || capacityBps <= 0) {
      if (refSource === "wifi") {
        return "Conexión inalámbrica (capacidad de enlace no determinable)";
      }
      return "Capacidad de referencia no determinable";
    }

    const num = Number(officialBps || 0);
    const util = Math.round((num / capacityBps) * 100);
    const capFormatted = formatThroughput(capacityBps.toString());
    return `${util}% de un enlace de ${capFormatted}`;
  });

  const pillTone = $derived(() => {
    switch (level) {
      case "ok":
        return "success";
      case "warn":
        return "warning";
      case "problem":
        return "danger";
      default:
        return "neutral";
    }
  });
</script>

<div class="verdict-card" data-testid="verdict-card">
  <Card variant="default">
    <div class="card-inner">
      <div class="header-row">
        <StatusPill tone={pillTone()}>
          {titleText()}
        </StatusPill>
        <HelpTooltip
          text={t("tooltips.official_bps")}
          label="Explicación del veredicto y velocidad oficial"
        />
      </div>

      <div class="hero-content">
        <div class="speed-value" data-testid="main-speed">
          {speedFormatted}
        </div>
        <div class="capacity-subtitle" data-testid="capacity-text">
          {capacityText()}
        </div>
      </div>

      <div class="verdict-title" data-testid="verdict-title">
        {titleText()}
      </div>

      <p class="disclaimer-note">
        La velocidad oficial procede exclusivamente de los datos recibidos por el receptor en
        destino.
      </p>
    </div>
  </Card>
</div>

<style>
  .verdict-card {
    margin-bottom: var(--spacing-4, 16px);
  }

  .card-inner {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3, 12px);
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .hero-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1, 4px);
  }

  .speed-value {
    font-size: var(--font-size-4xl, 2.5rem);
    font-weight: var(--font-weight-bold, 700);
    line-height: 1.1;
    color: var(--text-primary, #ffffff);
    letter-spacing: -0.02em;
  }

  .capacity-subtitle {
    font-size: var(--font-size-base, 1rem);
    color: var(--text-secondary, #94a3b8);
  }

  .verdict-title {
    font-size: var(--font-size-lg, 1.125rem);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--text-primary, #ffffff);
    padding-top: var(--spacing-2, 8px);
    border-top: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
  }

  .disclaimer-note {
    font-size: var(--font-size-xs, 0.75rem);
    color: var(--text-muted, #64748b);
    margin: 0;
  }
</style>
