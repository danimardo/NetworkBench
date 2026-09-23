<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import { theme, type ThemeMode } from "../../lib/design-system/theme.svelte";
  import { getLocale, setLocale, t, type SupportedLocale } from "../../lib/i18n";

  let currentLocale = $state<SupportedLocale>(getLocale());

  function handleLocaleChange(loc: SupportedLocale) {
    currentLocale = loc;
    setLocale(loc);
  }

  const THEME_OPTIONS: { id: ThemeMode; labelKey: string }[] = [
    { id: "dark", labelKey: "settings.themeDark" },
    { id: "light", labelKey: "settings.themeLight" },
    { id: "system", labelKey: "settings.themeSystem" },
  ];
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
        <div class="nb-segmented-group" role="radiogroup" aria-label={t("settings.theme")}>
          {#each THEME_OPTIONS as opt (opt.id)}
            <button
              type="button"
              role="radio"
              aria-checked={theme.mode === opt.id}
              class="nb-segmented-item"
              class:nb-segmented-item-active={theme.mode === opt.id}
              onclick={() => theme.setMode(opt.id)}
            >
              {t(opt.labelKey)}
            </button>
          {/each}
        </div>
      </div>
    </div>
  </Card>

  <!-- Tarjeta de Idioma -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold">{t("settings.language")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">Español / English</p>
        </div>
        <div class="nb-segmented-group" role="radiogroup" aria-label={t("settings.language")}>
          <button
            type="button"
            role="radio"
            aria-checked={currentLocale === "es"}
            class="nb-segmented-item"
            class:nb-segmented-item-active={currentLocale === "es"}
            onclick={() => handleLocaleChange("es")}
          >
            Español
          </button>
          <button
            type="button"
            role="radio"
            aria-checked={currentLocale === "en"}
            class="nb-segmented-item"
            class:nb-segmented-item-active={currentLocale === "en"}
            onclick={() => handleLocaleChange("en")}
          >
            English
          </button>
        </div>
      </div>
    </div>
  </Card>

  <!-- Tarjeta de Reducción de Movimiento -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <label for="reduce-motion-toggle" class="cursor-pointer">
          <h2 class="text-base font-semibold">{t("settings.reduceMotion")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            Desactiva animaciones y transiciones no esenciales
          </p>
        </label>
        <input
          id="reduce-motion-toggle"
          type="checkbox"
          class="h-5 w-5 rounded border-[var(--border-default)] accent-[var(--color-brand-primary)] cursor-pointer"
          checked={theme.reduceMotion}
          onchange={(e) => theme.setReduceMotion((e.currentTarget as HTMLInputElement).checked)}
        />
      </div>
    </div>
  </Card>
</div>

<style>
  .nb-segmented-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--surface-secondary);
    border: 1px solid var(--border-default);
  }

  .nb-segmented-item {
    padding: 7px 14px;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-control);
    color: var(--color-text-secondary);
    cursor: pointer;
    background: transparent;
    border: none;
    transition:
      background var(--duration-base) ease,
      color var(--duration-base) ease;
  }

  .nb-segmented-item:hover:not(.nb-segmented-item-active) {
    background: var(--hover-tint);
  }

  .nb-segmented-item-active {
    background: var(--surface-raised);
    color: var(--color-text-primary);
    box-shadow: var(--shadow-sm);
  }

  .nb-segmented-item:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }
</style>
