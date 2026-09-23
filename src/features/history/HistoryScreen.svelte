<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import Card from "../../lib/components/Card.svelte";
  import Dialog from "../../lib/components/Dialog.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import TextField from "../../lib/components/TextField.svelte";
  import HistoryDetail from "./HistoryDetail.svelte";
  import HistoryTrend from "./HistoryTrend.svelte";
  import { HistoryModel } from "./model.svelte";
  import { t } from "../../lib/i18n";
  import type { HistoryItemSummary, RepeatPlanConfig } from "../../lib/contracts/history";
  import { ExportDialog } from "../export";

  interface Props {
    onRepeatTest?: (config: RepeatPlanConfig) => void;
  }

  let { onRepeatTest }: Props = $props();

  const model = new HistoryModel();

  let isExportOpen = $state(false);
  let exportSessionIds = $state<string[]>([]);
  let exportPeerName = $state("");

  function openExport(ids: string[], peerName?: string) {
    exportSessionIds = ids;
    exportPeerName = peerName ?? "";
    isExportOpen = true;
  }

  $effect(() => {
    model.load();
  });

  function formatRelativeDate(isoStr: string): string {
    const d = new Date(isoStr);
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const targetDate = new Date(d.getFullYear(), d.getMonth(), d.getDate());
    const diffDays = Math.round((today.getTime() - targetDate.getTime()) / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return t("history.today");
    if (diffDays === 1) return t("history.yesterday");

    return d.toLocaleDateString(undefined, { day: "numeric", month: "short" });
  }

  function formatSpeed(bpsStr: string | null | undefined): string {
    if (!bpsStr) return "--";
    const n = Number(bpsStr);
    if (isNaN(n)) return "--";
    if (n >= 1_000_000_000) {
      return `${(n / 1_000_000_000).toFixed(2).replace(".", ",")} Gbit/s`;
    }
    return `${(n / 1_000_000).toFixed(1).replace(".", ",")} Mbit/s`;
  }

  let groupedItems = $derived.by(() => {
    const groups: Record<string, HistoryItemSummary[]> = {};
    for (const item of model.items) {
      const rel = formatRelativeDate(item.createdAt);
      if (!groups[rel]) {
        groups[rel] = [];
      }
      groups[rel].push(item);
    }
    return Object.entries(groups);
  });

  function getVerdictTone(
    verdict: string | null | undefined,
  ): "success" | "warning" | "danger" | "neutral" {
    if (!verdict) return "neutral";
    if (verdict.includes("ok")) return "success";
    if (verdict.includes("warn")) return "warning";
    if (verdict.includes("problem")) return "danger";
    return "neutral";
  }

  function getVerdictLabel(verdict: string | null | undefined): string {
    if (!verdict) return "Sin datos";
    if (verdict.includes("ok")) return "Óptimo";
    if (verdict.includes("warn")) return "Aviso";
    if (verdict.includes("problem")) return "Problema";
    return "No evaluable";
  }
</script>

<div class="history-screen" role="region" aria-label={t("history.title")}>
  {#if model.selectedSession}
    <HistoryDetail
      session={model.selectedSession}
      cohortComparison={model.cohortComparison}
      onBack={() => model.closeDetail()}
      onRepeat={onRepeatTest}
      onExport={() =>
        openExport([model.selectedSessionId!], model.selectedSession?.peerId ?? undefined)}
      onDelete={() => model.promptDelete({ type: "single", value: model.selectedSessionId! })}
    />
  {:else}
    <div class="history-main">
      <div class="header-section">
        <h1 class="page-title">{t("history.title")}</h1>
        {#if model.items.length > 0}
          <div class="header-buttons">
            <Button variant="secondary" onclick={() => openExport(model.items.map((i) => i.id))}>
              {t("history.export")}
            </Button>
            <Button variant="ghost" onclick={() => model.promptDelete({ type: "all" })}>
              {t("history.delete_all_btn")}
            </Button>
          </div>
        {/if}
      </div>

      <!-- Barra de búsqueda y filtros -->
      <div class="filters-bar">
        <div class="search-box">
          <TextField
            label={t("history.search_placeholder")}
            placeholder={t("history.search_placeholder")}
            value={model.searchQuery}
            oninput={(val) => {
              model.searchQuery = val;
              model.load();
            }}
          />
        </div>

        <div class="select-filters">
          <select
            class="filter-select"
            aria-label={t("history.filter_all_verdicts")}
            value={model.selectedVerdict ?? ""}
            onchange={(e) => {
              const val = (e.target as HTMLSelectElement).value;
              model.selectedVerdict = val ? val : null;
              model.load();
            }}
          >
            <option value="">{t("history.filter_all_verdicts")}</option>
            <option value="ok">Óptimo</option>
            <option value="warn">Aviso</option>
            <option value="problem">Problema</option>
          </select>

          <select
            class="filter-select"
            aria-label={t("history.filter_all_protocols")}
            value={model.selectedProtocol ?? ""}
            onchange={(e) => {
              const val = (e.target as HTMLSelectElement).value;
              model.selectedProtocol = val ? val : null;
              model.load();
            }}
          >
            <option value="">{t("history.filter_all_protocols")}</option>
            <option value="tcp">TCP</option>
            <option value="udp">UDP</option>
          </select>
        </div>
      </div>

      <!-- Evolución si hay peer seleccionado -->
      {#if model.selectedPeerId && model.peerTrend.length > 0}
        <HistoryTrend points={model.peerTrend} peerName={model.items[0]?.peerName ?? "Equipo"} />
      {/if}

      <!-- Listado agrupado -->
      {#if model.items.length === 0}
        <Card>
          <div class="empty-state">
            <p class="empty-title">
              {model.searchQuery.trim() ? t("history.empty_search") : t("history.empty")}
            </p>
          </div>
        </Card>
      {:else}
        <div class="groups-container">
          {#each groupedItems as [groupDate, items] (groupDate)}
            <div class="date-group">
              <h3 class="group-heading">{groupDate}</h3>
              <ul class="session-list" role="list">
                {#each items as item (item.id)}
                  <li class="session-item">
                    <button class="session-card-btn" onclick={() => model.selectSession(item.id)}>
                      <div class="item-primary">
                        <span class="peer-name">{item.peerName}</span>
                        <div class="speed-row">
                          <span class="speeds">
                            {formatSpeed(item.forwardBps)} / {formatSpeed(item.reverseBps)}
                          </span>
                          {#if item.protocol.toLowerCase() !== "tcp"}
                            <span class="protocol-badge">{item.protocol.toUpperCase()}</span>
                          {/if}
                          {#if item.isPartial}
                            <span class="partial-badge">{t("history.incomplete_tag")}</span>
                          {/if}
                        </div>
                      </div>

                      <div class="item-secondary">
                        <StatusPill tone={getVerdictTone(item.diagnosticVerdict)}>
                          {getVerdictLabel(item.diagnosticVerdict)}
                        </StatusPill>
                        <span class="time-label">{item.createdAt.slice(11, 16)}</span>
                      </div>
                    </button>
                  </li>
                {/each}
              </ul>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Diálogo de confirmación de borrado -->
  {#if model.deleteModal.isOpen}
    <Dialog
      title={t("history.delete_confirm_title")}
      description={t("history.delete_confirm_desc")}
      onClose={() => model.cancelDelete()}
    >
      <div class="dialog-content">
        {#if model.deleteModal.preview}
          <div class="preview-stats">
            <span
              >Sesiones afectadas: <strong>{model.deleteModal.preview.affectedSessions}</strong
              ></span
            >
            <span
              >Muestras asociadas: <strong>{model.deleteModal.preview.affectedSamples}</strong
              ></span
            >
          </div>
        {/if}
        {#if model.deleteModal.error}
          <p class="error-msg" role="alert">{model.deleteModal.error}</p>
        {/if}
      </div>

      {#snippet actions()}
        <Button variant="ghost" onclick={() => model.cancelDelete()}>
          {t("common.cancel")}
        </Button>
        <Button
          variant="primary"
          onclick={() => model.executeDelete()}
          disabled={model.deleteModal.isDeleting || !model.deleteModal.preview}
        >
          {model.deleteModal.isDeleting ? "Eliminando..." : t("history.delete_btn")}
        </Button>
      {/snippet}
    </Dialog>
  {/if}

  <ExportDialog
    open={isExportOpen}
    sessionIds={exportSessionIds}
    peerName={exportPeerName}
    onclose={() => (isExportOpen = false)}
  />
</div>

<style>
  .history-screen {
    max-width: 900px;
    margin: 0 auto;
    padding: var(--nb-space-4, 1rem) 0;
  }

  .header-section {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--nb-space-4, 1rem);
  }

  .header-buttons {
    display: flex;
    align-items: center;
    gap: var(--nb-space-2, 0.5rem);
  }

  .page-title {
    font-size: var(--nb-font-size-xl, 1.25rem);
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .filters-bar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--nb-space-3, 0.75rem);
    margin-bottom: var(--nb-space-4, 1rem);
  }

  .search-box {
    flex: 1;
    min-width: 220px;
  }

  .select-filters {
    display: flex;
    gap: var(--nb-space-2, 0.5rem);
  }

  .filter-select {
    padding: var(--nb-space-2, 0.5rem) var(--nb-space-3, 0.75rem);
    background: var(--color-surface-sunken, rgba(0, 0, 0, 0.2));
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm, 6px);
    color: var(--color-text-primary);
    font-size: var(--nb-font-size-sm, 0.875rem);
  }

  .groups-container {
    display: flex;
    flex-direction: column;
    gap: var(--nb-space-4, 1rem);
  }

  .date-group {
    display: flex;
    flex-direction: column;
    gap: var(--nb-space-2, 0.5rem);
  }

  .group-heading {
    font-size: var(--nb-font-size-sm, 0.875rem);
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--nb-space-1, 0.25rem);
  }

  .session-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--nb-space-2, 0.5rem);
  }

  .session-card-btn {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--nb-space-3, 0.75rem) var(--nb-space-4, 1rem);
    background: var(--color-surface-card, rgba(255, 255, 255, 0.03));
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-card, 8px);
    text-align: left;
    cursor: pointer;
    transition:
      background 0.15s ease,
      border-color 0.15s ease;
  }

  .session-card-btn:hover {
    background: var(--color-surface-sunken, rgba(255, 255, 255, 0.06));
    border-color: var(--color-accent);
  }

  .item-primary {
    display: flex;
    flex-direction: column;
    gap: var(--nb-space-1, 0.25rem);
  }

  .peer-name {
    font-size: var(--nb-font-size-base, 1rem);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .speed-row {
    display: flex;
    align-items: center;
    gap: var(--nb-space-2, 0.5rem);
    font-size: var(--nb-font-size-sm, 0.875rem);
    color: var(--color-text-secondary);
  }

  .speeds {
    font-family: var(--font-mono, monospace);
  }

  .protocol-badge,
  .partial-badge {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 4px;
    font-weight: 600;
  }

  .protocol-badge {
    background: var(--color-surface-sunken);
    color: var(--color-accent);
    border: 1px solid var(--color-border-subtle);
  }

  .partial-badge {
    background: rgba(234, 179, 8, 0.15);
    color: var(--color-warning);
    border: 1px solid rgba(234, 179, 8, 0.3);
  }

  .item-secondary {
    display: flex;
    align-items: center;
    gap: var(--nb-space-3, 0.75rem);
  }

  .time-label {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-tertiary);
    font-family: var(--font-mono, monospace);
  }

  .empty-state {
    padding: var(--nb-space-6, 1.5rem);
    text-align: center;
  }

  .empty-title {
    color: var(--color-text-tertiary);
    font-size: var(--nb-font-size-sm, 0.875rem);
  }

  .dialog-content {
    display: flex;
    flex-direction: column;
    gap: var(--nb-space-3, 0.75rem);
    font-size: var(--nb-font-size-sm, 0.875rem);
  }

  .preview-stats {
    display: flex;
    flex-direction: column;
    gap: var(--nb-space-1, 0.25rem);
    padding: var(--nb-space-3, 0.75rem);
    background: var(--color-surface-sunken);
    border-radius: var(--radius-sm, 6px);
  }

  .error-msg {
    color: var(--color-danger);
    font-size: var(--nb-font-size-xs, 0.75rem);
  }
</style>
