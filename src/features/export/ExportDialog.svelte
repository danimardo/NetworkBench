<script lang="ts">
  import Dialog from "../../lib/components/Dialog.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import { t, getLocale } from "../../lib/i18n";
  import { previewExport, executeExport } from "../../lib/api/export";
  import type { ExportFormat, ExportPreviewResponse } from "../../lib/contracts/export";

  let {
    open = false,
    sessionIds = [],
    peerName = "",
    onclose,
    onexported,
  }: {
    open: boolean;
    sessionIds: string[];
    peerName?: string;
    onclose: () => void;
    onexported?: (paths: string[]) => void;
  } = $props();

  let selectedFormat = $state<ExportFormat>("pdf");
  let anonymize = $state(false);
  let destinationDir = $state("C:\\Users\\Mock\\Downloads");
  let previewData = $state<ExportPreviewResponse | null>(null);
  let isExporting = $state(false);
  let errorMessage = $state<string | null>(null);
  let successFiles = $state<string[] | null>(null);

  // Cargar previsualización reactivamente cuando cambian formato, ids o anonimización
  $effect(() => {
    if (!open || sessionIds.length === 0) return;

    errorMessage = null;
    previewExport({
      sessionIds,
      format: selectedFormat,
      anonymize,
    })
      .then((res) => {
        previewData = res;
      })
      .catch((err) => {
        errorMessage = err instanceof Error ? err.message : String(err);
      });
  });

  async function handleExecute() {
    if (!previewData || isExporting) return;

    isExporting = true;
    errorMessage = null;

    try {
      const delimiter = getLocale() === "es" ? ";" : ",";
      const decimalSeparator = getLocale() === "es" ? "," : ".";

      const res = await executeExport({
        token: previewData.token,
        sessionIds,
        format: selectedFormat,
        anonymize,
        destinationDir,
        delimiter,
        decimalSeparator,
      });

      successFiles = res.exportedFiles;
      if (onexported) {
        onexported(res.exportedFiles);
      }
    } catch (err) {
      errorMessage = err instanceof Error ? err.message : String(err);
    } finally {
      isExporting = false;
    }
  }

  function handleClose() {
    successFiles = null;
    errorMessage = null;
    onclose();
  }
</script>

