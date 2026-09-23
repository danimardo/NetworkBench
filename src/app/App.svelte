<script lang="ts">
  import { onMount } from "svelte";
  import AppBackground from "../lib/components/AppBackground.svelte";
  import TitleBar from "../lib/components/TitleBar.svelte";
  import Sidebar from "../lib/components/Sidebar.svelte";
  import Card from "../lib/components/Card.svelte";
  import StatusPill from "../lib/components/StatusPill.svelte";
  import { router } from "./router.svelte";
  import { snapshotStore } from "../lib/api/snapshot.svelte";
  import { t } from "../lib/i18n";
  import { theme } from "../lib/design-system/theme.svelte";
  import { logger } from "../lib/logging";
  import SettingsScreen from "../features/settings/SettingsScreen.svelte";
  import CloseDialog from "./CloseDialog.svelte";
  import { evaluateAppClose } from "../lib/api/settings";
  import { restoreAndShowWindow, setupWindowTracking } from "../lib/window";

  let showCloseDialog = $state(false);

  onMount(() => {
    logger.installGlobalErrorCapture();
    void snapshotStore.init();
    if (theme.mode === "system") {
      theme.setMode("dark");
    }
    void restoreAndShowWindow();
    void setupWindowTracking();

    void (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        await win.onCloseRequested(async (event) => {
          const decision = await evaluateAppClose(false);
          if (decision === "requireConfirmation") {
            event.preventDefault();
            showCloseDialog = true;
          }
        });
      } catch {
        // Entorno de test o navegador sin backend Tauri
      }
    })();
  });

  function handleNavigate(routeId: string) {
    if (routeId === "inicio" || routeId === "historial" || routeId === "ajustes") {
      router.navigate(routeId);
    }
  }

  function handleConfirmExit() {
    showCloseDialog = false;
    // Forzar salida
    window.close();
  }
</script>

<div class="relative flex h-screen flex-col overflow-hidden text-[var(--color-text-primary)]">
  <!-- Fondo visual ambiental desacoplado -->
  <AppBackground state={snapshotStore.snapshot?.isSessionActive ? "warning" : "idle"} />

  <!-- Barra de título accesible con controles de ventana y arrastre -->
  <TitleBar appName={t("app.name")} />

  <div class="relative z-10 flex flex-1 overflow-hidden">
    <!-- Barra lateral de navegación principal -->
    <Sidebar
      activeId={router.currentRoute}
      onNavigate={handleNavigate}
      disabled={snapshotStore.snapshot?.isSessionActive ?? false}
      disabledHint={t("session.running")}
    />

    <!-- Área de contenido de la shell -->
    <main class="flex-1 overflow-y-auto p-6" id="main-content" tabindex="-1">
      {#if router.currentRoute === "inicio"}
        <div class="space-y-6">
          <header class="flex items-center justify-between">
            <div>
              <h1 class="text-2xl font-bold tracking-tight">{t("nav.peers")}</h1>
              <p class="text-sm text-[var(--color-text-secondary)]">{t("peers.discover")}</p>
            </div>
            <StatusPill tone={snapshotStore.snapshot?.isSessionActive ? "warning" : "neutral"}>
              {snapshotStore.snapshot?.isSessionActive ? t("session.running") : t("common.loading")}
            </StatusPill>
          </header>

          <Card variant="default">
            <div class="p-6 text-center">
              <p class="text-base font-semibold">{t("peers.emptyTitle")}</p>
              <p class="mt-1 text-sm text-[var(--color-text-secondary)]">{t("peers.emptyDesc")}</p>
            </div>
          </Card>
        </div>
      {:else if router.currentRoute === "historial"}
        <div class="space-y-6">
          <header>
            <h1 class="text-2xl font-bold tracking-tight">{t("history.title")}</h1>
            <p class="text-sm text-[var(--color-text-secondary)]">{t("history.empty")}</p>
          </header>
        </div>
      {:else if router.currentRoute === "ajustes"}
        <SettingsScreen />
      {/if}
    </main>
  </div>

  <CloseDialog
    open={showCloseDialog}
    oncancel={() => (showCloseDialog = false)}
    onconfirm={handleConfirmExit}
  />
</div>
