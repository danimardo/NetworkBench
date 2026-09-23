<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();
</script>

<div class="space-y-6">
  <Card variant="default">
    <div class="p-6 space-y-6">
      <div class="flex items-center gap-4">
        <div
          class="flex h-12 w-12 items-center justify-center rounded-lg bg-[var(--color-primary)] text-white font-bold text-xl"
        >
          NB
        </div>
        <div>
          <h2 class="text-lg font-bold">NetworkBench</h2>
          <p class="text-xs text-[var(--color-text-secondary)]">
            {t("settings.aboutSubtitle")}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-4 rounded-lg bg-[var(--color-surface-sunken)] p-4 text-xs">
        <div>
          <span class="text-[var(--color-text-muted)] block">{t("settings.appVersion")}:</span>
          <span class="font-mono font-semibold">{model.aboutInfo?.appVersion ?? "0.1.0"}</span>
        </div>
        <div>
          <span class="text-[var(--color-text-muted)] block">{t("settings.protocolVersion")}:</span>
          <span class="font-mono font-semibold">{model.aboutInfo?.protocolVersion ?? "v1"}</span>
        </div>
        <div>
          <span class="text-[var(--color-text-muted)] block">{t("settings.engine")}:</span>
          <span class="font-mono font-semibold"
            >{model.aboutInfo?.engineName ?? "Microsoft NTTTCP"} ({model.aboutInfo?.engineVersion ??
              "5.35"})</span
          >
        </div>
        <div>
          <span class="text-[var(--color-text-muted)] block">{t("settings.license")}:</span>
          <span class="font-mono font-semibold"
            >{model.aboutInfo?.license ?? "GPL-3.0-or-later"}</span
          >
        </div>
      </div>

      <div class="space-y-2 text-xs text-[var(--color-text-secondary)] leading-relaxed">
        <p>
          {t("settings.licenseNotice")}
        </p>
        <p>
          {t("settings.attributionNotice")}
        </p>
      </div>

      <div
        class="pt-2 border-t border-[var(--color-border)] text-xs text-[var(--color-text-muted)]"
      >
        {model.aboutInfo?.copyright ?? "© 2026 Daniel Díez Mardomingo y colaboradores"}
      </div>
    </div>
  </Card>
</div>
