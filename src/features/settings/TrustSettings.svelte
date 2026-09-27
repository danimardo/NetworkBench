<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  function handleAutoAcceptToggle(value: boolean) {
    void model.update({ autoAcceptTrusted: value });
  }
</script>

<div class="space-y-6">
  <!-- Auto-aceptación de equipos de confianza -->
  <Card variant="default" enterIndex={0}>
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.autoAcceptTrusted")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.autoAcceptTrustedDesc")}
          </p>
        </div>
        <Switch
          checked={model.prefs?.autoAcceptTrusted ?? false}
          onchange={handleAutoAcceptToggle}
          disabled={!model.prefs}
          label={t("settings.autoAcceptTrusted")}
        />
      </div>

      <div class="rounded bg-[var(--surface-field)] p-3 text-xs text-[var(--color-text-muted)]">
        {t("settings.autoAcceptSecurityNotice")}
      </div>
    </div>
  </Card>
</div>