{#if open}
  <Dialog
    title={t("export.dialog_title")}
    description={peerName ? peerName : undefined}
    onClose={handleClose}
    size="md"
  >
    <div class="export-dialog-body">
      {#if successFiles}
        <!-- Estado de Éxito -->
        <div class="export-success" role="status">
          <Icon name="check" size={32} />
          <h4>{t("export.success_title")}</h4>
          <p>{t("export.success_message")}</p>
          <ul class="file-list">
            {#each successFiles as f (f)}
              <li class="file-item mono">{f}</li>
            {/each}
          </ul>
        </div>
      {:else}
        <!-- Selector de Formato -->
        <div class="form-group">
          <label class="group-label" for="export-format-selector">{t("export.format_label")}</label>
          <div class="format-segmented" id="export-format-selector" role="radiogroup">
            <button
              type="button"
              role="radio"
              aria-checked={selectedFormat === "pdf"}
              class="seg-btn"
              class:active={selectedFormat === "pdf"}
              onclick={() => (selectedFormat = "pdf")}
            >
              PDF
            </button>
            <button
              type="button"
              role="radio"
              aria-checked={selectedFormat === "json"}
              class="seg-btn"
              class:active={selectedFormat === "json"}
              onclick={() => (selectedFormat = "json")}
            >
              JSON
            </button>
            <button
              type="button"
              role="radio"
              aria-checked={selectedFormat === "csv"}
              class="seg-btn"
              class:active={selectedFormat === "csv"}
              onclick={() => (selectedFormat = "csv")}
            >
              CSV
            </button>
          </div>
        </div>

        <!-- Aviso de Contenido (Disclosure §20.1) -->
        <div class="disclosure-box" role="note">
          <div class="disclosure-header">
            <Icon name="info" size={18} />
            <strong>{t("export.disclosure.header")}</strong>
          </div>
          <p class="disclosure-text">{t("export.disclosure.summary")}</p>
          <ul class="disclosure-items">
            <li>{t("export.disclosure.equipment_names")}</li>
            {#if !anonymize}
              <li>{t("export.disclosure.ip_addresses")}</li>
              <li>{t("export.disclosure.mac_adapters")}</li>
            {:else}
              <li class="anonymized-item">{t("export.disclosure.anonymized_notice")}</li>
            {/if}
            <li>{t("export.disclosure.test_results")}</li>
            <li>{t("export.disclosure.configuration")}</li>
          </ul>
        </div>

        <!-- Casilla de Anonimización -->
        <div class="anon-checkbox-group">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={anonymize} class="nb-checkbox" />
            <span>{t("export.hide_ips_macs")}</span>
          </label>
        </div>

        <!-- Nota de formato CSV según locale -->
        {#if selectedFormat === "csv"}
          <div class="csv-locale-note">
            <small>
              {#if getLocale() === "es"}
                {t("export.csv_format_note_es")}
              {:else}
                {t("export.csv_format_note_en")}
              {/if}
            </small>
          </div>
        {/if}

        <!-- Previsualización de Nombres de Archivo -->
        {#if previewData}
          <div class="files-preview">
            <span class="preview-label">{t("export.files_to_generate")}:</span>
            <ul class="preview-file-list">
              {#each previewData.fileNames as fn (fn)}
                <li class="preview-file-item mono">{fn}</li>
              {/each}
            </ul>
          </div>
        {/if}

        <!-- Mensaje de Error -->
        {#if errorMessage}
          <div class="error-banner" role="alert">
            <Icon name="x-circle" size={18} />
            <span>{errorMessage}</span>
          </div>
        {/if}
      {/if}
    </div>

    {#snippet actions()}
      {#if successFiles}
        <Button variant="primary" onclick={handleClose}>
          {t("common.close")}
        </Button>
      {:else}
        <Button variant="secondary" onclick={handleClose} disabled={isExporting}>
          {t("common.cancel")}
        </Button>
        <Button variant="primary" onclick={handleExecute} disabled={isExporting || !previewData}>
          {isExporting ? t("export.exporting") : t("export.action")}
        </Button>
      {/if}
    {/snippet}
  </Dialog>
{/if}

<style>
  .export-dialog-body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 8px 0;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .group-label {
    font-size: var(--nb-text-sm, 13px);
    font-weight: 500;
    color: var(--nb-text-secondary, #64748b);
  }

  .format-segmented {
    display: flex;
    background: var(--nb-bg-secondary, #f1f5f9);
    padding: 3px;
    border-radius: 6px;
    gap: 2px;
  }

  .seg-btn {
    flex: 1;
    padding: 6px 12px;
    font-size: var(--nb-text-sm, 13px);
    font-weight: 500;
    border: none;
    background: transparent;
    color: var(--nb-text-secondary, #64748b);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .seg-btn.active {
    background: var(--nb-bg-surface, #ffffff);
    color: var(--nb-text-primary, #0f172a);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  }

  .disclosure-box {
    background: var(--nb-bg-secondary, #f8fafc);
    border: 1px solid var(--nb-border-default, #e2e8f0);
    border-radius: 6px;
    padding: 12px;
    font-size: var(--nb-text-sm, 13px);
  }

  .disclosure-header {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--nb-text-primary, #0f172a);
    margin-bottom: 6px;
  }

  .disclosure-text {
    color: var(--nb-text-secondary, #64748b);
    margin: 0 0 8px 0;
  }

  .disclosure-items {
    margin: 0;
    padding-left: 20px;
    color: var(--nb-text-secondary, #64748b);
  }

  .anonymized-item {
    color: var(--nb-color-success, #16a34a);
    font-weight: 500;
  }

  .anon-checkbox-group {
    display: flex;
    align-items: center;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--nb-text-sm, 13px);
    cursor: pointer;
    user-select: none;
  }

  .nb-checkbox {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .csv-locale-note {
    background: var(--nb-bg-subtle, #f1f5f9);
    padding: 8px 12px;
    border-radius: 4px;
    color: var(--nb-text-secondary, #64748b);
  }

  .files-preview {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .preview-label {
    font-size: var(--nb-text-xs, 12px);
    color: var(--nb-text-secondary, #64748b);
  }

  .preview-file-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .preview-file-item {
    background: var(--nb-bg-secondary, #f8fafc);
    border: 1px solid var(--nb-border-default, #e2e8f0);
    padding: 6px 10px;
    border-radius: 4px;
    font-size: var(--nb-text-xs, 12px);
  }

  .mono {
    font-family: monospace;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #fef2f2;
    color: #dc2626;
    border: 1px solid #fecaca;
    border-radius: 6px;
    padding: 8px 12px;
    font-size: var(--nb-text-sm, 13px);
  }

  .export-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 16px 0;
    color: var(--nb-color-success, #16a34a);
  }

  .export-success h4 {
    margin: 8px 0 4px 0;
    font-size: var(--nb-text-lg, 16px);
    color: var(--nb-text-primary, #0f172a);
  }

  .export-success p {
    margin: 0 0 16px 0;
    color: var(--nb-text-secondary, #64748b);
    font-size: var(--nb-text-sm, 13px);
  }

  .file-list {
    margin: 0;
    padding: 0;
    list-style: none;
    width: 100%;
    text-align: left;
  }

  .file-item {
    background: var(--nb-bg-secondary, #f8fafc);
    border: 1px solid var(--nb-border-default, #e2e8f0);
    padding: 6px 10px;
    border-radius: 4px;
    font-size: var(--nb-text-xs, 12px);
    margin-bottom: 4px;
    word-break: break-all;
  }
</style>
