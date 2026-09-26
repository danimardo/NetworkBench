<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Switch from "../../lib/components/Switch.svelte";
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
    const trimmed = String(portInput ?? "").trim();
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

  function handleMdnsToggle(value: boolean) {
    void model.update({ mdnsEnabled: value });
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
          class="nb-input"
          bind:value={portInput}
          onblur={handlePortBlur}
          aria-label={t("settings.networkControlPort")}
        />
        <span class="text-xs text-[var(--color-text-muted)]">
          {t("settings.defaultPortNotice")}
        </span>
      </div>
      {#if portError}
        <p class="text-xs text-[var(--color-danger)]" role="alert">{portError}</p>
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
        <Switch
          checked={model.prefs?.mdnsEnabled ?? true}
          onchange={handleMdnsToggle}
          disabled={!model.prefs}
          label={t("settings.mdnsDiscovery")}
        />
      </div>
    </div>
  </Card>
</div>

<style>
  /* La clase `nb-input` no existía: el campo se pintaba con el aspecto nativo del navegador. */
  .nb-input {
    width: 10rem;
    padding: 9px var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-default);
    background: var(--surface-field);
    color: var(--color-text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    font-variant-numeric: tabular-nums;
  }

  .nb-input:hover {
    border-color: var(--border-hover);
  }

  .nb-input:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }
</style>
