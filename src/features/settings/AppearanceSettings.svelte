<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Segmented from "../../lib/components/Segmented.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import { theme, type ThemeMode } from "../../lib/design-system/theme.svelte";
  import { getLocale, setLocale, t, type SupportedLocale } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  // Los nombres de idioma se escriben en su propio idioma a propósito: quien no entiende
  // el idioma actual tiene que poder reconocer el suyo.
  const LOCALE_OPTIONS: readonly { id: SupportedLocale; label: string }[] = [
    { id: "es", label: "Español" },
    { id: "en", label: "English" },
  ];

  function handleLocaleChange(loc: SupportedLocale) {
    // Cambia al instante en pantalla y se guarda, para recordarlo al reabrir la app.
    setLocale(loc);
    void model.update({ locale: loc });
  }

  const themeOptions = $derived<{ id: ThemeMode; label: string }[]>([
    { id: "dark", label: t("settings.themeDark") },
    { id: "light", label: t("settings.themeLight") },
    { id: "system", label: t("settings.themeSystem") },
  ]);
</script>

<div class="space-y-6">
  <!-- Tarjeta de Tema de Interfaz -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold">{t("settings.theme")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.appearance")}
          </p>
        </div>
        <Segmented
          options={themeOptions}
          value={theme.mode}
          onchange={(mode) => theme.setMode(mode)}
          label={t("settings.theme")}
        />
      </div>
    </div>
  </Card>

  <!-- Tarjeta de Idioma -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold">{t("settings.language")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">{t("settings.languageDesc")}</p>
        </div>
        <Segmented
          options={LOCALE_OPTIONS}
          value={getLocale()}
          onchange={handleLocaleChange}
          label={t("settings.language")}
        />
      </div>
    </div>
  </Card>

  <!-- Tarjeta de Reducción de Movimiento -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.reduceMotion")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.reduceMotionDesc")}
          </p>
        </div>
        <Switch
          checked={theme.reduceMotion}
          onchange={(v) => theme.setReduceMotion(v)}
          label={t("settings.reduceMotion")}
        />
      </div>
    </div>
  </Card>
</div>
