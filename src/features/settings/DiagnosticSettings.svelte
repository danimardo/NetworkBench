<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Segmented from "../../lib/components/Segmented.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  type LogLevel = "warn" | "info" | "debug";

  // «Warn», «Info» y «Debug» son nombres de nivel, no texto de interfaz; solo el
  // «recomendado» se traduce.
  const logLevels = $derived<{ id: LogLevel; label: string }[]>([
    { id: "warn", label: `Warn (${t("settings.recommended")})` },
    { id: "info", label: "Info" },
    { id: "debug", label: "Debug" },
  ]);

  function handleLogLevelChange(level: LogLevel) {
    void model.update({ logLevel: level });
  }
</script>

<div class="space-y-6">
  <!-- Nivel de Registro Temporal -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.logLevel")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.logLevelDesc")}
          </p>
        </div>

        <Segmented
          options={logLevels}
          value={model.prefs?.logLevel}
          onchange={handleLogLevelChange}
          label={t("settings.logLevel")}
        />
      </div>

      {#if model.prefs?.logLevel === "debug"}
        <div class="rounded bg-[var(--surface-field)] p-3 text-xs text-[var(--color-warning)]">
          {t("settings.debugLevelWarning")}
        </div>
      {/if}
    </div>
  </Card>

  <!-- Actualizaciones del Sistema -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
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
        <div class="rounded bg-[var(--surface-field)] p-3 text-xs">
          {#if model.updateStatus.status === "upToDate"}
            <span class="text-[var(--color-success)]">{t("settings.upToDateMessage")}</span>
          {:else if model.updateStatus.status === "updateAvailable"}
            <div class="space-y-1">
              <span class="font-medium text-[var(--color-text-primary)]">
                {t("settings.updateAvailableTitle")}: {model.updateStatus.version}
              </span>
              <p class="text-[var(--color-text-secondary)]">{model.updateStatus.notes}</p>
            </div>
          {:else if model.updateStatus.status === "deferredDueToActiveSession"}
            <span class="text-[var(--color-warning)]">
              {t("settings.updateDeferredMessage")}
            </span>
          {/if}
        </div>
      {/if}
    </div>
  </Card>
</div>
