<script lang="ts">
  import { onMount } from "svelte";
  import AppBackground from "../lib/components/AppBackground.svelte";
  import TitleBar from "../lib/components/TitleBar.svelte";
  import Sidebar from "../lib/components/Sidebar.svelte";
  import { router } from "./router.svelte";
  import { snapshotStore } from "../lib/api/snapshot.svelte";
  import { t } from "../lib/i18n";
  import { theme } from "../lib/design-system/theme.svelte";
  import { logger } from "../lib/logging";
  import SettingsScreen from "../features/settings/SettingsScreen.svelte";
  import PeersScreen from "../features/peers/PeersScreen.svelte";
  import { PeersModel } from "../features/peers/model.svelte";
  import SessionScreen from "../features/session/SessionScreen.svelte";
  import { SessionController } from "../features/session/controller.svelte";
  import ResultScreen from "../features/results/ResultScreen.svelte";
  import HistoryScreen from "../features/history/HistoryScreen.svelte";
  import ConsentDialog from "../features/consent/ConsentDialog.svelte";
  import { ConsentModel } from "../features/consent/model.svelte";
  import type { Peer } from "../lib/contracts/peer";
  import CloseDialog from "./CloseDialog.svelte";
  import { evaluateAppClose } from "../lib/api/settings";
  import { restoreAndShowWindow, setupWindowTracking } from "../lib/window";

  let showCloseDialog = $state(false);

  const peersModel = new PeersModel();
  const session = new SessionController();
  const consent = new ConsentModel();

  // Una solicitud entrante puede llegar en cualquier pantalla: se vigila siempre.
  onMount(() => {
    void consent.start();
    return () => consent.stop();
  });

  // La lista de equipos solo se sondea mientras Inicio está a la vista.
  $effect(() => {
    if (router.currentRoute !== "inicio") return;
    peersModel.start();
    return () => peersModel.stop();
  });

  async function handleSelectPeer(peer: Peer) {
    if (!peersModel.isTrusted(peer)) {
      await peersModel.beginPairing(peer);
      return;
    }
    router.navigate("session");
    await session.begin(peer);
  }

  async function handleRetry() {
    const peer = session.peer;
    if (!peer) return;
    router.navigate("session");
    await session.begin(peer);
  }

  function handleNewTest() {
    session.reset();
    router.navigate("inicio");
  }

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
        <PeersScreen
          peers={peersModel.peers}
          isScanning={peersModel.scanning}
          statusOf={peersModel.statusOf}
          errorMessage={peersModel.error}
          onDismissError={() => peersModel.clearError()}
          busy={peersModel.busy}
          onSelectPeer={(peer) => void handleSelectPeer(peer)}
          onManualConnect={(host, port) => void peersModel.manualConnect(host, port)}
          activePairingPeer={peersModel.pairing?.peer ?? null}
          pairingCode={peersModel.pairing?.code ?? ""}
          pairingSecondsLeft={peersModel.pairing?.secondsLeft ?? 60}
          onCancelPairing={() => void peersModel.cancelPairing()}
          onConfirmPairing={() => void peersModel.confirmPairing()}
        />
      {:else if router.currentRoute === "session" && session.peer}
        <SessionScreen
          peer={session.peer}
          plan={session.plan}
          phase={session.phase === "idle" ? "preparing" : session.phase}
          progressPercent={session.progressPercent}
          currentThroughputBps={session.currentBps}
          errorMessage={session.errorMessage}
          onCancel={() => void session.cancel()}
          onViewResults={() => router.navigate("results")}
          onRetry={() => void handleRetry()}
        />
      {:else if router.currentRoute === "results" && session.result}
        <ResultScreen
          result={session.result}
          samplesForward={session.samplesForward}
          samplesReverse={session.samplesReverse}
          onRetest={() => void handleRetry()}
          onNewTest={handleNewTest}
        />
      {:else if router.currentRoute === "results"}
        <div class="space-y-4" role="alert">
          <p>{session.errorMessage || t("session.errors.noResult")}</p>
          <button type="button" class="underline" onclick={handleNewTest}>
            {t("nav.peers")}
          </button>
        </div>
      {:else if router.currentRoute === "historial"}
        <HistoryScreen />
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

  <ConsentDialog model={consent} />
</div>
