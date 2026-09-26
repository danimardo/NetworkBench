<script lang="ts">
  import { onMount } from "svelte";
  import Card from "../../lib/components/Card.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Dialog from "../../lib/components/Dialog.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import { t } from "../../lib/i18n";
  import {
    getFirewallRulesStatus,
    createMissingFirewallRules,
    removeFirewallRules,
    type FirewallRulesReport,
    type RuleStatus,
  } from "../../lib/api/firewall";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  let report = $state<FirewallRulesReport | null>(null);
  /** Leyendo el estado del sistema (PowerShell, sin elevación). */
  let cargando = $state(true);
  /** Esperando el UAC de Windows: el helper tarda lo que tarde la persona en responder. */
  let trabajando = $state<"create" | "remove" | null>(null);
  let confirmarEliminar = $state(false);
  let verManual = $state(false);
  let copiado = $state(false);

  async function cargar() {
    cargando = true;
    try {
      report = await getFirewallRulesStatus();
    } catch (e) {
      model.reportError(e);
    } finally {
      cargando = false;
    }
  }

  onMount(() => {
    void cargar();
  });

  /** Las que no están bien y se pueden crear (el helper exige que el programa exista). */
  const porCrear = $derived(
    report?.reglas.filter((r) => r.estado !== "present" && r.programaExiste) ?? [],
  );
  const hayReglas = $derived(report?.reglas.some((r) => r.estado !== "missing") ?? false);
  const todasBien = $derived(report !== null && report.reglas.every((r) => r.estado === "present"));
  const ocupado = $derived(cargando || trabajando !== null);

  async function crear() {
    trabajando = "create";
    const habia = porCrear.length;
    try {
      report = await createMissingFirewallRules();
      model.flashSuccess(habia === 0 ? t("settings.fw.nothingToCreate") : t("settings.fw.created"));
    } catch (e) {
      model.reportError(e);
    } finally {
      trabajando = null;
    }
  }

  async function eliminar() {
    confirmarEliminar = false;
    trabajando = "remove";
    try {
      report = await removeFirewallRules();
      model.flashSuccess(t("settings.fw.removed"));
    } catch (e) {
      model.reportError(e);
    } finally {
      trabajando = null;
    }
  }

  const instrucciones = $derived(
    report
      ? [
          t("settings.fw.manualIntro"),
          "",
          ...report.reglas.map((r) => r.netshAgregar),
          "",
          t("settings.fw.manualRemoveIntro"),
          "",
          ...report.reglas.map((r) => `netsh advfirewall firewall delete rule name="${r.nombre}"`),
        ].join("\n")
      : "",
  );

  async function copiar() {
    try {
      await navigator.clipboard.writeText(instrucciones);
      copiado = true;
      setTimeout(() => (copiado = false), 2000);
    } catch {
      // Sin portapapeles: el texto sigue visible para seleccionarlo a mano.
    }
  }

  function tono(estado: RuleStatus): "success" | "warning" | "danger" {
    return estado === "present" ? "success" : estado === "missing" ? "danger" : "warning";
  }

  function nombreDelPrograma(ruta: string): string {
    return ruta.split(/[\\/]/).pop() ?? ruta;
  }
</script>

