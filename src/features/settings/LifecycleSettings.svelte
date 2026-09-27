<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Segmented from "../../lib/components/Segmented.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  type CloseAction = "ask" | "minimize" | "exit";

  // Las etiquetas se calculan con `$derived` para que sigan al idioma.
  const closeOptions = $derived<{ id: CloseAction; label: string }[]>([
    { id: "ask", label: t("settings.closeActionAsk") },
    { id: "minimize", label: t("settings.closeActionMinimize") },
    { id: "exit", label: t("settings.closeActionExit") },
  ]);

  function handleCloseActionChange(value: CloseAction) {
    void model.update({ closeAction: value });
  }
</script>

<div class="space-y-6">
  <!-- Autoarranque con el sistema -->
  <Card variant="default" enterIndex={0}>
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

  <!-- Qué hace el botón de cerrar la ventana (§5.2) -->
  <Card variant="default" enterIndex={1}>
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between gap-4">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.closeAction")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.closeActionDesc")}
          </p>
        </div>
        <Segmented
          options={closeOptions}
          value={model.prefs?.closeAction}
          onchange={handleCloseActionChange}
          label={t("settings.closeAction")}
        />
      </div>
    </div>
  </Card>
</div>
