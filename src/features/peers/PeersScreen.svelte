<script lang="ts">
  import { tick } from "svelte";
  import DeviceCard from "../../lib/components/DeviceCard.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Dialog from "../../lib/components/Dialog.svelte";
  import TextField from "../../lib/components/TextField.svelte";
  import VerificationCode from "../../lib/components/VerificationCode.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import Tooltip from "../../lib/components/Tooltip.svelte";
  import ContextMenu, { type ContextMenuItem } from "../../lib/components/ContextMenu.svelte";
  import { t } from "../../lib/i18n";
  import type { Peer } from "../../lib/contracts/peer";

  interface Props {
    peers?: Peer[];
    isScanning?: boolean;
    onSelectPeer?: (peer: Peer) => void;
    onManualConnect?: (address: string, port: number) => void;
    /** «Buscar de nuevo»: vuelve a preguntar a la red por equipos. */
    onRescan?: () => void;
    onCancelPairing?: () => void;
    onConfirmPairing?: (peer: Peer, code: string) => void;
    activePairingPeer?: Peer | null;
    pairingCode?: string;
    pairingSecondsLeft?: number;
    /** Disponibilidad y compatibilidad por equipo; sin ella todos se muestran disponibles. */
    statusOf?: (peer: Peer) => {
      availability: "available" | "busy" | "checking" | "unreachable";
      compatible: boolean;
    };
    errorMessage?: string | null;
    onDismissError?: () => void;
    busy?: boolean;
    /** ¿Hay un registro guardado del equipo? Sin él, confianza y eliminar no aplican. */
    isSaved?: (peer: Peer) => boolean;
    onToggleFavorite?: (peer: Peer) => void;
    onRevokeTrust?: (peer: Peer) => void;
    onForget?: (peer: Peer) => void;
    onCheckNow?: (peer: Peer) => void;
  }

  let {
    peers = [],
    isScanning = false,
    onSelectPeer,
    onManualConnect,
    onRescan,
    onCancelPairing,
    onConfirmPairing,
    activePairingPeer = null,
    pairingCode = "",
    pairingSecondsLeft = 60,
    statusOf,
    errorMessage = null,
    onDismissError,
    busy = false,
    isSaved = () => true,
    onToggleFavorite,
    onRevokeTrust,
    onForget,
    onCheckNow,
  }: Props = $props();

  let showManualModal = $state(false);
  let manualAddress = $state("");
  let manualPort = $state("7411");
  let manualError = $state("");
  // Quién abrió el diálogo (§13 del correo: el retorno de foco lo gestiona quien lo
  // instancia, Dialog.svelte solo mueve el foco hacia dentro al montarse). Hay dos
  // botones que pueden abrirlo (el de la cabecera y el del estado
  // vacío): `document.activeElement` en el momento de abrir vale para cualquiera de los dos.
  let manualModalTrigger: HTMLElement | null = null;

  function handleOpenManual() {
    manualModalTrigger = document.activeElement as HTMLElement | null;
    manualAddress = "";
    manualPort = "7411";
    manualError = "";
    showManualModal = true;
  }

  async function closeManualModal() {
    showManualModal = false;
    // Sin esperar el siguiente `tick`, el botón desencadenante todavía estaría marcado
    // `inert` por el diálogo que se está destruyendo (Dialog.svelte quita `inert` en su
    // `onDestroy`, que Svelte no ejecuta de forma síncrona con esta asignación) y
    // `.focus()` no haría nada.
    await tick();
    manualModalTrigger?.focus();
    manualModalTrigger = null;
  }

  function handleManualSubmit(e: SubmitEvent) {
    e.preventDefault();
    const addr = manualAddress.trim();
    const portNum = parseInt(manualPort, 10);

    if (!addr) {
      manualError = t("peers.screen.errAddress");
      return;
    }
    if (isNaN(portNum) || portNum < 1024 || portNum > 65535) {
      manualError = t("peers.screen.errPort");
      return;
    }

    manualError = "";
    onManualConnect?.(addr, portNum);
    closeManualModal();
  }

  /** Sin efecto mientras ya está buscando: no se encadenan búsquedas. */
  function handleRescan() {
    if (isScanning) return;
    onRescan?.();
  }

  // Menú contextual (clic derecho, tecla de menú o Mayús+F10 sobre una tarjeta).
  let menu = $state<{ peer: Peer; x: number; y: number } | null>(null);
  let menuTrigger: HTMLElement | null = null;
  let peerToForget = $state<Peer | null>(null);
  let forgetTrigger: HTMLElement | null = null;

  function openMenu(e: MouseEvent, peer: Peer) {
    e.preventDefault();
    const host = e.currentTarget as HTMLElement;
    menuTrigger = (host.querySelector('[role="button"]') as HTMLElement | null) ?? host;
    // Desde el teclado el evento no trae posición útil (0,0): se ancla a la tarjeta.
    const desdeTeclado = e.clientX === 0 && e.clientY === 0;
    const r = host.getBoundingClientRect();
    menu = {
      peer,
      x: desdeTeclado ? r.left + 24 : e.clientX,
      y: desdeTeclado ? r.top + r.height / 2 : e.clientY,
    };
  }

  async function closeMenu() {
    menu = null;
    await tick();
    menuTrigger?.focus();
    menuTrigger = null;
  }

  function menuItems(peer: Peer): ContextMenuItem[] {
    const guardado = isSaved(peer);
    const confiable = peer.trustState === "trusted" || peer.trustState === "trustedAutoAccept";
    return [
      {
        id: "favorite",
        label: peer.favorite ? t("peers.menu.unfavorite") : t("peers.menu.favorite"),
        icon: peer.favorite ? "star-outline" : "star",
        disabled: !guardado,
      },
      { id: "copyIp", label: t("peers.menu.copyIp"), icon: "copy", disabled: !peer.addresses[0] },
      {
        id: "checkNow",
        label: t("peers.menu.checkNow"),
        icon: "refresh",
        disabled: !guardado,
      },
      {
        id: "revokeTrust",
        label: t("peers.menu.revokeTrust"),
        icon: "shield",
        disabled: !guardado || !confiable,
        separatorBefore: true,
      },
      {
        id: "forget",
        label: t("peers.menu.forget"),
        icon: "trash",
        danger: true,
        disabled: !guardado,
      },
    ];
  }

  async function copiarIp(peer: Peer) {
    const direccion = peer.addresses[0];
    if (!direccion) return;
    // Solo el host: el puerto de control no le sirve a quien pega la IP.
    const host = direccion.startsWith("[")
      ? direccion.slice(1, direccion.indexOf("]"))
      : direccion.replace(/:\d+$/, "");
    try {
      await navigator.clipboard.writeText(host);
    } catch {
      // Sin permiso del portapapeles no hay nada útil que hacer: la IP sigue visible en la tarjeta.
    }
  }

  async function onMenuSelect(id: string) {
    const actual = menu?.peer;
    if (!actual) return;
    if (id === "forget") {
      // La confirmación toma el relevo del foco: se cierra el menú sin devolverlo a la tarjeta.
      forgetTrigger = menuTrigger;
      menu = null;
      menuTrigger = null;
      peerToForget = actual;
      return;
    }
    if (id === "favorite") onToggleFavorite?.(actual);
    else if (id === "revokeTrust") onRevokeTrust?.(actual);
    else if (id === "checkNow") onCheckNow?.(actual);
    else if (id === "copyIp") await copiarIp(actual);
    await closeMenu();
  }

  async function closeForget() {
    peerToForget = null;
    await tick();
    forgetTrigger?.focus();
    forgetTrigger = null;
  }

  function confirmForget() {
    const peer = peerToForget;
    if (peer) onForget?.(peer);
    peerToForget = null;
    forgetTrigger = null;
  }

  function mapTrust(peer: Peer) {
    switch (peer.trustState) {
      case "trusted":
      case "trustedAutoAccept":
        return "trusted" as const;
      case "known":
        return "known" as const;
      default:
        return "unknown" as const;
    }
  }
