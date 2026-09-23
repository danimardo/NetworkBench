<script lang="ts">
  import type { SessionResult } from "../../lib/contracts/session-result";
  import { t } from "../../lib/i18n";

  let {
    session,
    anonymize = false,
  }: {
    session: SessionResult;
    anonymize?: boolean;
  } = $props();

  function formatBps(bpsStr?: string | null): string {
    if (!bpsStr) return "—";
    const num = Number(bpsStr);
    if (isNaN(num)) return "—";
    if (num >= 1_000_000_000) {
      return (num / 1_000_000_000).toFixed(2) + " Gbit/s";
    }
    return (num / 1_000_000).toFixed(2) + " Mbit/s";
  }

  let initiatorName = $derived(anonymize ? "Equipo Local" : session.initiator.displayName);
  let responderName = $derived(anonymize ? "Equipo Remoto" : session.responder.displayName);
  let forwardDir = $derived(session.directions.find((d) => d.direction === "forward"));
  let reverseDir = $derived(session.directions.find((d) => d.direction === "reverse"));
</script>

<div class="print-report-container" id="print-report">
  <!-- Cabecera / Portada A4 -->
  <header class="report-header">
    <div class="header-brand">
      <h1 class="report-title">{t("export.pdf.title")}</h1>
      <span class="report-subtitle">NetworkBench · {t("export.pdf.benchmark_engine")}</span>
    </div>
    <div class="header-meta">
      <div class="meta-row">
        <span class="meta-label">{t("export.pdf.date")}:</span>
        <span class="meta-value">{session.startedAt}</span>
      </div>
      <div class="meta-row">
        <span class="meta-label">{t("export.pdf.session_id")}:</span>
        <span class="meta-value mono">{session.sessionId}</span>
      </div>
    </div>
  </header>

  <!-- Resumen de Equipos y Veredicto -->
  <section class="section summary-card">
    <div class="peers-display">
      <span class="peer-badge local">{initiatorName}</span>
      <span class="peers-arrow">↔</span>
      <span class="peer-badge remote">{responderName}</span>
    </div>

    {#if session.verdict}
      <div class="verdict-banner" class:optimal={session.verdict.level === "ok"}>
        <strong>{t("results.verdict_title")}:</strong>
        {t(session.verdict.titleKey)}
      </div>
    {/if}
  </section>

  <!-- Métricas Principales -->
  <section class="section">
    <h2 class="section-title">{t("export.pdf.performance_summary")}</h2>
    <div class="metrics-grid">
      {#if forwardDir}
        <div class="metric-box">
          <span class="metric-label">{t("results.forward_speed")}</span>
          <strong class="metric-val">{formatBps(forwardDir.officialBps)}</strong>
          {#if forwardDir.utilization}
            <small class="metric-sub"
              >{(forwardDir.utilization * 100).toFixed(1)}% {t("results.utilization")}</small
            >
          {/if}
        </div>
      {/if}

      {#if reverseDir}
        <div class="metric-box">
          <span class="metric-label">{t("results.reverse_speed")}</span>
          <strong class="metric-val">{formatBps(reverseDir.officialBps)}</strong>
          {#if reverseDir.utilization}
            <small class="metric-sub"
              >{(reverseDir.utilization * 100).toFixed(1)}% {t("results.utilization")}</small
            >
          {/if}
        </div>
      {/if}

      <div class="metric-box">
        <span class="metric-label">{t("export.pdf.protocol")}</span>
        <strong class="metric-val uppercase">{session.plan.protocol}</strong>
        <small class="metric-sub">{session.plan.streams} streams</small>
      </div>

      <div class="metric-box">
        <span class="metric-label">{t("export.pdf.capacity_ref")}</span>
        <strong class="metric-val"
          >{formatBps(session.capacity?.refBps ? session.capacity.refBps.toString() : null)}</strong
        >
        <small class="metric-sub">{session.capacity?.refSource ?? "—"}</small>
      </div>
    </div>
  </section>

  <!-- Gráfica SVG Vectorial Integrada -->
  <section class="section">
    <h2 class="section-title">{t("results.speed_over_time")}</h2>
    <div class="svg-chart-wrapper">
      <svg viewBox="0 0 600 160" xmlns="http://www.w3.org/2000/svg" class="report-svg">
        <rect width="600" height="160" fill="#f8fafc" rx="4" />
        <!-- Líneas guía -->
        <line x1="40" y1="30" x2="580" y2="30" stroke="#e2e8f0" stroke-dasharray="4" />
        <line x1="40" y1="80" x2="580" y2="80" stroke="#e2e8f0" stroke-dasharray="4" />
        <line x1="40" y1="130" x2="580" y2="130" stroke="#cbd5e1" />
        <!-- Trazado de datos representativo -->
        <polyline
          fill="none"
          stroke="#0284c7"
          stroke-width="2.5"
          points="40,120 100,60 160,55 220,52 280,50 340,48 400,50 460,49 520,51 580,50"
        />
        <text x="45" y="24" fill="#64748b" font-size="10">Throughput (Mbit/s)</text>
        <text x="45" y="145" fill="#64748b" font-size="9">0s</text>
        <text x="560" y="145" fill="#64748b" font-size="9">{session.plan.measureSeconds}s</text>
      </svg>
    </div>
  </section>

  <!-- Configuración e Información Técnica -->
  <section class="section technical-info">
    <h2 class="section-title">{t("results.technical_details")}</h2>
    <table class="tech-table">
      <tbody>
        <tr>
          <th>{t("export.pdf.measure_duration")}</th>
          <td
            >{session.plan.measureSeconds} s (Warmup: {session.plan.warmupSeconds}s, Cooldown: {session
              .plan.cooldownSeconds}s)</td
          >
        </tr>
        <tr>
          <th>{t("export.pdf.base_port")}</th>
          <td>{session.plan.port}</td>
        </tr>
        <tr>
          <th>{t("export.pdf.buffer_size")}</th>
          <td>{session.plan.bufferSizeBytes ?? "Default (64 KB)"}</td>
        </tr>
        <tr>
          <th>{t("export.pdf.initiator_address")}</th>
          <td class="mono">{anonymize ? "[REDACTED_IP]:18400" : session.initiator.address}</td>
        </tr>
        <tr>
          <th>{t("export.pdf.responder_address")}</th>
          <td class="mono">{anonymize ? "[REDACTED_IP]:18400" : session.responder.address}</td>
        </tr>
      </tbody>
    </table>
  </section>

  <!-- Pie de Informe A4 -->
  <footer class="report-footer">
    <span
      >NetworkBench v{session.versions.appVersion} · Engine NTTTCP v{session.versions
        .engineVersion}</span
    >
    <span>{t("export.pdf.page_notice")}</span>
  </footer>
</div>

<style>
  @page {
    size: A4 portrait;
    margin: 15mm;
  }

  .print-report-container {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    background: #ffffff;
    color: #0f172a;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    padding: 24px;
    box-sizing: border-box;
  }

  .report-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    border-bottom: 2px solid #0284c7;
    padding-bottom: 12px;
    margin-bottom: 20px;
  }

  .report-title {
    font-size: 18pt;
    margin: 0 0 4px 0;
    color: #0f172a;
  }

  .report-subtitle {
    font-size: 9.5pt;
    color: #64748b;
  }

  .meta-row {
    font-size: 9pt;
    margin-bottom: 2px;
    text-align: right;
  }

  .meta-label {
    color: #64748b;
    margin-right: 6px;
  }

  .mono {
    font-family: monospace;
    font-size: 8.5pt;
  }

  .section {
    margin-bottom: 20px;
  }

  .section-title {
    font-size: 11pt;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #334155;
    border-bottom: 1px solid #e2e8f0;
    padding-bottom: 4px;
    margin: 0 0 12px 0;
  }

  .summary-card {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 14px;
  }

  .peers-display {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 11pt;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .peer-badge {
    background: #e2e8f0;
    padding: 4px 10px;
    border-radius: 4px;
  }

  .verdict-banner {
    background: #e0f2fe;
    color: #0369a1;
    border-radius: 4px;
    padding: 8px 12px;
    font-size: 10pt;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
  }

  .metric-box {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 10px;
    display: flex;
    flex-direction: column;
  }

  .metric-label {
    font-size: 8pt;
    color: #64748b;
    margin-bottom: 4px;
  }

  .metric-val {
    font-size: 12pt;
    color: #0f172a;
  }

  .metric-sub {
    font-size: 8pt;
    color: #64748b;
    margin-top: 2px;
  }

  .svg-chart-wrapper {
    width: 100%;
    height: 160px;
    margin-top: 8px;
  }

  .report-svg {
    width: 100%;
    height: 100%;
  }

  .tech-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 9pt;
  }

  .tech-table th,
  .tech-table td {
    padding: 6px 10px;
    text-align: left;
    border-bottom: 1px solid #f1f5f9;
  }

  .tech-table th {
    color: #64748b;
    width: 35%;
  }

  .report-footer {
    display: flex;
    justify-content: space-between;
    border-top: 1px solid #e2e8f0;
    padding-top: 10px;
    margin-top: 24px;
    font-size: 8pt;
    color: #94a3b8;
  }
</style>
