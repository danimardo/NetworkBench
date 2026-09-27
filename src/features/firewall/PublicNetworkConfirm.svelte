<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import Dialog from "../../lib/components/Dialog.svelte";
  import { t } from "../../lib/i18n";

  interface Props {
    onConfirm: () => void;
    onCancel: () => void;
  }

  let { onConfirm, onCancel }: Props = $props();
</script>

<!--
  Abrir los puertos en redes públicas amplía la exposición: la decisión se explica y se
  confirma, siempre igual, venga de donde venga (el aviso de arranque o Ajustes).
-->
<Dialog onClose={onCancel} title={t("settings.fw.public.confirmTitle")} size="sm">
  <div class="space-y-3 text-sm text-[var(--color-text-secondary)]">
    <p>{t("settings.fw.public.confirmBody")}</p>
    <p>{t("settings.fw.public.confirmUac")}</p>
  </div>

  {#snippet actions()}
    <Button variant="secondary" onclick={onCancel} data-testid="fw-public-confirm-cancel">
      {t("common.cancel")}
    </Button>
    <Button variant="primary" onclick={onConfirm} data-testid="fw-public-confirm-accept">
      {t("settings.fw.public.confirmAllow")}
    </Button>
  {/snippet}
</Dialog>
