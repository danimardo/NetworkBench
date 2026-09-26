<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../lib/i18n";
  import { SettingsModel } from "./model.svelte";
  import AppearanceSettings from "./AppearanceSettings.svelte";
  import NetworkSettings from "./NetworkSettings.svelte";
  import TrustSettings from "./TrustSettings.svelte";
  import LifecycleSettings from "./LifecycleSettings.svelte";
  import FirewallSettings from "./FirewallSettings.svelte";
  import DataSettings from "./DataSettings.svelte";
  import DiagnosticSettings from "./DiagnosticSettings.svelte";
  import AboutScreen from "./AboutScreen.svelte";

  const model = new SettingsModel();

  type TabId =
    | "appearance"
    | "network"
    | "trust"
    | "lifecycle"
    | "firewall"
    | "data"
    | "diagnostics"
    | "about";

  let activeTab = $state<TabId>("appearance");
  let tabEls: Record<string, HTMLButtonElement | undefined> = {};

  /**
   * Navegación por flechas del `tablist` (WAI-ARIA APG, patrón «activación automática»):
   * hasta ahora solo se podía activar una pestaña con Tab + clic/Enter, sin flechas ni
   * Home/End, y las pestañas no seleccionadas quedaban en el orden de tabulación en vez de
   * fuera de él (`tabindex` en roving, más abajo).
   */
  function handleTabsKeydown(event: KeyboardEvent) {
    const i = TABS.findIndex((tab) => tab.id === activeTab);
    let next = i;
    if (event.key === "ArrowRight") next = (i + 1) % TABS.length;
    else if (event.key === "ArrowLeft") next = (i - 1 + TABS.length) % TABS.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = TABS.length - 1;
    else return;

    event.preventDefault();
    const destino = TABS[next];
    if (!destino) return;
    activeTab = destino.id;
    tabEls[destino.id]?.focus();
  }

  const TABS: { id: TabId; labelKey: string }[] = [
    { id: "appearance", labelKey: "settings.tabAppearance" },
    { id: "network", labelKey: "settings.tabNetwork" },
    { id: "trust", labelKey: "settings.tabTrust" },
    { id: "lifecycle", labelKey: "settings.tabLifecycle" },
    { id: "firewall", labelKey: "settings.tabFirewall" },
    { id: "data", labelKey: "settings.tabData" },
    { id: "diagnostics", labelKey: "settings.tabDiagnostics" },
    { id: "about", labelKey: "settings.tabAbout" },
  ];

  onMount(() => {
    model.load();
  });
</script>

<div class="space-y-6">
  <div>
    <h1 class="text-xl font-bold text-[var(--color-text-primary)]">
      {t("settings.title")}
    </h1>
    <p class="text-sm text-[var(--color-text-secondary)]">
      {t("settings.subtitle")}
    </p>
  </div>

  {#if model.saveSuccess}
    <div
      class="rounded-md bg-[var(--color-surface-success)] p-3 text-xs text-[var(--color-text-success)]"
      role="status"
    >
      {t("settings.saveSuccess")}
    </div>
  {/if}

  {#if model.saveError}
    <div
      class="rounded-md bg-[var(--color-surface-danger)] p-3 text-xs text-[var(--color-text-danger)]"
      role="alert"
    >
      {model.saveError}
    </div>
  {/if}

  <!-- Navegación por pestañas -->
  <div
    class="border-b border-[var(--color-border)]"
    role="tablist"
    aria-label={t("settings.title")}
    tabindex="-1"
    onkeydown={handleTabsKeydown}
  >
    <div class="flex gap-2 overflow-x-auto pb-px">
      {#each TABS as tab (tab.id)}
        <button
          bind:this={tabEls[tab.id]}
          type="button"
          role="tab"
          id="tab-{tab.id}"
          aria-controls="panel-{tab.id}"
          aria-selected={activeTab === tab.id}
          tabindex={activeTab === tab.id ? 0 : -1}
          class="whitespace-nowrap px-4 py-2 text-sm font-medium transition-colors border-b-2"
          class:border-[var(--color-primary)]={activeTab === tab.id}
          class:text-[var(--color-primary)]={activeTab === tab.id}
          class:border-transparent={activeTab !== tab.id}
          class:text-[var(--color-text-secondary)]={activeTab !== tab.id}
          onclick={() => (activeTab = tab.id)}
        >
          {t(tab.labelKey)}
        </button>
      {/each}
    </div>
  </div>

  <!-- Contenido de las pestañas -->
  <div id="panel-{activeTab}" role="tabpanel" aria-labelledby="tab-{activeTab}">
    {#if activeTab === "appearance"}
      <AppearanceSettings />
    {:else if activeTab === "network"}
      <NetworkSettings {model} />
    {:else if activeTab === "trust"}
      <TrustSettings {model} />
    {:else if activeTab === "lifecycle"}
      <LifecycleSettings {model} />
    {:else if activeTab === "firewall"}
      <FirewallSettings />
    {:else if activeTab === "data"}
      <DataSettings {model} />
    {:else if activeTab === "diagnostics"}
      <DiagnosticSettings {model} />
    {:else if activeTab === "about"}
      <AboutScreen {model} />
    {/if}
  </div>
</div>
