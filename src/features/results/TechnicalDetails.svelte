<script lang="ts">
  import Dialog from "../../lib/components/Dialog.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import { t } from "../../lib/i18n";
  import type { SessionResult } from "../../lib/contracts/session-result";
  import { getDiagnosticReport } from "../../lib/api/diagnostics";

  interface Props {
    open: boolean;
    result: SessionResult;
    onClose: () => void;
  }

  let { open, result, onClose }: Props = $props();

  let copied = $state(false);
  let copyLoading = $state(false);

  async function handleCopy() {
    copyLoading = true;
    try {
      const report = await getDiagnosticReport(result);
      await navigator.clipboard.writeText(report);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 3000);
    } catch {
      // Fallback
      copied = false;
    } finally {
      copyLoading = false;
    }
  }
</script>

{#if open}
  <Dialog
    title={t("verdict.technical_details")}
    description="Parámetros exactos de la sesión, configuración de hardware y diagnóstico consolidado."
    {onClose}
  >
    <div class="technical-details-content" data-testid="technical-details-dialog">
      <div class="disclosure-alert">
        <Icon name="info" size={16} />
        <span>
          Aviso: La información técnica exportable ha sido saneada automáticamente para eliminar
          contraseñas o certificados privados. Puede compartirse para diagnóstico técnico de red.
        </span>
      </div>

      <div class="details-section">
        <h4 class="section-title">Identificación y versiones</h4>
        <dl class="meta-grid">
          <dt>ID de sesión:</dt>
          <dd class="mono">{result.sessionId}</dd>
          <dt>Versión App:</dt>
          <dd>{result.versions.appVersion}</dd>
          <dt>Motor:</dt>
          <dd>NTTTCP v{result.versions.engineVersion}</dd>
          <dt>Hash Umbrales:</dt>
          <dd class="mono">{result.versions.thresholdsHash.slice(0, 16)}...</dd>
        </dl>
      </div>

      <div class="details-section">
        <h4 class="section-title">Direcciones y Motor</h4>
        {#each result.directions as dir (dir.direction)}
          <div class="direction-block">
            <h5 class="dir-heading">Sentido: {dir.direction} ({dir.status})</h5>
            <dl class="meta-grid">
              <dt>Velocidad oficial (Receptor):</dt>
              <dd class="highlight">{dir.officialBps ? `${dir.officialBps} bps` : "N/A"}</dd>
              {#if dir.sender}
                <dt>Bytes enviados:</dt>
                <dd>{dir.sender.totalBytes}</dd>
                <dt>CPU Emisor:</dt>
                <dd>
                  {dir.sender.cpuPercent != null ? `${dir.sender.cpuPercent.toFixed(1)}%` : "N/A"}
                </dd>
              {/if}
              {#if dir.receiver}
                <dt>Bytes recibidos:</dt>
                <dd>{dir.receiver.totalBytes}</dd>
                <dt>CPU Receptor:</dt>
                <dd>
                  {dir.receiver.cpuPercent != null
                    ? `${dir.receiver.cpuPercent.toFixed(1)}%`
                    : "N/A"}
                </dd>
              {/if}
            </dl>
          </div>
        {/each}
      </div>

      <div class="footer-actions">
        <Button
          variant="primary"
          onclick={handleCopy}
          disabled={copyLoading}
          aria-label={t("verdict.copy_report")}
        >
          <Icon name={copied ? "check" : "copy"} size={16} />
          {copied ? t("common.copied") : t("verdict.copy_report")}
        </Button>
        <Button variant="secondary" onclick={onClose}>
          {t("common.close")}
        </Button>
      </div>
    </div>
  </Dialog>
{/if}

<style>
  .technical-details-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-4, 16px);
    max-height: 65vh;
    overflow-y: auto;
    padding-right: var(--spacing-1, 4px);
  }

  .disclosure-alert {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-2, 8px);
    padding: var(--spacing-3, 12px);
    background: var(--bg-hover, rgba(255, 255, 255, 0.04));
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-md, 8px);
    font-size: var(--font-size-xs, 0.75rem);
    color: var(--text-secondary, #94a3b8);
  }

  .details-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2, 8px);
  }

  .section-title {
    font-size: var(--font-size-sm, 0.875rem);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--text-primary, #ffffff);
    margin: 0;
  }

  .direction-block {
    background: var(--bg-surface, rgba(15, 23, 42, 0.6));
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.06));
    border-radius: var(--radius-md, 8px);
    padding: var(--spacing-3, 12px);
  }

  .dir-heading {
    font-size: var(--font-size-xs, 0.75rem);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--accent, #8b5cf6);
    margin: 0 0 var(--spacing-2, 8px) 0;
    text-transform: capitalize;
  }

  .meta-grid {
    display: grid;
    grid-template-columns: 180px 1fr;
    gap: var(--spacing-1, 4px) var(--spacing-2, 8px);
    font-size: var(--font-size-xs, 0.75rem);
    margin: 0;
  }

  dt {
    color: var(--text-muted, #64748b);
  }

  dd {
    color: var(--text-primary, #ffffff);
    margin: 0;
  }

  .mono {
    font-family: monospace;
  }

  .highlight {
    font-weight: var(--font-weight-bold, 700);
    color: var(--color-emerald, #10b981);
  }

  .footer-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-2, 8px);
    padding-top: var(--spacing-3, 12px);
    border-top: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
  }
</style>
