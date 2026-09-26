<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  function handleTrayToggle(value: boolean) {
    void model.update({ minimizeToTray: value });
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
        <Switch
          checked={model.autostart}
          onchange={() => model.toggleAutostart()}
          label={t("settings.autostart")}
        />
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
        <Switch
          checked={model.prefs?.minimizeToTray ?? false}
          onchange={handleTrayToggle}
          disabled={!model.prefs}
          label={t("settings.minimizeToTray")}
        />
      </div>
    </div>
  </Card>
</div>
