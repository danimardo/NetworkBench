<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import { t } from "../../lib/i18n";
  import { repeatPlan } from "../../lib/api/history";
  import type { RepeatPlanConfig } from "../../lib/contracts/history";

  interface Props {
    sessionId: string;
    onRepeat?: (config: RepeatPlanConfig) => void;
    onDelete?: () => void;
    onExport?: () => void;
  }

  let { sessionId, onRepeat, onDelete, onExport }: Props = $props();

  let isRepeating = $state(false);
  let repeatError = $state<string | null>(null);

  async function handleRepeat() {
    isRepeating = true;
    repeatError = null;
    try {
      const config = await repeatPlan(sessionId);
      onRepeat?.(config);
    } catch {
      repeatError = "No se pudieron reconstruir los parámetros válidos de la prueba";
    } finally {
      isRepeating = false;
    }
  }
</script>

<div class="history-actions" role="toolbar" aria-label="Acciones de la sesión">
  <Button variant="primary" onclick={handleRepeat} disabled={isRepeating}>
    {isRepeating ? "Reconstruyendo..." : t("history.repeat_test")}
  </Button>

  {#if onExport}
    <Button variant="secondary" onclick={onExport}>
      {t("history.export")}
    </Button>
  {/if}

  {#if onDelete}
    <Button variant="ghost" onclick={onDelete}>
      {t("history.delete")}
    </Button>
  {/if}

  {#if repeatError}
    <span class="repeat-error" role="alert">{repeatError}</span>
  {/if}
</div>

<style>
  .history-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--nb-space-3, 0.75rem);
    margin-top: var(--nb-space-4, 1rem);
  }

  .repeat-error {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-danger);
    width: 100%;
  }
</style>
