<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import Card from "../../lib/components/Card.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import { t } from "../../lib/i18n";
  import type { AppError, ErrorAction } from "../../lib/contracts/errors";

  interface Props {
    error: AppError;
    onAction?: (action: ErrorAction) => void;
    onCancel?: () => void;
  }

  let { error, onAction, onCancel }: Props = $props();

  let showManualInstructions = $state(false);
  let copyFeedback = $state(false);

  let errorTitle = $derived.by(() => {
    return t(error.messageKey);
  });

  let errorDesc = $derived.by(() => {
    const desc = t(`${error.messageKey}-desc`);
    return desc !== `${error.messageKey}-desc` ? desc : t(error.messageKey);
  });

  let manualCommands = $derived.by(() => {
    return [
      `# Reglas de Firewall de Windows para NetworkBench (PowerShell como Administrador)`,
      `New-NetFirewallRule -DisplayName "NetworkBench - Control" -Direction Inbound -Protocol TCP -LocalPort 5201 -Action Allow -Profile Domain,Private`,
      `New-NetFirewallRule -DisplayName "NetworkBench - NTTTCP TCP" -Direction Inbound -Protocol TCP -LocalPort 5001-5064 -Action Allow -Profile Domain,Private`,
      `New-NetFirewallRule -DisplayName "NetworkBench - NTTTCP UDP" -Direction Inbound -Protocol UDP -LocalPort 5001-5064 -Action Allow -Profile Domain,Private`,
    ].join("\n");
  });

  async function handleCopyInstructions() {
    try {
      await navigator.clipboard.writeText(manualCommands);
      copyFeedback = true;
      setTimeout(() => {
        copyFeedback = false;
      }, 2000);
    } catch {
      // Fallback si no hay clipboard
    }
  }

  function handleActionClick(action: ErrorAction) {
    if (action === "show_firewall_instructions") {
      showManualInstructions = !showManualInstructions;
    } else {
      onAction?.(action);
    }
  }

  function getActionLabel(action: ErrorAction): string {
    return t(`actions.${action}`);
  }

  function getSeverityTone(severity: string): "info" | "warning" | "danger" {
    switch (severity) {
      case "info":
        return "info";
      case "warning":
        return "warning";
      case "fatal":
      case "error":
      default:
        return "danger";
    }
  }
</script>

<div class="error-resolution-container" role="region" aria-label={errorTitle}>
  <Card>
    <div class="header-row">
      <div class="icon-title">
        <div class="error-icon" class:warning={error.severity === "warning"}>
          <Icon name="alert-triangle" size={28} />
        </div>
        <div>
          <h2 class="error-title">{errorTitle}</h2>
          <span class="error-ref" aria-label={`Código de referencia: ${error.code}`}>
            Ref. {error.code}
          </span>
        </div>
      </div>
      <StatusPill tone={getSeverityTone(error.severity)}>
        {error.severity.toUpperCase()}
      </StatusPill>
    </div>

    <p class="error-desc">{errorDesc}</p>

    {#if showManualInstructions}
      <div class="manual-instructions-box" role="region" aria-label="Instrucciones manuales">
        <div class="instructions-header">
          <strong>Instrucciones para PowerShell (como Administrador)</strong>
          <Button variant="ghost" onclick={handleCopyInstructions}>
            {copyFeedback ? "¡Copiado!" : "Copiar comandos"}
          </Button>
        </div>
        <pre class="commands-block"><code>{manualCommands}</code></pre>
      </div>
    {/if}

    <div class="actions-toolbar" role="toolbar" aria-label="Acciones de resolución">
      {#each error.actions as action (action)}
        <Button
          variant={action === "retry" || action === "configure_firewall" ? "primary" : "secondary"}
          onclick={() => handleActionClick(action)}
        >
          {getActionLabel(action)}
        </Button>
      {/each}

      {#if onCancel}
        <Button variant="ghost" onclick={onCancel}>Cancelar</Button>
      {/if}
    </div>
  </Card>
</div>

<style>
  .error-resolution-container {
    max-width: 720px;
    margin: var(--nb-space-6) auto;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--nb-space-4);
  }

  .icon-title {
    display: flex;
    align-items: center;
    gap: var(--nb-space-3);
  }

  .error-icon {
    color: var(--nb-color-danger, #ef4444);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .error-icon.warning {
    color: var(--nb-color-warning, #f59e0b);
  }

  .error-title {
    font-size: var(--nb-font-size-lg, 1.125rem);
    font-weight: 600;
    color: var(--nb-color-text-primary, #ffffff);
    margin: 0;
  }

  .error-ref {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--nb-color-text-muted, #9ca3af);
  }

  .error-desc {
    font-size: var(--nb-font-size-base, 0.875rem);
    color: var(--nb-color-text-secondary, #d1d5db);
    line-height: 1.5;
    margin-bottom: var(--nb-space-4);
  }

  .manual-instructions-box {
    background-color: var(--nb-color-surface-sunken, #111827);
    border: 1px solid var(--nb-color-border-subtle, #374151);
    border-radius: var(--nb-radius-md, 6px);
    padding: var(--nb-space-3);
    margin-bottom: var(--nb-space-4);
  }

  .instructions-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--nb-space-2);
    font-size: var(--nb-font-size-sm, 0.8125rem);
    color: var(--nb-color-text-primary, #ffffff);
  }

  .commands-block {
    background: transparent;
    color: var(--nb-color-text-code, #10b981);
    font-family: var(--nb-font-mono, monospace);
    font-size: var(--nb-font-size-xs, 0.75rem);
    overflow-x: auto;
    padding: var(--nb-space-2);
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .actions-toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--nb-space-2);
    margin-top: var(--nb-space-4);
    padding-top: var(--nb-space-3);
    border-top: 1px solid var(--nb-color-border-subtle, #374151);
  }
</style>
