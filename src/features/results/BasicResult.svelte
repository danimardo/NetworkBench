<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import { formatThroughput, type BasicResultModel } from "./model";

  interface Props {
    result: BasicResultModel;
    onRepeat?: () => void;
    onBackToPeers?: () => void;
  }

  let { result, onRepeat, onBackToPeers }: Props = $props();
</script>

<div class="nb-result-container">
  <header class="nb-result-header">
    <div class="nb-result-title-group">
      <div class="nb-result-title-row">
        <h1 class="nb-result-title">Resultado de la medición</h1>
        <StatusPill tone="success">Completada</StatusPill>
      </div>
      <p class="nb-result-subtitle">
        Medición bidireccional secuencial con {result.peer.displayName} ({result.peer
          .addresses[0] ?? "red local"}).
      </p>
    </div>

    <div class="nb-result-header-actions">
      <Button variant="secondary" onclick={() => onRepeat?.()} data-testid="result-repeat-btn">
        <Icon name="refresh" size={14} />
        Repetir prueba
      </Button>
      <Button variant="primary" onclick={() => onBackToPeers?.()} data-testid="result-back-btn">
        Volver a equipos
      </Button>
    </div>
  </header>

  <div class="nb-result-cards-grid">
    <!-- Sentido A -> B (Envío) -->
    <Card>
      <div class="nb-direction-card">
        <div class="nb-direction-head">
          <div class="nb-direction-icon">
            <Icon name="arrow-right" size={20} />
          </div>
          <div>
            <h2 class="nb-direction-title">Envío (A → B)</h2>
            <span class="nb-direction-desc">Desde este equipo hacia {result.peer.displayName}</span>
          </div>
        </div>

        <div class="nb-direction-throughput">
          <span class="nb-throughput-number" data-testid="forward-throughput">
            {formatThroughput(result.forwardOfficialBps)}
          </span>
          <span class="nb-throughput-source">Medición oficial del receptor</span>
        </div>
      </div>
    </Card>

    <!-- Sentido B -> A (Recepción) -->
    <Card>
      <div class="nb-direction-card">
        <div class="nb-direction-head">
          <div class="nb-direction-icon">
            <Icon name="arrow-right" size={20} />
          </div>
          <div>
            <h2 class="nb-direction-title">Recepción (B → A)</h2>
            <span class="nb-direction-desc">Desde {result.peer.displayName} hacia este equipo</span>
          </div>
        </div>

        <div class="nb-direction-throughput">
          <span class="nb-throughput-number" data-testid="reverse-throughput">
            {formatThroughput(result.reverseOfficialBps)}
          </span>
          <span class="nb-throughput-source">Medición oficial del receptor</span>
        </div>
      </div>
    </Card>
  </div>

  <Card>
    <div class="nb-result-meta-row">
      <div class="nb-meta-item">
        <span class="nb-meta-label">Protocolo:</span>
        <span class="nb-meta-value"
          >{result.plan.protocol.toUpperCase()} ({result.plan.streams} stream)</span
        >
      </div>
      <div class="nb-meta-item">
        <span class="nb-meta-label">Duración por sentido:</span>
        <span class="nb-meta-value">{result.durationSeconds} s</span>
      </div>
      <div class="nb-meta-item">
        <span class="nb-meta-label">Fecha y hora:</span>
        <span class="nb-meta-value">{result.completedAt}</span>
      </div>
    </div>
  </Card>
</div>

<style>
  .nb-result-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    padding: var(--space-6);
    max-width: 900px;
    margin: 0 auto;
    width: 100%;
  }

  .nb-result-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .nb-result-title-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .nb-result-title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-title);
    color: var(--color-text-primary);
    margin: 0;
  }

  .nb-result-subtitle {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: var(--space-1) 0 0 0;
  }

  .nb-result-header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .nb-result-cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: var(--space-4);
  }

  .nb-direction-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-2);
  }

  .nb-direction-head {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .nb-direction-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-sm);
    background: var(--icon-chip-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-accent);
  }

  .nb-direction-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-title);
    color: var(--color-text-primary);
    margin: 0;
  }

  .nb-direction-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
  }

  .nb-direction-throughput {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding-top: var(--space-2);
  }

  .nb-throughput-number {
    font-size: var(--font-size-2xl);
    font-weight: var(--font-weight-title);
    font-variant-numeric: tabular-nums;
    color: var(--color-accent);
  }

  .nb-throughput-source {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
  }

  .nb-result-meta-row {
    display: flex;
    align-items: center;
    justify-content: space-around;
    padding: var(--space-2);
  }

  .nb-meta-item {
    display: flex;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
  }

  .nb-meta-label {
    color: var(--color-text-muted);
  }

  .nb-meta-value {
    color: var(--color-text-primary);
    font-weight: var(--font-weight-subtitle);
  }
</style>
