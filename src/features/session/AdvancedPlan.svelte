<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import Card from "../../lib/components/Card.svelte";
  import Icon from "../../lib/components/Icon.svelte";
  import TextField from "../../lib/components/TextField.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import { t } from "../../lib/i18n";
  import {
    benchmarkPlanSchema,
    type BenchmarkDirection,
    type BenchmarkPlan,
    type BenchmarkProtocol,
  } from "../../lib/contracts/plan";

  interface Props {
    plan?: BenchmarkPlan;
    isOpen?: boolean;
    onApply?: (plan: BenchmarkPlan) => void;
    onClose?: () => void;
  }

  let {
    plan = {
      protocol: "tcp",
      direction: "both_sequential",
      streams: 1,
      warmupSeconds: 2,
      measureSeconds: 20,
      cooldownSeconds: 2,
      port: 5001,
      bufferSizeBytes: "65536",
    },
    isOpen = $bindable(false),
    onApply,
    onClose,
  }: Props = $props();

  // Estado reactivo editable del formulario
  let protocol = $state<BenchmarkProtocol>("tcp");
  let direction = $state<BenchmarkDirection>("both_sequential");
  let streams = $state<number>(1);
  let measureSeconds = $state<number>(20);
  let warmupSeconds = $state<number>(2);
  let cooldownSeconds = $state<number>(2);
  let port = $state<number>(5001);
  let bufferSizeBytes = $state<string>("65536");
  let expectedCapacityMbps = $state<string>("");
  let udpTargetRateMbps = $state<string>("100");
  let udpPacketSizeBytes = $state<number>(1472);

  $effect.pre(() => {
    protocol = plan.protocol;
    direction = plan.direction;
    streams = plan.streams;
    measureSeconds = plan.measureSeconds;
    warmupSeconds = plan.warmupSeconds;
    cooldownSeconds = plan.cooldownSeconds;
    port = plan.port;
    bufferSizeBytes = plan.bufferSizeBytes ?? "65536";
    expectedCapacityMbps = plan.expectedCapacityBps
      ? (Number(plan.expectedCapacityBps) / 1_000_000).toString()
      : "";
    udpTargetRateMbps = plan.udpTargetRateBps
      ? (Number(plan.udpTargetRateBps) / 1_000_000).toString()
      : "100";
    udpPacketSizeBytes = plan.udpPacketSizeBytes ?? 1472;
  });

  // Límites según dirección
  let isSimultaneous = $derived(direction === "both" || direction === "both_simultaneous");
  let maxStreams = $derived(isSimultaneous ? 32 : 64);

  // Autoajuste de streams al cambiar a simultáneo
  $effect(() => {
    if (isSimultaneous && streams > 32) {
      streams = 32;
    }
  });

  // Construcción y validación del plan actual
  let currentPlan = $derived.by<BenchmarkPlan>(() => {
    return {
      protocol,
      direction,
      streams,
      measureSeconds,
      warmupSeconds,
      cooldownSeconds,
      port,
      bufferSizeBytes: bufferSizeBytes.trim() ? bufferSizeBytes.trim() : undefined,
      expectedCapacityBps: expectedCapacityMbps.trim()
        ? (Number(expectedCapacityMbps.trim()) * 1_000_000).toString()
        : undefined,
      udpTargetRateBps:
        protocol === "udp" && udpTargetRateMbps.trim()
          ? (Number(udpTargetRateMbps.trim()) * 1_000_000).toString()
          : undefined,
      udpPacketSizeBytes: protocol === "udp" ? udpPacketSizeBytes : undefined,
    };
  });

  let validationResult = $derived.by<{ isValid: boolean; error: string | null }>(() => {
    const result = benchmarkPlanSchema.safeParse(currentPlan);
    if (!result.success) {
      return {
        isValid: false,
        error: result.error.issues?.[0]?.message ?? "Parámetros fuera de los límites permitidos",
      };
    }
    return { isValid: true, error: null };
  });

  // Cálculo de puertos asignados para la vista previa
  let allocatedPortsText = $derived.by<string>(() => {
    if (isSimultaneous) {
      return `${port}..${port + 31} (envío) y ${port + 32}..${port + 63} (recepción)`;
    }
    const endPort = port + Math.max(1, streams) - 1;
    return `${port}..${endPort} (${streams} ${streams === 1 ? "puerto" : "puertos"})`;
  });

  // Estimación de tiempo total
  let estimatedTotalSeconds = $derived.by<number>(() => {
    const perPhase = warmupSeconds + measureSeconds + cooldownSeconds;
    if (direction === "both_sequential") {
      return perPhase * 2 + 1; // 1s switching
    }
    return perPhase;
  });

  function restoreDefaults() {
    protocol = "tcp";
    direction = "both_sequential";
    streams = 1;
    measureSeconds = 20;
    warmupSeconds = 2;
    cooldownSeconds = 2;
    port = 5001;
    bufferSizeBytes = "65536";
    expectedCapacityMbps = "";
    udpTargetRateMbps = "100";
    udpPacketSizeBytes = 1472;
  }

  function handleApply() {
    if (validationResult.isValid) {
      onApply?.(currentPlan);
      isOpen = false;
    }
  }
