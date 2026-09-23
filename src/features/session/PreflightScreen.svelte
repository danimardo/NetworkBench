<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import Card from "../../lib/components/Card.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import { t } from "../../lib/i18n";
  import type { AppError } from "../../lib/contracts/errors";

  export type CheckType =
    "engine" | "networkInterface" | "ports" | "firewall" | "diskSpace" | "version";

  export type CheckStatus = "pending" | "running" | "passed" | "warning" | "failed";

  export interface PreflightItem {
    id: CheckType;
    titleKey: string;
    status: CheckStatus;
    error?: AppError;
  }

  interface Props {
    checks: PreflightItem[];
    onRetry?: () => void;
    onProceed?: () => void;
    onCancel?: () => void;
    onErrorClick?: (error: AppError) => void;
  }

  let { checks, onRetry, onProceed, onCancel, onErrorClick }: Props = $props();

  let hasFailed = $derived.by(() => checks.some((c) => c.status === "failed"));
  let allPassedOrWarn = $derived.by(
    () => checks.length > 0 && checks.every((c) => c.status === "passed" || c.status === "warning"),
  );

  function getStatusTone(
    status: CheckStatus,
  ): "neutral" | "info" | "success" | "warning" | "danger" {
    switch (status) {
      case "passed":
        return "success";
      case "warning":
        return "warning";
      case "failed":
        return "danger";
      case "running":
        return "info";
      case "pending":
      default:
        return "neutral";
    }
  }

  function getStatusIcon(status: CheckStatus): "check" | "alert-triangle" | "clock" {
    switch (status) {
      case "passed":
        return "check";
      case "warning":
      case "failed":
        return "alert-triangle";
      case "running":
      case "pending":
      default:
        return "clock";
    }
  }
</script>

<div
  class="preflight-screen-container"
  role="region"
  aria-label="Comprobaciones previas (Preflight)"
>
  <Card>
    <div class="header">
      <h2 class="title">Verificaciones previas a la medición</h2>
      <p class="subtitle">
        Comprobando que el motor, los puertos, el cortafuegos y el adaptador están listos antes de
        iniciar el benchmark.
      </p>
    </div>

    <ul class="checks-list" role="list">
      {#each checks as check (check.id)}
        <li class="check-item" class:failed={check.status === "failed"}>
          <div class="check-info">
            <span
              class="check-icon"
              class:success={check.status === "passed"}
              class:danger={check.status === "failed"}
            >
              <Icon name={getStatusIcon(check.status)} size={18} />
            </span>
            <span class="check-title">{t(check.titleKey)}</span>
          </div>
          <div class="check-status">
            <StatusPill tone={getStatusTone(check.status)}>
              {t(`preflight.status_${check.status}`)}
            </StatusPill>
            {#if check.error && onErrorClick}
              <Button variant="ghost" onclick={() => onErrorClick?.(check.error!)}>
                Ver detalle
              </Button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>

    <div class="footer-actions">
      {#if hasFailed && onRetry}
        <Button variant="primary" onclick={onRetry}>Reintentar verificaciones</Button>
      {/if}

      {#if allPassedOrWarn && onProceed}
        <Button variant="primary" onclick={onProceed}>Continuar con la prueba</Button>
      {/if}

      {#if onCancel}
        <Button variant="ghost" onclick={onCancel}>Cancelar</Button>
      {/if}
    </div>
  </Card>
</div>

<style>
  .preflight-screen-container {
    max-width: 680px;
    margin: var(--nb-space-6) auto;
  }

  .header {
    margin-bottom: var(--nb-space-4);
  }

  .title {
    font-size: var(--nb-font-size-lg, 1.125rem);
    font-weight: 600;
    color: var(--nb-color-text-primary, #ffffff);
    margin: 0 0 var(--nb-space-1) 0;
  }

  .subtitle {
    font-size: var(--nb-font-size-sm, 0.8125rem);
    color: var(--nb-color-text-secondary, #9ca3af);
    margin: 0;
    line-height: 1.4;
  }

  .checks-list {
    list-style: none;
    padding: 0;
    margin: var(--nb-space-4) 0;
    border-top: 1px solid var(--nb-color-border-subtle, #374151);
  }

  .check-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--nb-space-3) 0;
    border-bottom: 1px solid var(--nb-color-border-subtle, #374151);
  }

  .check-info {
    display: flex;
    align-items: center;
    gap: var(--nb-space-3);
  }

  .check-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--nb-color-text-muted, #9ca3af);
  }

  .check-icon.success {
    color: var(--nb-color-success, #10b981);
  }

  .check-icon.danger {
    color: var(--nb-color-danger, #ef4444);
  }

  .check-title {
    font-size: var(--nb-font-size-base, 0.875rem);
    color: var(--nb-color-text-primary, #ffffff);
  }

  .check-status {
    display: flex;
    align-items: center;
    gap: var(--nb-space-2);
  }

  .footer-actions {
    display: flex;
    gap: var(--nb-space-2);
    margin-top: var(--nb-space-4);
    justify-content: flex-end;
  }
</style>
