<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import ProgressBar from "../../lib/components/ProgressBar.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import Card from "../../lib/components/Card.svelte";
  import type { IconName } from "../../lib/components/icons";
  import { t } from "../../lib/i18n";
  import type { Peer } from "../../lib/contracts/peer";
  import type { BenchmarkPlan } from "../../lib/contracts/plan";

  export type ExecutionPhase =
    | "preparing"
    | "runningSend"
    | "runningReceive"
    | "analyzing"
    | "completed"
    | "cancelling"
    | "cancelled"
    | "failed";

  interface Props {
    peer: Peer;
    plan: BenchmarkPlan;
    phase: ExecutionPhase;
    progressPercent?: number;
    currentThroughputBps?: string;
    errorMessage?: string;
    onCancel?: () => void;
    onViewResults?: () => void;
    onRetry?: () => void;
  }

  let {
    peer,
    plan,
    phase,
    progressPercent = 0,
    currentThroughputBps = "0",
    errorMessage = "",
    onCancel,
    onViewResults,
    onRetry,
  }: Props = $props();

  function formatBps(bpsStr: string): string {
    const num = Number(bpsStr);
    if (!num || isNaN(num) || num <= 0) return "0 Mbit/s";
    const mbps = num / 1_000_000;
    if (mbps < 1000) {
      return `${mbps.toFixed(1)} Mbit/s`;
    }
    return `${(mbps / 1000).toFixed(2)} Gbit/s`;
  }

  let currentInfo = $derived.by(() => {
    const infoMap: Record<
      ExecutionPhase,
      {
        title: string;
        subtitle: string;
        icon: IconName;
        tone: "info" | "success" | "warning" | "danger";
      }
    > = {
      preparing: {
        title: t("session.screen.preparingTitle"),
        subtitle: t("session.screen.preparingSubtitle"),
        icon: "clock",
        tone: "info",
      },
      runningSend: {
        title: t("session.screen.sendTitle"),
        subtitle: t("session.screen.sendSubtitle", {
          streams: plan.streams,
          peer: peer.displayName,
        }),
        icon: "arrow-right",
        tone: "info",
      },
      runningReceive: {
        title: t("session.screen.receiveTitle"),
        subtitle: t("session.screen.receiveSubtitle", {
          streams: plan.streams,
          peer: peer.displayName,
        }),
        icon: "arrow-right",
        tone: "info",
      },
      analyzing: {
        title: t("session.screen.analyzingTitle"),
        subtitle: t("session.screen.analyzingSubtitle"),
        icon: "info",
        tone: "info",
      },
      completed: {
        title: t("session.screen.completedTitle"),
        subtitle: t("session.screen.completedSubtitle"),
        icon: "check",
        tone: "success",
      },
      cancelling: {
        title: t("session.screen.cancellingTitle"),
        subtitle: t("session.screen.cancellingSubtitle"),
        icon: "alert-triangle",
        tone: "warning",
      },
      cancelled: {
        title: t("session.screen.cancelledTitle"),
        subtitle: t("session.screen.cancelledSubtitle"),
        icon: "x-circle",
        tone: "warning",
      },
      failed: {
        title: t("session.screen.failedTitle"),
        subtitle: errorMessage || t("session.screen.failedSubtitle"),
        icon: "alert-triangle",
        tone: "danger",
      },
    };
    return infoMap[phase];
  });
  let isRunning = $derived(
    phase === "preparing" ||
      phase === "runningSend" ||
      phase === "runningReceive" ||
      phase === "analyzing",
  );
</script>

<div class="nb-session-container">
  <header class="nb-session-header">
    <div class="nb-session-peer-badge">
      <Icon name="desktop" size={16} />
      <span class="nb-session-peer-name">{peer.displayName}</span>
      <span class="nb-session-peer-ip">({peer.addresses[0] ?? t("device.noAddress")})</span>
    </div>

    <!-- Botón Cancelar SIEMPRE visible durante la ejecución -->
    {#if isRunning}
      <Button
        variant="secondary"
        onclick={() => onCancel?.()}
        data-testid="session-cancel-btn"
        aria-label={t("session.screen.cancelAria")}
      >
        <Icon name="close" size={14} />
        {t("common.cancel")}
      </Button>
    {/if}
  </header>

  <Card>
    <div class="nb-session-body">
      <div class="nb-phase-head">
        <div class="nb-phase-icon" class:nb-phase-pulse={isRunning}>
          <Icon name={currentInfo.icon} size={28} />
        </div>
        <div class="nb-phase-texts">
          <div class="nb-phase-pill-row">
            <h1 class="nb-phase-title">{currentInfo.title}</h1>
            <StatusPill tone={currentInfo.tone}>
              {phase}
            </StatusPill>
          </div>
          <p class="nb-phase-subtitle">{currentInfo.subtitle}</p>
        </div>
      </div>

      {#if isRunning}
        <div class="nb-progress-section">
          <ProgressBar
            value={progressPercent}
            tone={currentInfo.tone === "info" ? "accent" : currentInfo.tone}
            label={currentInfo.title}
          />

          {#if phase === "runningSend" || phase === "runningReceive"}
            <div class="nb-live-metric" role="status" aria-live="polite">
              <span class="nb-live-label">{t("session.screen.currentSpeed")}</span>
              <span class="nb-live-value" data-testid="live-throughput"
                >{formatBps(currentThroughputBps)}</span
              >
            </div>
          {/if}
        </div>
      {/if}

      {#if phase === "completed"}
        <div class="nb-session-terminal-actions">
          <Button
            variant="primary"
            onclick={() => onViewResults?.()}
            data-testid="view-results-btn"
          >
            {t("session.screen.viewResults")}
          </Button>
        </div>
      {:else if phase === "cancelled" || phase === "failed"}
        <div class="nb-session-terminal-actions">
          <Button variant="secondary" onclick={() => onRetry?.()} data-testid="retry-btn">
            {t("results.basic.repeat")}
          </Button>
        </div>
      {/if}
    </div>
  </Card>
</div>

<style>
  .nb-session-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    padding: var(--space-6);
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
  }

  .nb-session-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .nb-session-peer-badge {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    background: var(--color-surface);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
  }

  .nb-session-peer-name {
    font-weight: var(--font-weight-title);
    color: var(--color-text-primary);
  }

  .nb-session-peer-ip {
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }

  .nb-session-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    padding: var(--space-4);
  }

  .nb-phase-head {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
  }

  .nb-phase-icon {
    width: 52px;
    height: 52px;
    border-radius: var(--radius-full);
    background: var(--icon-chip-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-accent);
    flex-shrink: 0;
  }

  .nb-phase-pulse {
    animation: nb-pulse 2s infinite ease-in-out;
  }

  @keyframes nb-pulse {
    0%,
    100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.8;
      transform: scale(1.04);
    }
  }

  .nb-phase-texts {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    flex: 1;
  }

  .nb-phase-pill-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .nb-phase-title {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-title);
    color: var(--color-text-primary);
    margin: 0;
  }

  .nb-phase-subtitle {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: 0;
  }

  .nb-progress-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-top: var(--space-2);
  }

  .nb-live-metric {
    display: flex;
    align-items: baseline;
    justify-content: center;
    gap: var(--space-2);
    font-size: var(--font-size-md);
  }

  .nb-live-label {
    color: var(--color-text-secondary);
  }

  .nb-live-value {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-title);
    font-variant-numeric: tabular-nums;
    color: var(--color-accent);
  }

  .nb-session-terminal-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
    padding-top: var(--space-4);
    border-top: 1px solid var(--color-border);
  }
</style>
