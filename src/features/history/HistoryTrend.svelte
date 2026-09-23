<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import { t } from "../../lib/i18n";
  import type { PeerTrendPoint } from "../../lib/contracts/history";

  interface Props {
    points: PeerTrendPoint[];
    peerName: string;
  }

  let { points, peerName }: Props = $props();

  function parseBps(val: string | null | undefined): number {
    if (!val) return 0;
    const n = Number(val);
    return isNaN(n) ? 0 : n;
  }

  let validPoints = $derived.by(() => {
    return points
      .map((p) => ({
        ...p,
        forwardBpsNum: parseBps(p.forwardBps),
        reverseBpsNum: parseBps(p.reverseBps),
      }))
      .filter((p) => p.forwardBpsNum > 0 || p.reverseBpsNum > 0);
  });

  let maxBps = $derived.by(() => {
    if (validPoints.length === 0) return 1_000_000_000;
    const max = Math.max(...validPoints.map((p) => Math.max(p.forwardBpsNum, p.reverseBpsNum)));
    return max > 0 ? max * 1.15 : 1_000_000_000;
  });

  function formatSpeed(bps: number): string {
    if (bps >= 1_000_000_000) {
      return `${(bps / 1_000_000_000).toFixed(2)} Gbit/s`;
    }
    return `${(bps / 1_000_000).toFixed(1)} Mbit/s`;
  }

  function getVerdictColor(verdictStr: string | null | undefined): string {
    if (!verdictStr) return "var(--color-text-tertiary)";
    if (verdictStr.includes("ok")) return "var(--color-success)";
    if (verdictStr.includes("warn")) return "var(--color-warning)";
    if (verdictStr.includes("problem")) return "var(--color-danger)";
    return "var(--color-accent)";
  }

  const width = 600;
  const height = 180;
  const padLeft = 70;
  const padRight = 30;
  const padTop = 20;
  const padBottom = 30;

  let plotW = $derived(width - padLeft - padRight);
  let plotH = $derived(height - padTop - padBottom);

  function getX(index: number, total: number): number {
    if (total <= 1) return padLeft + plotW / 2;
    return padLeft + (index / (total - 1)) * plotW;
  }

  function getY(bps: number): number {
    const ratio = Math.min(Math.max(bps / maxBps, 0), 1);
    return padTop + plotH - ratio * plotH;
  }
</script>

<div class="trend-container" role="region" aria-label="Evolución con este equipo">
  <Card>
    <div class="trend-header">
      <h3 class="trend-title">{t("history.evolution_title")}: {peerName}</h3>
      <span class="trend-badge">{validPoints.length} pruebas registradas</span>
    </div>

    {#if validPoints.length === 0}
      <p class="trend-empty">No hay suficientes mediciones completadas para trazar la evolución.</p>
    {:else}
      <div class="chart-wrapper">
        <svg
          viewBox={`0 0 ${width} ${height}`}
          class="trend-svg"
          role="img"
          aria-label={`Gráfica de evolución de velocidad con ${peerName}`}
        >
          <!-- Eje Y líneas -->
          <line
            x1={padLeft}
            y1={padTop}
            x2={width - padRight}
            y2={padTop}
            stroke="var(--color-border-subtle)"
            stroke-dasharray="3,3"
          />
          <line
            x1={padLeft}
            y1={padTop + plotH / 2}
            x2={width - padRight}
            y2={padTop + plotH / 2}
            stroke="var(--color-border-subtle)"
            stroke-dasharray="3,3"
          />
          <line
            x1={padLeft}
            y1={padTop + plotH}
            x2={width - padRight}
            y2={padTop + plotH}
            stroke="var(--color-border-subtle)"
          />

          <!-- Etiquetas Eje Y -->
          <text x={padLeft - 8} y={padTop + 4} class="axis-label" text-anchor="end">
            {formatSpeed(maxBps)}
          </text>
          <text x={padLeft - 8} y={padTop + plotH / 2 + 4} class="axis-label" text-anchor="end">
            {formatSpeed(maxBps / 2)}
          </text>
          <text x={padLeft - 8} y={padTop + plotH + 4} class="axis-label" text-anchor="end">
            0
          </text>

          <!-- Línea de conexión forward (Subida) -->
          {#if validPoints.length > 1}
            <polyline
              points={validPoints
                .map((p, i) => `${getX(i, validPoints.length)},${getY(p.forwardBpsNum)}`)
                .join(" ")}
              fill="none"
              stroke="var(--color-accent)"
              stroke-width="2"
            />
          {/if}

          <!-- Puntos individuales con color de veredicto -->
          {#each validPoints as p, i (p.sessionId)}
            {@const cx = getX(i, validPoints.length)}
            {@const cy = getY(p.forwardBpsNum)}
            <circle
              {cx}
              {cy}
              r="5"
              fill={getVerdictColor(p.verdict)}
              stroke="var(--color-surface-card)"
              stroke-width="2"
            />
            <title>{`${p.createdAt.slice(0, 10)}: ${formatSpeed(p.forwardBpsNum)}`}</title>
          {/each}
        </svg>
      </div>
    {/if}
  </Card>
</div>

<style>
  .trend-container {
    margin-bottom: var(--nb-space-4, 1rem);
  }

  .trend-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--nb-space-3, 0.75rem);
  }

  .trend-title {
    font-size: var(--nb-font-size-base, 1rem);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .trend-badge {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-secondary);
  }

  .trend-empty {
    font-size: var(--nb-font-size-sm, 0.875rem);
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .chart-wrapper {
    width: 100%;
    overflow-x: auto;
  }

  .trend-svg {
    width: 100%;
    height: auto;
    display: block;
  }

  .axis-label {
    font-size: 10px;
    fill: var(--color-text-tertiary);
    font-family: var(--font-mono, monospace);
  }
</style>
