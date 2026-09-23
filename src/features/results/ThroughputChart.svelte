<script lang="ts">
  import type { SamplePoint } from "../../lib/api/samples";

  interface Props {
    samplesForward?: SamplePoint[];
    samplesReverse?: SamplePoint[];
    warmupSeconds?: number;
    measureSeconds?: number;
    cooldownSeconds?: number;
  }

  let {
    samplesForward = [],
    samplesReverse = [],
    warmupSeconds = 2,
    measureSeconds = 20,
    cooldownSeconds = 2,
  }: Props = $props();

  // Determinar unidad única: Mbit/s o Gbit/s
  const maxBps = $derived(() => {
    let max = 0;
    for (const s of samplesForward) {
      if (s.bps > max) max = s.bps;
    }
    for (const s of samplesReverse) {
      if (s.bps > max) max = s.bps;
    }
    return Math.max(max, 1_000_000);
  });

  const useGbps = $derived(maxBps() >= 1_000_000_000);
  const unitLabel = $derived(useGbps ? "Gbit/s" : "Mbit/s");
  const divisor = $derived(useGbps ? 1_000_000_000 : 1_000_000);

  const totalDurationSeconds = $derived(warmupSeconds + measureSeconds + cooldownSeconds);

  const chartWidth = 600;
  const chartHeight = 220;
  const paddingLeft = 60;
  const paddingRight = 20;
  const paddingTop = 20;
  const paddingBottom = 30;

  const innerWidth = chartWidth - paddingLeft - paddingRight;
  const innerHeight = chartHeight - paddingTop - paddingBottom;

  const yMax = $derived(() => {
    const rawMax = maxBps() / divisor;
    return Math.ceil(rawMax * 1.15) || 1;
  });

  function getX(tSeconds: number): number {
    const frac = Math.min(Math.max(tSeconds / (totalDurationSeconds || 1), 0), 1);
    return paddingLeft + frac * innerWidth;
  }

  function getY(value: number): number {
    const frac = Math.min(Math.max(value / yMax(), 0), 1);
    return paddingTop + (1 - frac) * innerHeight;
  }

  function buildPath(samples: SamplePoint[]): string {
    if (!samples.length) return "";
    let d = "";
    let inGap = false;

    for (let i = 0; i < samples.length; i++) {
      const pt = samples[i];
      if (!pt) continue;
      if (pt.gap) {
        inGap = true;
        continue;
      }
      const x = getX(pt.tMs / 1000);
      const y = getY(pt.bps / divisor);

      if (d === "" || inGap) {
        d += `M ${x.toFixed(1)} ${y.toFixed(1)} `;
        inGap = false;
      } else {
        d += `L ${x.toFixed(1)} ${y.toFixed(1)} `;
      }
    }
    return d;
  }

  const forwardPath = $derived(buildPath(samplesForward));
  const reversePath = $derived(buildPath(samplesReverse));

  const warmupX = $derived(getX(warmupSeconds));
  const measureEndX = $derived(getX(warmupSeconds + measureSeconds));
</script>

<div
  class="throughput-chart-container"
  role="group"
  aria-label="Gráfica temporal de velocidad de transferencia en {unitLabel}"
  data-testid="throughput-chart"
