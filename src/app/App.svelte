<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { scale, fade } from "svelte/transition";
  import { sectionEnterParams, sectionExitParams } from "./section-transition";
  import AppBackground from "../lib/components/AppBackground.svelte";
  import TitleBar from "../lib/components/TitleBar.svelte";
  import Sidebar from "../lib/components/Sidebar.svelte";
  import { router } from "./router.svelte";
  import { snapshotStore } from "../lib/api/snapshot.svelte";
  import { setLocale, t } from "../lib/i18n";
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
  import CloseFlow from "./CloseFlow.svelte";
  import { FirewallStartupCheck } from "../features/firewall/startup.svelte";
  import FirewallStartupNotice from "../features/firewall/FirewallStartupNotice.svelte";
  import { settingsNav } from "../features/settings/nav.svelte";
  import { restoreAndShowWindow, setupWindowTracking } from "../lib/window";

  const peersModel = new PeersModel();
  const session = new SessionController();
  const consent = new ConsentModel();
  const firewallCheck = new FirewallStartupCheck();

  function revisarCortafuegos() {
    settingsNav.requestedTab = "firewall";
    router.navigate("ajustes");
  }

  // Mientras el aviso está a la vista se vuelve a leer al cambiar de pantalla, para que no
  // siga avisando de algo que ya se arregló en Ajustes. `untrack`: no depende de sí mismo.
  $effect(() => {
    void router.currentRoute;
    untrack(() => {
      if (firewallCheck.visible) void firewallCheck.comprobar();
    });
  });

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

  /** Tope de espera al backend antes de mostrar la ventana, para que nunca se quede oculta. */
  const ESPERA_MAXIMA_IDIOMA_MS = 1500;

  /** El idioma elegido en Ajustes vive en `settings.json` y llega en el snapshot. */
  function aplicarIdiomaGuardado() {
    const guardado = snapshotStore.snapshot?.locale;
    if (guardado === "es" || guardado === "en") setLocale(guardado);
  }

  onMount(() => {
    logger.installGlobalErrorCapture();
    // El idioma se aplica en cuanto llega el snapshot, aunque sea tarde.
    const snapshotListo = snapshotStore.init().then(aplicarIdiomaGuardado);
    if (theme.mode === "system") {
      theme.setMode("dark");
    }
    // La ventana arranca oculta: se muestra con el idioma ya puesto para que quien usa
    // inglés no vea un destello en español, pero sin esperar más de ESPERA_MAXIMA_IDIOMA_MS.
    void Promise.race([
      snapshotListo,
      new Promise((resolver) => setTimeout(resolver, ESPERA_MAXIMA_IDIOMA_MS)),
    ]).then(() => restoreAndShowWindow());
    void setupWindowTracking();
    // En segundo plano: unos segundos de PowerShell sin elevación. No bloquea el arranque.
    void firewallCheck.comprobar();
  });

  function handleNavigate(routeId: string) {
    if (routeId === "inicio" || routeId === "historial" || routeId === "ajustes") {
      router.navigate(routeId);
    }
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
      {#key router.currentRoute}
        <div
          class="nb-screen-host"
          in:scale={sectionEnterParams(theme.reduceMotion)}
          out:fade={sectionExitParams(theme.reduceMotion)}
        >
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
              onRescan={() => void peersModel.rescan()}
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
        </div>
      {/key}
    </main>
  </div>

  <!-- Los diálogos del cierre de la ventana: el backend decide, aquí solo se pregunta. -->
  <CloseFlow />

  <ConsentDialog model={consent} />
</div>

<FirewallStartupNotice check={firewallCheck} onReview={revisarCortafuegos} />

<style>
  /* Sin esto, `in:scale` ancla el zoom en el centro geométrico DEL CONTENIDO
     (el valor por defecto de `transform-origin`), no de la ventana visible.
     En una pantalla más alta que el viewport (Historial con muchas filas,
     por ejemplo) ese centro cae muy por debajo de lo que se ve: al crecer
     del 90 % al 100 %, la cabecera y las primeras tarjetas —que están por
     encima del centro— se desplazan hacia arriba durante la animación, lo
     que se percibe como "sube desde abajo" aunque sea un escalado puro, sin
     ningún translateY (hallazgo real del propietario). Anclarlo arriba deja
     la cabecera fija: solo crece el contenido hacia abajo. */
  .nb-screen-host {
    transform-origin: top center;
  }
</style>
