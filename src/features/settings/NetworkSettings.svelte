<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  let portInput = $state<string>("");
  let portError = $state<string | null>(null);

  $effect(() => {
    if (model.prefs?.customControlPort) {
      portInput = String(model.prefs.customControlPort);
    }
  });

  function handlePortBlur() {
    portError = null;
    const trimmed = portInput.trim();
    if (!trimmed) {
      model.update({ customControlPort: null });
      return;
    }

    const val = parseInt(trimmed, 10);
    if (isNaN(val) || val < 1024 || val > 65000) {
      portError = t("settings.portRangeError");
      return;
    }

    model.update({ customControlPort: val });
  }

  function handleMdnsToggle() {
    if (!model.prefs) return;
    model.update({ mdnsEnabled: !model.prefs.mdnsEnabled });
  }
</script>

<div class="space-y-6">
  <!-- Puerto de Control Personalizado -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div>
        <h2 class="text-base font-semibold">{t("settings.networkControlPort")}</h2>
        <p class="text-sm text-[var(--color-text-secondary)]">
          {t("settings.networkControlPortDesc")}
        </p>
      </div>

      <div class="flex items-center gap-3">
        <input
          type="number"
          min="1024"
          max="65000"
          placeholder="7411"
          class="nb-input w-40"
          bind:value={portInput}
          onblur={handlePortBlur}
          aria-label={t("settings.networkControlPort")}
        />
        <span class="text-xs text-[var(--color-text-muted)]">
          {t("settings.defaultPortNotice")}
        </span>
      </div>
      {#if portError}
        <p class="text-xs text-[var(--color-text-danger)]" role="alert">{portError}</p>
      {/if}
    </div>
  </Card>

  <!-- Descubrimiento de Red Local mDNS -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.mdnsDiscovery")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.mdnsDiscoveryDesc")}
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={model.prefs?.mdnsEnabled ?? true}
          class="nb-switch"
          class:nb-switch-checked={model.prefs?.mdnsEnabled ?? true}
          onclick={handleMdnsToggle}
          aria-label={t("settings.mdnsDiscovery")}
        >
          <span class="nb-switch-thumb"></span>
        </button>
      </div>
    </div>
  </Card>
</div>
