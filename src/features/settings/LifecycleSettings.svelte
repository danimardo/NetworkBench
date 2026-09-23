<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  function handleTrayToggle() {
    if (!model.prefs) return;
    model.update({ minimizeToTray: !model.prefs.minimizeToTray });
  }
</script>

<div class="space-y-6">
  <!-- Autoarranque con el sistema -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.autostart")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.autostartDesc")}
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={model.autostart}
          class="nb-switch"
          class:nb-switch-checked={model.autostart}
          onclick={() => model.toggleAutostart()}
          aria-label={t("settings.autostart")}
        >
          <span class="nb-switch-thumb"></span>
        </button>
      </div>
    </div>
  </Card>

  <!-- Minimizar a la bandeja del sistema -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.minimizeToTray")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.minimizeToTrayDesc")}
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={model.prefs?.minimizeToTray ?? false}
          class="nb-switch"
          class:nb-switch-checked={model.prefs?.minimizeToTray ?? false}
          onclick={handleTrayToggle}
          aria-label={t("settings.minimizeToTray")}
        >
          <span class="nb-switch-thumb"></span>
        </button>
      </div>
    </div>
  </Card>
</div>