>
  <div class="chart-header">
    <div class="legend-row">
      <span class="legend-item">
        <span class="line-swatch line-forward"></span> Subida A → B (trazo continuo)
      </span>
      <span class="legend-item">
        <span class="line-swatch line-reverse"></span> Bajada B → A (trazo discontinuo)
      </span>
    </div>
    <span class="unit-badge">{unitLabel}</span>
  </div>

  <svg viewBox="0 0 {chartWidth} {chartHeight}" class="chart-svg" aria-hidden="true">
    <!-- Zona de calentamiento (warmup) sombreada -->
    {#if warmupSeconds > 0}
      <rect
        x={paddingLeft}
        y={paddingTop}
        width={warmupX - paddingLeft}
        height={innerHeight}
        class="zone-shaded"
      />
    {/if}

    <!-- Zona de enfriamiento (cooldown) sombreada -->
    {#if cooldownSeconds > 0}
      <rect
        x={measureEndX}
        y={paddingTop}
        width={chartWidth - paddingRight - measureEndX}
        height={innerHeight}
        class="zone-shaded"
      />
    {/if}

    <!-- Eje Y: Guías y ticks -->
    {#each [0, 0.25, 0.5, 0.75, 1.0] as tickFrac (tickFrac)}
      {@const val = (yMax() * tickFrac).toFixed(1)}
      {@const y = paddingTop + (1 - tickFrac) * innerHeight}
      <line x1={paddingLeft} y1={y} x2={chartWidth - paddingRight} y2={y} class="grid-line" />
      <text x={paddingLeft - 8} y={y + 4} class="axis-label y-axis" text-anchor="end">
        {val}
      </text>
    {/each}

    <!-- Eje X: marcas de tiempo -->
    <text x={paddingLeft} y={chartHeight - 8} class="axis-label" text-anchor="start">0s</text>
    <text x={warmupX} y={chartHeight - 8} class="axis-label" text-anchor="middle"
      >{warmupSeconds}s</text
    >
    <text x={measureEndX} y={chartHeight - 8} class="axis-label" text-anchor="middle"
      >{warmupSeconds + measureSeconds}s</text
    >
    <text x={chartWidth - paddingRight} y={chartHeight - 8} class="axis-label" text-anchor="end"
      >{totalDurationSeconds}s</text
    >

    <!-- Línea Forward (A -> B, violeta sólido) -->
    {#if forwardPath}
      <path d={forwardPath} class="chart-line line-forward-svg" />
    {/if}

    <!-- Línea Reverse (B -> A, verde/azul discontinuo) -->
    {#if reversePath}
      <path d={reversePath} class="chart-line line-reverse-svg" stroke-dasharray="6,4" />
    {/if}
  </svg>

  <p class="chart-disclaimer">
    Nota: Las muestras periódicas reflejan el caudal de la interfaz y pueden incluir tráfico ajeno a
    la prueba (FR-027). La velocidad oficial certificada proviene del cálculo consolidado del
    receptor.
  </p>
</div>

<style>
  .throughput-chart-container {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2, 8px);
    background: var(--bg-card, rgba(30, 41, 59, 0.5));
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-lg, 12px);
    padding: var(--spacing-3, 12px);
    outline: none;
  }

  .throughput-chart-container:focus-visible {
    border-color: var(--accent, #8b5cf6);
  }

  .chart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .legend-row {
    display: flex;
    gap: var(--spacing-3, 12px);
    font-size: var(--font-size-xs, 0.75rem);
    color: var(--text-secondary, #94a3b8);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .line-swatch {
    display: inline-block;
    width: 16px;
    height: 3px;
    border-radius: 2px;
  }

  .line-forward {
    background-color: var(--accent, #8b5cf6);
  }

  .line-reverse {
    background-color: var(--color-cyan, #06b6d4);
    border-top: 1px dashed var(--color-cyan, #06b6d4);
  }

  .unit-badge {
    font-size: var(--font-size-xs, 0.75rem);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--text-muted, #64748b);
  }

  .chart-svg {
    width: 100%;
    height: auto;
    overflow: visible;
  }

  .zone-shaded {
    fill: var(--bg-hover, rgba(255, 255, 255, 0.04));
  }

  .grid-line {
    stroke: var(--border-subtle, rgba(255, 255, 255, 0.06));
    stroke-width: 1;
  }

  .axis-label {
    font-size: 10px;
    fill: var(--text-muted, #64748b);
    font-family: inherit;
  }

  .chart-line {
    fill: none;
    stroke-width: 2.5;
    stroke-linejoin: round;
    stroke-linecap: round;
  }

  .line-forward-svg {
    stroke: var(--accent, #8b5cf6);
  }

  .line-reverse-svg {
    stroke: var(--color-cyan, #06b6d4);
  }

  .chart-disclaimer {
    font-size: var(--font-size-xs, 0.75rem);
    color: var(--text-muted, #64748b);
    margin: 0;
  }
</style>