<div class="space-y-6">
  <Card variant="default">
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-semibold">{t("settings.firewallRules")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.firewallRulesDesc")}
          </p>
        </div>
        <Button variant="secondary" onclick={cargar} disabled={ocupado}>
          {cargando ? t("common.checking") : t("settings.fw.refresh")}
        </Button>
      </div>

      {#if !report}
        <p class="text-sm text-[var(--color-text-muted)]" role="status">
          {t("settings.fw.checking")}
        </p>
      {:else}
        <ul class="nb-fw-list" aria-label={t("settings.firewallRules")}>
          {#each report.reglas as regla (regla.nombre)}
            <li class="nb-fw-row" data-testid="fw-rule">
              <div class="nb-fw-info">
                <span class="nb-fw-name">{regla.nombre}</span>
                <span class="nb-fw-meta">
                  {regla.protocolo} · {regla.puertos} ·
                  <span title={regla.programa}>{nombreDelPrograma(regla.programa)}</span>
                </span>
                {#if !regla.programaExiste}
                  <span class="nb-fw-detail">
                    {t("settings.fw.missingProgram", {
                      program: nombreDelPrograma(regla.programa),
                    })}
                  </span>
                {:else if regla.estado === "modified"}
                  <span class="nb-fw-detail">{regla.detalle}</span>
                {/if}
              </div>
              <StatusPill tone={tono(regla.estado)}>
                {t(`settings.fw.state.${regla.estado}`)}
              </StatusPill>
            </li>
          {/each}
        </ul>

        {#if todasBien}
          <p class="text-xs text-[var(--color-success)]" role="status">
            {t("settings.fw.allPresent")}
          </p>
        {/if}
        {#if !report.ayudanteDisponible}
          <p class="text-xs text-[var(--color-warning)]" role="status">
            {t("settings.fw.noHelper")}
          </p>
        {:else}
          <p class="text-xs text-[var(--color-text-muted)]">{t("settings.fw.needsUac")}</p>
        {/if}

        <div class="nb-fw-actions">
          <Button
            variant="primary"
            onclick={crear}
            disabled={ocupado || porCrear.length === 0 || !report.ayudanteDisponible}
            data-testid="fw-create"
          >
            {trabajando === "create" ? t("settings.fw.waitingUac") : t("settings.fw.create")}
          </Button>
          <Button
            variant="secondary"
            onclick={() => (confirmarEliminar = true)}
            disabled={ocupado || !hayReglas || !report.ayudanteDisponible}
            data-testid="fw-remove"
          >
            {trabajando === "remove" ? t("settings.fw.waitingUac") : t("settings.fw.remove")}
          </Button>
          <Button variant="ghost" onclick={() => (verManual = true)} data-testid="fw-manual">
            {t("settings.fw.manual")}
          </Button>
        </div>
      {/if}
    </div>
  </Card>

  {#if confirmarEliminar}
    <Dialog
      onClose={() => (confirmarEliminar = false)}
      title={t("settings.fw.removeTitle")}
      size="sm"
    >
      <p class="text-sm text-[var(--color-text-secondary)]">{t("settings.fw.removeWarning")}</p>

      {#snippet actions()}
        <Button variant="secondary" onclick={() => (confirmarEliminar = false)}>
          {t("common.cancel")}
        </Button>
        <Button variant="primary" onclick={eliminar} data-testid="fw-remove-confirm">
          {t("settings.fw.removeConfirm")}
        </Button>
      {/snippet}
    </Dialog>
  {/if}

  {#if verManual}
    <Dialog onClose={() => (verManual = false)} title={t("settings.fw.manualTitle")}>
      <pre class="nb-fw-commands" data-testid="fw-instructions"><code>{instrucciones}</code></pre>

      {#snippet actions()}
        <Button variant="secondary" onclick={copiar}>
          {copiado ? t("errorResolution.copied") : t("errorResolution.copyCommands")}
        </Button>
        <Button variant="primary" onclick={() => (verManual = false)}>{t("common.close")}</Button>
      {/snippet}
    </Dialog>
  {/if}
</div>

<style>
  .nb-fw-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .nb-fw-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
    background: var(--surface-field);
  }

  .nb-fw-row + .nb-fw-row {
    border-top: 1px solid var(--border-subtle);
  }

  .nb-fw-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .nb-fw-name {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-control);
    color: var(--color-text-primary);
  }

  .nb-fw-meta {
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }

  .nb-fw-detail {
    font-size: var(--font-size-xs);
    color: var(--color-warning);
  }

  .nb-fw-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    padding-top: var(--space-2);
  }

  .nb-fw-commands {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    max-height: 320px;
    overflow: auto;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    background: var(--surface-field);
    color: var(--color-text-secondary);
    font-family: var(--font-mono, monospace);
    font-size: var(--font-size-xs);
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
  }
</style>
