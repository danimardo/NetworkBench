<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import AppIcon from "../../lib/components/AppIcon.svelte";
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
        <AppIcon size={56} framed />
        <div>
          <h2 class="text-lg font-bold">NetworkBench</h2>
          <p class="text-xs text-[var(--color-text-secondary)]">
            {t("settings.aboutSubtitle")}
          </p>
          <p class="mt-1 text-xs font-medium text-[var(--color-text-primary)]">
            {t("settings.developedBy")}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-4 rounded-lg bg-[var(--surface-field)] p-4 text-xs">
        <div>
          <span class="text-[var(--color-text-muted)] block">{t("settings.appVersion")}:</span>
          <span class="font-mono font-semibold">{model.aboutInfo?.appVersion ?? "—"}</span>
        </div>
        <div>
          <span class="text-[var(--color-text-muted)] block">{t("settings.buildHash")}:</span>
          <span class="font-mono font-semibold" data-testid="build-hash">
            {model.aboutInfo?.buildHash ?? "—"}
            {#if model.aboutInfo}
              · {model.aboutInfo.buildDate.slice(0, 10)}
            {/if}
            {#if model.aboutInfo?.buildDirty}
              <span class="text-[var(--color-warning)]">({t("settings.buildDirtyNotice")})</span>
            {/if}
          </span>
        </div>
        <div>
          <span class="text-[var(--color-text-muted)] block">{t("settings.protocolVersion")}:</span>
          <span class="font-mono font-semibold">{model.aboutInfo?.protocolVersion ?? "—"}</span>
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
        class="pt-2 border-t border-[var(--border-default)] text-xs text-[var(--color-text-muted)]"
      >
        {model.aboutInfo?.copyright ?? ""}
      </div>
    </div>
  </Card>
</div>
