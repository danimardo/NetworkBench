<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import Button from "../../lib/components/Button.svelte";
  import {
    getFirewallRulesStatus,
    createMissingFirewallRules,
    NOMBRE_REGLA_CONTROL,
    type FirewallRulesReport,
    type RuleStatus,
  } from "../../lib/api/firewall";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  let portInput = $state<string>("");
  let portError = $state<string | null>(null);
  /** La regla de firewall del puerto no vale tras cambiarlo: se ofrece recrearla. */
  let avisoRegla = $state<{ puerto: number; estado: RuleStatus } | null>(null);
  let recreando = $state(false);

  function revisarRegla(informe: FirewallRulesReport) {
    const control = informe.reglas.find((r) => r.nombre === NOMBRE_REGLA_CONTROL);
    avisoRegla =
      control && control.estado !== "present"
        ? { puerto: informe.puertoControl, estado: control.estado }
        : null;
  }

  async function comprobarRegla() {
    try {
      revisarRegla(await getFirewallRulesStatus());
    } catch {
      // Sin poder leer el cortafuegos no se afirma nada: no hay aviso.
      avisoRegla = null;
    }
  }

  async function recrearRegla() {
    recreando = true;
    try {
      const informe = await createMissingFirewallRules();
      revisarRegla(informe);
      model.flashSuccess(t("settings.fw.created"));
    } catch (e) {
      model.reportError(e);
    } finally {
      recreando = false;
    }
  }

  $effect(() => {
    if (model.prefs?.customControlPort) {
      portInput = String(model.prefs.customControlPort);
    }
  });

  async function handlePortBlur() {
    portError = null;
    const anterior = model.prefs?.customControlPort ?? null;
    const trimmed = String(portInput ?? "").trim();

    let nuevo: number | null = null;
    if (trimmed) {
      const val = parseInt(trimmed, 10);
      if (isNaN(val) || val < 1024 || val > 65000) {
        portError = t("settings.portRangeError");
        return;
      }
      nuevo = val;
    }

    // Salir del campo sin cambiar nada no es un cambio: no se guarda ni se avisa.
    if (nuevo === anterior) return;

    const guardado = await model.update({ customControlPort: nuevo });
    if (!guardado) {
      // Puerto ocupado u otro fallo: el backend conserva el anterior y el campo también.
      portInput = anterior === null ? "" : String(anterior);
      return;
    }
    // El puerto ya está aplicado (el backend reabre el canal); falta saber si el
    // cortafuegos lo permite.
    await comprobarRegla();
  }

  function handleMdnsToggle(value: boolean) {
    void model.update({ mdnsEnabled: value });
  }
</script>

<div class="space-y-6">
  <!-- Puerto de Control Personalizado -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div>
        <h2 class="text-base font-semibold">{t("settings.networkControlPort")}</h2>
        <p class="text-sm text-[var(--color-text-secondary)]">
          {t("settings.networkControlPortDesc")}
        </p>
      </div>

      <div class="flex items-center gap-3">
        <input
          type="number"
          min="1024"
          max="65000"
          placeholder="7411"
          class="nb-input"
          bind:value={portInput}
          onblur={handlePortBlur}
          aria-label={t("settings.networkControlPort")}
        />
        <span class="text-xs text-[var(--color-text-muted)]">
          {t("settings.defaultPortNotice")}
        </span>
      </div>
      {#if portError}
        <p class="text-xs text-[var(--color-danger)]" role="alert">{portError}</p>
      {/if}
    </div>
  </Card>

  {#if avisoRegla}
    <Card variant="default">
      <div
        class="p-5 flex items-center justify-between gap-4"
        role="status"
        data-testid="fw-port-notice"
      >
        <p class="text-sm text-[var(--color-warning)]">
          {t(
            avisoRegla.estado === "missing" ? "settings.fw.portNoRule" : "settings.fw.portOutdated",
            {
              port: avisoRegla.puerto,
            },
          )}
        </p>
        <Button
          variant="primary"
          onclick={recrearRegla}
          disabled={recreando}
          data-testid="fw-recreate"
        >
          {recreando ? t("settings.fw.waitingUac") : t("settings.fw.recreate")}
        </Button>
      </div>
    </Card>
  {/if}

  <!-- Descubrimiento de Red Local mDNS -->
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.mdnsDiscovery")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.mdnsDiscoveryDesc")}
          </p>
        </div>
        <Switch
          checked={model.prefs?.mdnsEnabled ?? true}
          onchange={handleMdnsToggle}
          disabled={!model.prefs}
          label={t("settings.mdnsDiscovery")}
        />
      </div>
    </div>
  </Card>
</div>

<style>
  /* La clase `nb-input` no existía: el campo se pintaba con el aspecto nativo del navegador. */
  .nb-input {
    width: 10rem;
    padding: 9px var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-default);
    background: var(--surface-field);
    color: var(--color-text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    font-variant-numeric: tabular-nums;
  }

  .nb-input:hover {
    border-color: var(--border-hover);
  }

  .nb-input:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }
</style>
