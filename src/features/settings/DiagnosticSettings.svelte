<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Button from "../../lib/components/Button.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  const LOG_LEVELS = [
    { id: "warn", label: "Warn (Recomendado)" },
    { id: "info", label: "Info" },
    { id: "debug", label: "Debug" },
  ] as const;

  function handleLogLevelChange(level: "warn" | "info" | "debug") {
    model.update({ logLevel: level });
  }
</script>

<div class="space-y-6">
  <!-- Nivel de Registro Temporal -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold">{t("settings.logLevel")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.logLevelDesc")}
          </p>
        </div>

        <div class="nb-segmented-group" role="radiogroup" aria-label={t("settings.logLevel")}>
          {#each LOG_LEVELS as lvl (lvl.id)}
            <button
              type="button"
              role="radio"
              aria-checked={model.prefs?.logLevel === lvl.id}
              class="nb-segmented-item"
              class:nb-segmented-item-active={model.prefs?.logLevel === lvl.id}
              onclick={() => handleLogLevelChange(lvl.id)}
            >
              {lvl.label}
            </button>
          {/each}
        </div>
      </div>

      {#if model.prefs?.logLevel === "debug"}
        <div
          class="rounded bg-[var(--color-surface-sunken)] p-3 text-xs text-[var(--color-text-warning)]"
        >
          {t("settings.debugLevelWarning")}
        </div>
      {/if}
    </div>
  </Card>

  <!-- Actualizaciones del Sistema -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold">{t("settings.updates")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.updatesDesc")}
          </p>
        </div>

        <Button
          variant="secondary"
          onclick={() => model.checkForUpdates()}
          disabled={model.isCheckingUpdate}
        >
          {model.isCheckingUpdate ? t("common.checking") : t("settings.checkUpdatesBtn")}
        </Button>
      </div>

      {#if model.updateStatus}
        <div class="rounded bg-[var(--color-surface-sunken)] p-3 text-xs">
          {#if model.updateStatus.status === "upToDate"}
            <span class="text-[var(--color-text-success)]">{t("settings.upToDateMessage")}</span>
          {:else if model.updateStatus.status === "updateAvailable"}
            <div class="space-y-1">
              <span class="font-medium text-[var(--color-text-primary)]">
                {t("settings.updateAvailableTitle")}: {model.updateStatus.version}
              </span>
              <p class="text-[var(--color-text-secondary)]">{model.updateStatus.notes}</p>
            </div>
          {:else if model.updateStatus.status === "deferredDueToActiveSession"}
            <span class="text-[var(--color-text-warning)]">
              {t("settings.updateDeferredMessage")}
            </span>
          {/if}
        </div>
      {/if}
    </div>
  </Card>
</div>
