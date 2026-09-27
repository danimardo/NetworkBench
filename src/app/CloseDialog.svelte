<script lang="ts">
  import Dialog from "../lib/components/Dialog.svelte";
  import Button from "../lib/components/Button.svelte";
  import { t } from "../lib/i18n";

  interface Props {
    open: boolean;
    oncancel: () => void;
    onconfirm: () => void;
  }

  let { open, oncancel, onconfirm }: Props = $props();
</script>

{#if open}
  <Dialog onClose={oncancel} title={t("app.closeConfirmTitle")} size="sm">
    <div class="space-y-4">
      <div
        class="rounded-md bg-[var(--color-warning-soft)] p-3 text-xs text-[var(--color-warning)]"
      >
        {t("app.closeConfirmWarning")}
      </div>

      <p class="text-sm text-[var(--color-text-secondary)]">
        {t("app.closeConfirmDescription")}
      </p>
    </div>

    {#snippet actions()}
      <Button variant="secondary" onclick={oncancel}>
        {t("app.closeKeepRunning")}
      </Button>
      <Button variant="primary" onclick={onconfirm}>
        {t("app.closeConfirmExit")}
      </Button>
    {/snippet}
  </Dialog>
{/if}
