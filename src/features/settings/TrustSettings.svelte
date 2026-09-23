<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  function handleAutoAcceptToggle() {
    if (!model.prefs) return;
    model.update({ autoAcceptTrusted: !model.prefs.autoAcceptTrusted });
  }
</script>

<div class="space-y-6">
  <!-- Auto-aceptación de equipos de confianza -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.autoAcceptTrusted")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.autoAcceptTrustedDesc")}
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={model.prefs?.autoAcceptTrusted ?? false}
          class="nb-switch"
          class:nb-switch-checked={model.prefs?.autoAcceptTrusted ?? false}
          onclick={handleAutoAcceptToggle}
          aria-label={t("settings.autoAcceptTrusted")}
        >
          <span class="nb-switch-thumb"></span>
        </button>
      </div>

      <div
        class="rounded bg-[var(--color-surface-sunken)] p-3 text-xs text-[var(--color-text-muted)]"
      >
        {t("settings.autoAcceptSecurityNotice")}
      </div>
    </div>
  </Card>
</div>