</script>

<div class="advanced-plan-container" role="region" aria-label={t("advanced.title")}>
  <div class="header-row">
    <div class="title-block">
      <h3 class="title">{t("advanced.title")}</h3>
      <p class="subtitle">{t("advanced.subtitle")}</p>
    </div>
    <div class="header-actions">
      <Button variant="ghost" onclick={restoreDefaults}>
        {t("advanced.restore_defaults")}
      </Button>
      {#if onClose}
        <Button variant="ghost" onclick={onClose} aria-label="Cerrar opciones avanzadas">✕</Button>
      {/if}
    </div>
  </div>

  <!-- Selector de Protocolo (TCP / UDP) -->
  <div class="form-section">
    <label class="section-label" for="protocol-segmented">{t("advanced.protocol")}</label>
    <div
      class="segmented-control"
      id="protocol-segmented"
      role="radiogroup"
      aria-label={t("advanced.protocol")}
    >
      <button
        type="button"
        role="radio"
        aria-checked={protocol === "tcp"}
        class="segment-btn"
        class:active={protocol === "tcp"}
        onclick={() => (protocol = "tcp")}
      >
        {t("advanced.tcp")}
      </button>
      <button
        type="button"
        role="radio"
        aria-checked={protocol === "udp"}
        class="segment-btn"
        class:active={protocol === "udp"}
        onclick={() => (protocol = "udp")}
      >
        {t("advanced.udp")}
      </button>
    </div>
  </div>

  <!-- Aviso explicativo UDP (§18) -->
  {#if protocol === "udp"}
    <div class="info-callout" role="note">
      <Icon name="info" size={20} />
      <p>{t("advanced.udp_notice")}</p>
    </div>
  {/if}

  <!-- Selector de Dirección -->
  <div class="form-section">
    <label class="section-label" for="direction-segmented">{t("advanced.direction")}</label>
    <div
      class="direction-grid"
      id="direction-segmented"
      role="radiogroup"
      aria-label={t("advanced.direction")}
    >
      <button
        type="button"
        role="radio"
        aria-checked={direction === "both_sequential"}
        class="dir-btn"
        class:active={direction === "both_sequential"}
        onclick={() => (direction = "both_sequential")}
      >
        {t("advanced.dir_both_seq")}
      </button>
      <button
        type="button"
        role="radio"
        aria-checked={direction === "forward"}
        class="dir-btn"
        class:active={direction === "forward"}
        onclick={() => (direction = "forward")}
      >
        {t("advanced.dir_forward")}
      </button>
      <button
        type="button"
        role="radio"
        aria-checked={direction === "reverse"}
        class="dir-btn"
        class:active={direction === "reverse"}
        onclick={() => (direction = "reverse")}
      >
        {t("advanced.dir_reverse")}
      </button>
      <button
        type="button"
        role="radio"
        aria-checked={isSimultaneous}
        class="dir-btn"
        class:active={isSimultaneous}
        onclick={() => (direction = "both_simultaneous")}
      >
        {t("advanced.dir_both_sim")}
      </button>
    </div>
    {#if isSimultaneous}
      <p class="hint-text">{t("advanced.simultaneous_notice")}</p>
    {/if}
  </div>

  <!-- Grid de parámetros numéricos -->
  <div class="params-grid">
    <TextField
      label={`${t("advanced.streams")} (1–${maxStreams})`}
      hint={t("advanced.streams_hint")}
      value={streams.toString()}
      oninput={(val) => {
        const n = parseInt(val, 10);
        if (!isNaN(n)) streams = n;
      }}
    />

    <TextField
      label={t("advanced.duration")}
      hint={t("advanced.duration_hint")}
      value={measureSeconds.toString()}
      oninput={(val) => {
        const n = parseInt(val, 10);
        if (!isNaN(n)) measureSeconds = n;
      }}
    />

    <TextField
      label={t("advanced.warmup")}
      hint="0–10 s"
      value={warmupSeconds.toString()}
      oninput={(val) => {
        const n = parseInt(val, 10);
        if (!isNaN(n)) warmupSeconds = n;
      }}
    />

    <TextField
      label={t("advanced.cooldown")}
      hint="0–10 s"
      value={cooldownSeconds.toString()}
      oninput={(val) => {
        const n = parseInt(val, 10);
        if (!isNaN(n)) cooldownSeconds = n;
      }}
    />

    <TextField
      label={t("advanced.port")}
      hint="1024–65000"
      value={port.toString()}
      oninput={(val) => {
        const n = parseInt(val, 10);
        if (!isNaN(n)) port = n;
      }}
    />

    <TextField
      label={t("advanced.buffer_size")}
      hint="4096..4194304 (potencias de 2)"
      value={bufferSizeBytes}
      oninput={(val) => (bufferSizeBytes = val)}
    />

    <TextField
      label={t("advanced.expected_capacity")}
      hint={t("advanced.expected_capacity_hint")}
      value={expectedCapacityMbps}
      oninput={(val) => (expectedCapacityMbps = val)}
    />
  </div>

  <!-- Parámetros específicos de UDP si aplica -->
  {#if protocol === "udp"}
    <div class="params-grid udp-params">
      <TextField
        label={t("advanced.udp_target_rate")}
        hint={t("advanced.udp_target_rate_hint")}
        value={udpTargetRateMbps}
        oninput={(val) => (udpTargetRateMbps = val)}
      />

      <TextField
        label={t("advanced.udp_packet_size")}
        hint={t("advanced.udp_packet_size_hint")}
        value={udpPacketSizeBytes.toString()}
        oninput={(val) => {
          const n = parseInt(val, 10);
          if (!isNaN(n)) udpPacketSizeBytes = n;
        }}
      />
    </div>
  {/if}

  <!-- Vista previa y Validación del Plan -->
  <Card>
    <div class="preview-box">
      <div class="preview-header">
        <h4>{t("advanced.preview_title")}</h4>
        <StatusPill tone={validationResult.isValid ? "success" : "danger"}>
          {validationResult.isValid ? "PLAN VÁLIDO" : "PARÁMETROS INVÁLIDOS"}
        </StatusPill>
      </div>

      {#if !validationResult.isValid}
        <div class="error-msg" role="alert">
          <Icon name="x-circle" size={16} />
          <span>{validationResult.error}</span>
        </div>
      {:else}
        <div class="preview-details">
          <p>{t("advanced.preview_ports", { ports: allocatedPortsText })}</p>
          <p>{t("advanced.preview_estimated", { seconds: estimatedTotalSeconds.toString() })}</p>
        </div>
      {/if}

      {#if onApply}
        <div class="apply-action">
          <Button variant="primary" disabled={!validationResult.isValid} onclick={handleApply}>
            Aplicar configuración
          </Button>
        </div>
      {/if}
    </div>
  </Card>
</div>

<style>
  .advanced-plan-container {
    display: flex;
    flex-direction: column;
    gap: var(--nb-space-4, 1rem);
    padding: var(--nb-space-4, 1rem);
    background: var(--color-surface-sunken, rgba(0, 0, 0, 0.2));
    border-radius: var(--radius-card, 8px);
    border: 1px solid var(--color-border-subtle);
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .title {
    font-size: var(--nb-font-size-base, 1rem);
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .subtitle {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-tertiary);
    margin: 0;
  }

  .header-actions {
    display: flex;
    gap: var(--nb-space-2, 0.5rem);
    align-items: center;
  }

  .section-label {
    display: block;
    font-size: var(--nb-font-size-xs, 0.75rem);
    font-weight: 500;
    color: var(--color-text-secondary);
    margin-bottom: var(--nb-space-2, 0.5rem);
  }

  .segmented-control {
    display: inline-flex;
    background: var(--color-surface-recessed, rgba(255, 255, 255, 0.05));
    border-radius: var(--radius-sm, 6px);
    padding: 2px;
    border: 1px solid var(--color-border-subtle);
  }

  .segment-btn {
    padding: var(--nb-space-2, 0.5rem) var(--nb-space-4, 1rem);
    font-size: var(--nb-font-size-sm, 0.875rem);
    border: none;
    border-radius: var(--radius-sm, 6px);
    background: transparent;
    color: var(--color-text-secondary);
    cursor: default;
    transition:
      background 120ms ease,
      color 120ms ease;
  }

  .segment-btn.active {
    background: var(--color-brand, #3b82f6);
    color: #ffffff;
    font-weight: 600;
  }

  .direction-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: var(--nb-space-2, 0.5rem);
  }

  .dir-btn {
    padding: var(--nb-space-2, 0.5rem) var(--nb-space-3, 0.75rem);
    font-size: var(--nb-font-size-xs, 0.75rem);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm, 6px);
    background: var(--color-surface-recessed, rgba(255, 255, 255, 0.05));
    color: var(--color-text-primary);
    cursor: default;
    text-align: center;
    transition:
      border-color 120ms ease,
      background 120ms ease;
  }

  .dir-btn.active {
    border-color: var(--color-brand, #3b82f6);
    background: rgba(59, 130, 246, 0.15);
    font-weight: 600;
  }

  .hint-text {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-tertiary);
    margin-top: var(--nb-space-1, 0.25rem);
  }

  .info-callout {
    display: flex;
    align-items: flex-start;
    gap: var(--nb-space-3, 0.75rem);
    padding: var(--nb-space-3, 0.75rem);
    border-radius: var(--radius-card, 8px);
    background: var(--color-surface-sunken, rgba(255, 255, 255, 0.05));
    border-left: 4px solid var(--color-info, #3b82f6);
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-primary);
  }

  .info-callout p {
    margin: 0;
    line-height: 1.4;
  }

  .params-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--nb-space-3, 0.75rem);
  }

  .udp-params {
    padding-top: var(--nb-space-2, 0.5rem);
    border-top: 1px dashed var(--color-border-subtle);
  }

  .preview-box {
    display: flex;
    flex-direction: column;
    gap: var(--nb-space-2, 0.5rem);
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .preview-header h4 {
    font-size: var(--nb-font-size-sm, 0.875rem);
    margin: 0;
    color: var(--color-text-primary);
  }

  .preview-details {
    font-size: var(--nb-font-size-xs, 0.75rem);
    color: var(--color-text-secondary);
  }

  .preview-details p {
    margin: 2px 0;
  }

  .error-msg {
    display: flex;
    align-items: center;
    gap: var(--nb-space-2, 0.5rem);
    color: var(--color-error, #ef4444);
    font-size: var(--nb-font-size-xs, 0.75rem);
  }

  .apply-action {
    display: flex;
    justify-content: flex-end;
    margin-top: var(--nb-space-2, 0.5rem);
  }
</style>
