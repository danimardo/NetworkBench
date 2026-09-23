<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Dialog from "../../lib/components/Dialog.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  let showPurgeConfirm = $state(false);
  let isPurging = $state(false);

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  async function handleConfirmPurge() {
    isPurging = true;
    try {
      await model.purgeHistory();
      showPurgeConfirm = false;
    } finally {
      isPurging = false;
    }
  }
</script>

<div class="space-y-6">
  <!-- Ubicación y tamaño de datos -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div>
        <h2 class="text-base font-semibold">{t("settings.localDataStorage")}</h2>
        <p class="text-sm text-[var(--color-text-secondary)]">
          {t("settings.localDataStorageDesc")}
        </p>
      </div>

      {#if model.dataInfo}
        <div class="space-y-2 rounded bg-[var(--color-surface-sunken)] p-3 text-xs">
          <div class="flex flex-col gap-1">
            <span class="text-[var(--color-text-secondary)]">{t("settings.databaseFile")}:</span>
            <span class="break-all font-mono font-medium">{model.dataInfo.dbPath}</span>
            <span class="text-[var(--color-text-muted)]">
              {formatBytes(model.dataInfo.dbSizeBytes)}
            </span>
          </div>
          <div class="flex flex-col gap-1 pt-2 border-t border-[var(--color-border)]">
            <span class="text-[var(--color-text-secondary)]">{t("settings.settingsFile")}:</span>
            <span class="break-all font-mono font-medium">{model.dataInfo.settingsPath}</span>
            <span class="text-[var(--color-text-muted)]">
              {formatBytes(model.dataInfo.settingsSizeBytes)}
            </span>
          </div>
        </div>
      {/if}

      <div class="pt-2">
        <Button variant="secondary" onclick={() => (showPurgeConfirm = true)}>
          {t("settings.purgeHistoryBtn")}
        </Button>
      </div>
    </div>
  </Card>

  <!-- Diálogo de Confirmación de Purga -->
  {#if showPurgeConfirm}
    <Dialog
      onClose={() => (showPurgeConfirm = false)}
      title={t("settings.purgeConfirmTitle")}
      size="sm"
    >
      <p class="text-sm text-[var(--color-text-secondary)]">
        {t("settings.purgeConfirmWarning")}
      </p>

      {#snippet actions()}
        <Button variant="secondary" onclick={() => (showPurgeConfirm = false)} disabled={isPurging}>
          {t("common.cancel")}
        </Button>
        <Button variant="primary" onclick={handleConfirmPurge} disabled={isPurging}>
          {isPurging ? t("common.deleting") : t("settings.confirmPurge")}
        </Button>
      {/snippet}
    </Dialog>
  {/if}
</div>