</script>

<div class="nb-peers-container">
  <header class="nb-peers-header">
    <div class="nb-peers-title-group">
      <h1 class="nb-peers-title">{t("peers.screen.title")}</h1>
      <p class="nb-peers-subtitle">
        {t("peers.screen.subtitle")}
      </p>
    </div>
    <div class="nb-peers-actions">
      <!-- Siempre disponible: también con equipos en la lista puede aparecer otro nuevo. -->
      <Tooltip text={t("peers.screen.rescan")} placement="bottom">
        {#snippet children(tooltipId)}
          <button
            type="button"
            class="nb-iconbtn"
            class:nb-iconbtn-spinning={isScanning}
            aria-label={t("peers.screen.rescan")}
            aria-describedby={tooltipId}
            disabled={isScanning}
            onclick={handleRescan}
            data-testid="rescan-btn"
          >
            <Icon name="refresh" size={16} />
          </button>
        {/snippet}
      </Tooltip>
      <!-- Con la lista vacía ya está el botón grande del centro: nunca hay dos a la vez. -->
      {#if peers.length > 0}
        <Button variant="secondary" onclick={handleOpenManual} data-testid="manual-connect-btn">
          {#snippet icon()}<Icon name="link" size={14} />{/snippet}
          {t("peers.screen.connectByIp")}
        </Button>
      {/if}
    </div>
  </header>

  {#if errorMessage}
    <div class="nb-peers-error" role="alert" data-testid="peers-error">
      <span>{errorMessage}</span>
      <Button variant="ghost" onclick={() => onDismissError?.()}>{t("common.close")}</Button>
    </div>
  {/if}

  {#if peers.length === 0}
    <div class="nb-peers-empty" role="status" aria-live="polite">
      <!-- La lupa es el botón de volver a buscar (mismo comportamiento que el de la cabecera). -->
      <button
        type="button"
        class="nb-peers-empty-icon"
        class:nb-peers-empty-icon-scanning={isScanning}
        aria-label={t("peers.screen.rescan")}
        disabled={isScanning}
        onclick={handleRescan}
        data-testid="rescan-empty-btn"
      >
        <Icon name="search" size={28} />
      </button>
      <h2 class="nb-peers-empty-title">
        {isScanning ? t("peers.discover") : t("peers.screen.emptyTitle")}
      </h2>
      <p class="nb-peers-empty-desc">
        {t("peers.screen.emptyDesc")}
      </p>
      {#if !isScanning}
        <Button variant="primary" onclick={handleOpenManual} data-testid="manual-connect-empty-btn">
          {t("peers.screen.connectByIp")}
        </Button>
      {/if}
    </div>
  {:else}
    <div class="nb-peers-grid" role="list">
      {#each peers as peer, i (peer.fingerprint)}
        <div
          role="listitem"
          class="nb-enter"
          style:--nb-i={i}
          oncontextmenu={(e) => openMenu(e, peer)}
        >
          <DeviceCard
            name={peer.displayName}
            alias={peer.alias ?? ""}
            ip={peer.addresses[0] ?? ""}
            adapterType="ethernet"
            trust={mapTrust(peer)}
            availability={statusOf?.(peer).availability ?? "available"}
            compatible={statusOf?.(peer).compatible ?? true}
            favorite={peer.favorite ?? false}
            lastSeen={peer.lastSeen}
            onToggleFavorite={() => onToggleFavorite?.(peer)}
            onclick={() => onSelectPeer?.(peer)}
          />
        </div>
      {/each}
    </div>
  {/if}

  {#if menu}
    <ContextMenu
      items={menuItems(menu.peer)}
      x={menu.x}
      y={menu.y}
      label={t("peers.menu.label", { name: menu.peer.displayName })}
      onSelect={(id) => void onMenuSelect(id)}
      onClose={() => void closeMenu()}
    />
  {/if}

  {#if peerToForget}
    <Dialog
      title={t("peers.menu.forgetTitle", { name: peerToForget.displayName })}
      onClose={closeForget}
    >
      <div class="nb-manual-form">
        <p class="nb-manual-desc">{t("peers.menu.forgetDesc")}</p>
        <div class="nb-dialog-actions">
          <Button variant="ghost" onclick={closeForget} data-testid="forget-cancel-btn">
            {t("common.cancel")}
          </Button>
          <Button variant="primary" onclick={confirmForget} data-testid="forget-confirm-btn">
            {t("peers.menu.forgetConfirm")}
          </Button>
        </div>
      </div>
    </Dialog>
  {/if}

  <!-- Diálogo de conexión manual -->
  {#if showManualModal}
    <Dialog title={t("peers.screen.manualTitle")} onClose={closeManualModal}>
      <form onsubmit={handleManualSubmit} class="nb-manual-form">
        <p class="nb-manual-desc">
          {t("peers.screen.manualDesc")}
        </p>

        <TextField
          label={t("peers.screen.addressLabel")}
          placeholder={t("peers.screen.addressPlaceholder")}
          bind:value={manualAddress}
          error={manualError ? manualError : undefined}
        />

        <TextField label={t("peers.screen.portLabel")} placeholder="7411" bind:value={manualPort} />

        <div class="nb-dialog-actions">
          <Button variant="ghost" onclick={closeManualModal}>
            {t("common.cancel")}
          </Button>
          <Button variant="primary" type="submit">{t("peers.connectBtn")}</Button>
        </div>
      </form>
    </Dialog>
  {/if}

  <!-- Diálogo de emparejamiento (Pairing modal) -->
  {#if activePairingPeer}
    <Dialog title={t("peers.screen.pairingTitle")} onClose={() => onCancelPairing?.()}>
      <div class="nb-pairing-content">
        <p class="nb-pairing-instructions">
          {t("peers.screen.pairingInstructions")}
        </p>

        <div class="nb-pairing-code-box">
          <VerificationCode code={pairingCode || "------"} />
        </div>

        <div class="nb-pairing-timer" role="timer" aria-live="off">
          <Icon name="clock" size={14} />
          <span>{t("peers.screen.pairingExpires", { seconds: pairingSecondsLeft })}</span>
        </div>

        <div class="nb-dialog-actions">
          <Button
            variant="ghost"
            onclick={() => onCancelPairing?.()}
            data-testid="pairing-cancel-btn"
          >
            {t("common.cancel")}
          </Button>
          <Button
            variant="primary"
            onclick={() => onConfirmPairing?.(activePairingPeer!, pairingCode)}
            data-testid="pairing-confirm-btn"
            disabled={busy}
          >
            {t("peers.screen.pairingConfirm")}
          </Button>
        </div>
      </div>
    </Dialog>
  {/if}
</div>

<style>
  .nb-peers-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    padding: var(--space-6);
    max-width: 1000px;
    margin: 0 auto;
    width: 100%;
  }

  .nb-peers-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .nb-peers-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .nb-iconbtn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
    background: var(--surface-secondary);
    color: var(--color-text-secondary);
    cursor: default;
    transition:
      background var(--duration-base) ease,
      border-color var(--duration-base) ease,
      color var(--duration-base) ease;
  }

  .nb-iconbtn:hover:not(:disabled) {
    background: var(--surface-secondary-hover);
    border-color: var(--border-hover);
    color: var(--color-text-primary);
  }

  .nb-iconbtn:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  .nb-iconbtn:disabled {
    color: var(--color-text-disabled);
  }

  .nb-iconbtn-spinning :global(svg) {
    animation: nb-peers-spin var(--duration-spinner) linear infinite;
  }

  @keyframes nb-peers-spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-iconbtn-spinning :global(svg) {
      animation: none;
    }
  }

  .nb-peers-title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-title);
    color: var(--color-text-primary);
    margin: 0;
  }

  .nb-peers-subtitle {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: var(--space-1) 0 0 0;
  }

  .nb-peers-error {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-md);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
  }

  .nb-peers-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-12) var(--space-6);
    background: var(--surface-secondary);
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-lg);
    text-align: center;
    gap: var(--space-3);
  }

  .nb-peers-empty-icon {
    width: 56px;
    height: 56px;
    border-radius: var(--radius-pill);
    background: var(--icon-chip-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-accent);
    border: none;
    padding: 0;
    cursor: default;
    transition:
      background var(--duration-base) ease,
      box-shadow var(--duration-base) ease;
  }

  .nb-peers-empty-icon:hover:not(:disabled) {
    box-shadow: 0 0 0 4px var(--hover-tint);
  }

  .nb-peers-empty-icon:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 3px;
  }

  .nb-peers-empty-icon-scanning {
    animation: nb-peers-pulse var(--duration-emphasis-pulse) ease-in-out infinite alternate;
  }

  @keyframes nb-peers-pulse {
    from {
      opacity: 1;
    }
    to {
      opacity: 0.45;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-peers-empty-icon-scanning {
      animation: none;
    }
  }

  .nb-peers-empty-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-title);
    color: var(--color-text-primary);
    margin: 0;
  }

  .nb-peers-empty-desc {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    max-width: 400px;
    margin: 0;
  }

  .nb-peers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--space-4);
  }

  .nb-manual-form,
  .nb-pairing-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .nb-manual-desc,
  .nb-pairing-instructions {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: 0;
  }

  .nb-pairing-code-box {
    display: flex;
    justify-content: center;
    padding: var(--space-4) 0;
  }

  .nb-pairing-timer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }

  .nb-dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
    margin-top: var(--space-2);
  }
</style>
