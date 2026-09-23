<script lang="ts">
  import DeviceCard from "../../lib/components/DeviceCard.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Dialog from "../../lib/components/Dialog.svelte";
  import TextField from "../../lib/components/TextField.svelte";
  import VerificationCode from "../../lib/components/VerificationCode.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import { t } from "../../lib/i18n";
  import type { Peer } from "../../lib/contracts/peer";

  interface Props {
    peers?: Peer[];
    isScanning?: boolean;
    onSelectPeer?: (peer: Peer) => void;
    onManualConnect?: (address: string, port: number) => void;
    onCancelPairing?: () => void;
    onConfirmPairing?: (peer: Peer, code: string) => void;
    activePairingPeer?: Peer | null;
    pairingCode?: string;
    pairingSecondsLeft?: number;
  }

  let {
    peers = [],
    isScanning = false,
    onSelectPeer,
    onManualConnect,
    onCancelPairing,
    onConfirmPairing,
    activePairingPeer = null,
    pairingCode = "",
    pairingSecondsLeft = 60,
  }: Props = $props();

  let showManualModal = $state(false);
  let manualAddress = $state("");
  let manualPort = $state("7411");
  let manualError = $state("");

  function handleOpenManual() {
    manualAddress = "";
    manualPort = "7411";
    manualError = "";
    showManualModal = true;
  }

  function handleManualSubmit(e: SubmitEvent) {
    e.preventDefault();
    const addr = manualAddress.trim();
    const portNum = parseInt(manualPort, 10);

    if (!addr) {
      manualError = "Introduce una dirección IP o nombre de host válido";
      return;
    }
    if (isNaN(portNum) || portNum < 1024 || portNum > 65535) {
      manualError = "El puerto debe ser un número entre 1024 y 65535";
      return;
    }

    manualError = "";
    showManualModal = false;
    onManualConnect?.(addr, portNum);
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
      <h1 class="nb-peers-title">Equipos disponibles</h1>
      <p class="nb-peers-subtitle">
        Selecciona un equipo de la red local para iniciar una medición de rendimiento de red.
      </p>
    </div>
    <Button variant="secondary" onclick={handleOpenManual} data-testid="manual-connect-btn">
      <Icon name="link" size={14} />
      Conectar manualmente
    </Button>
  </header>

  {#if peers.length === 0}
    <div class="nb-peers-empty" role="status" aria-live="polite">
      <div class="nb-peers-empty-icon">
        <Icon name="search" size={28} />
      </div>
      <h2 class="nb-peers-empty-title">
        {isScanning ? "Buscando equipos en la red local..." : "No se han detectado equipos"}
      </h2>
      <p class="nb-peers-empty-desc">
        Asegúrate de que NetworkBench esté abierto en el otro equipo o utiliza la conexión manual.
      </p>
      {#if !isScanning}
        <Button variant="primary" onclick={handleOpenManual}>Conectar por IP</Button>
      {/if}
    </div>
  {:else}
    <div class="nb-peers-grid" role="list">
      {#each peers as peer (peer.fingerprint)}
        <div role="listitem">
          <DeviceCard
            name={peer.displayName}
            alias={peer.alias ?? ""}
            ip={peer.addresses[0] ?? "127.0.0.1"}
            adapterType="ethernet"
            trust={mapTrust(peer)}
            availability="available"
            compatible={true}
            onclick={() => onSelectPeer?.(peer)}
          />
        </div>
      {/each}
    </div>
  {/if}

  <!-- Diálogo de conexión manual -->
  {#if showManualModal}
    <Dialog title="Conexión manual por IP" onClose={() => (showManualModal = false)}>
      <form onsubmit={handleManualSubmit} class="nb-manual-form">
        <p class="nb-manual-desc">
          Introduce la dirección IP o nombre DNS y el puerto de control (por defecto 7411).
        </p>

        <TextField
          label="Dirección IP o Hostname"
          placeholder="ej. 192.168.1.50 o pc-laboratorio"
          bind:value={manualAddress}
          error={manualError ? manualError : undefined}
        />

        <TextField label="Puerto de control" placeholder="7411" bind:value={manualPort} />

        <div class="nb-dialog-actions">
          <Button variant="ghost" onclick={() => (showManualModal = false)}>
            {t("common.cancel")}
          </Button>
          <Button variant="primary" type="submit">Conectar</Button>
        </div>
      </form>
    </Dialog>
  {/if}

  <!-- Diálogo de emparejamiento (Pairing modal) -->
  {#if activePairingPeer}
    <Dialog title="Emparejar equipo nuevo" onClose={() => onCancelPairing?.()}>
      <div class="nb-pairing-content">
        <p class="nb-pairing-instructions">
          Comprueba que el siguiente código de 6 dígitos coincide exactamente en ambos equipos.
        </p>

        <div class="nb-pairing-code-box">
          <VerificationCode code={pairingCode || "------"} />
        </div>

        <div class="nb-pairing-timer" role="timer" aria-live="off">
          <Icon name="clock" size={14} />
          <span>Caduca en {pairingSecondsLeft} s</span>
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
          >
            Confirmar código
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

  .nb-peers-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-12) var(--space-6);
    background: var(--color-surface);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-lg);
    text-align: center;
    gap: var(--space-3);
  }

  .nb-peers-empty-icon {
    width: 56px;
    height: 56px;
    border-radius: var(--radius-full);
    background: var(--icon-chip-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-accent);
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
