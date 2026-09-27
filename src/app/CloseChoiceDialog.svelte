<script lang="ts">
  import Dialog from "../lib/components/Dialog.svelte";
  import Button from "../lib/components/Button.svelte";
  import { t } from "../lib/i18n";
  import type { EleccionAlCerrar } from "../lib/api/lifecycle";

  interface Props {
    /** Lo elegido y si se quiere recordar. */
    onChoose: (eleccion: EleccionAlCerrar, recordar: boolean) => void;
    /** Escape o clic fuera: no hacer nada (§5.2). */
    onDismiss: () => void;
  }

  let { onChoose, onDismiss }: Props = $props();

  let recordar = $state(false);
</script>

<!-- Diálogo de §5.2: «¿Qué quieres hacer?» al pulsar cerrar con el ajuste en «Preguntar». -->
<Dialog onClose={onDismiss} title={t("app.closeChoice.title")} size="md">
  <div class="space-y-4">
    <p class="text-sm text-[var(--color-text-secondary)]">{t("app.closeChoice.body")}</p>

    <label class="nb-remember">
      <input type="checkbox" bind:checked={recordar} data-testid="close-remember" />
      <span>{t("app.closeChoice.remember")}</span>
    </label>
  </div>

  {#snippet actions()}
    <Button
      variant="secondary"
      onclick={() => onChoose("minimize", recordar)}
      data-testid="close-minimize"
    >
      {t("app.closeChoice.minimize")}
    </Button>
    <Button variant="primary" onclick={() => onChoose("exit", recordar)} data-testid="close-exit">
      {t("app.closeChoice.exit")}
    </Button>
  {/snippet}
</Dialog>

<style>
  .nb-remember {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    cursor: default;
  }

  .nb-remember input {
    width: 16px;
    height: 16px;
    accent-color: var(--color-accent);
  }
</style>
