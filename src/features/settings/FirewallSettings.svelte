<script lang="ts">
  import { onMount } from "svelte";
  import Card from "../../lib/components/Card.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Dialog from "../../lib/components/Dialog.svelte";
  import StatusPill from "../../lib/components/StatusPill.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import PublicNetworkConfirm from "../firewall/PublicNetworkConfirm.svelte";
  import { t } from "../../lib/i18n";
  import {
    getFirewallRulesStatus,
    createMissingFirewallRules,
    removeFirewallRules,
    openNetworkSettings,
    publicNetworks,
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
  /** Qué se está confirmando: dar el permiso desde el aviso o desde el interruptor. */
  let confirmandoPublico = $state<"notice" | "switch" | null>(null);

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
          ...report.reglas.map((r) => r.comandoAgregar),
          "",
          t("settings.fw.manualRemoveIntro"),
          "",
          "Remove-NetFirewallRule -Group 'NetworkBench'",
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

  /** Redes que Windows considera públicas: con las reglas por defecto no se aplican en ellas. */
  const redesPublicas = $derived(report ? publicNetworks(report) : []);
  const avisarRedPublica = $derived(redesPublicas.length > 0 && report?.permitirPublico === false);

  /** Guarda la decisión y, si se ha aceptado, recrea las reglas con el perfil público (UAC). */
  async function permitirPublico() {
    if (!(await model.update({ firewallAllowPublic: true }))) return;
    // Con el permiso dado, las reglas actuales pasan a estar «desactualizadas»: se relee.
    await cargar();
    await crear();
  }

  async function alternarPublico(valor: boolean) {
    if (await model.update({ firewallAllowPublic: valor })) await cargar();
  }

  async function abrirConfiguracionDeRed() {
    try {
      await openNetworkSettings();
    } catch (e) {
      model.reportError(e);
    }
  }

  function tono(estado: RuleStatus): "success" | "warning" | "danger" {
    return estado === "present" ? "success" : estado === "missing" ? "danger" : "warning";
  }

  function nombreDelPrograma(ruta: string): string {
    return ruta.split(/[\\/]/).pop() ?? ruta;
  }

  /** "Dominio, Privado" o "Dominio, Privado, Público" — para ver de un vistazo si una
   * regla vale (también) en las redes que Windows considera públicas. */
  function etiquetasPerfiles(perfiles: string[]): string {
    const etiqueta = (p: string) =>
      p === "Domain"
        ? t("settings.fw.profile.domain")
        : p === "Private"
          ? t("settings.fw.profile.private")
          : p === "Public"
            ? t("settings.fw.profile.public")
            : p;
    return perfiles.map(etiqueta).join(", ");
  }
</script>

<div class="space-y-6">
  <Card variant="default" enterIndex={0}>
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
                  <span title={regla.programa}>{nombreDelPrograma(regla.programa)}</span> ·
                  {etiquetasPerfiles(regla.perfiles)}
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

  {#if report && avisarRedPublica}
    <Card variant="default" enterIndex={1}>
      <div class="p-5 space-y-3" role="status" data-testid="fw-public-notice">
        <p class="text-sm text-[var(--color-warning)]">
          {t("settings.fw.public.notice", {
            networks: redesPublicas.map((r) => `${r.nombre} (${r.interfaz})`).join(", "),
          })}
        </p>
        <p class="text-xs text-[var(--color-text-muted)]">{t("settings.fw.public.hint")}</p>
        <div class="nb-fw-actions">
          <Button
            variant="secondary"
            onclick={() => (confirmandoPublico = "notice")}
            disabled={ocupado || !report.ayudanteDisponible}
            data-testid="fw-public-allow"
          >
            {t("settings.fw.public.allow")}
          </Button>
          <Button
            variant="ghost"
            onclick={abrirConfiguracionDeRed}
            disabled={ocupado}
            data-testid="fw-public-settings"
          >
            {t("settings.fw.public.openSettings")}
          </Button>
        </div>
      </div>
    </Card>
  {/if}

  {#if report}
    <Card variant="default" enterIndex={2}>
      <div class="p-5 space-y-3">
        <div class="flex items-center justify-between">
          <div class="pr-4">
            <h2 class="text-base font-semibold">{t("settings.fw.public.toggleTitle")}</h2>
            <p class="text-sm text-[var(--color-text-secondary)]">
              {t("settings.fw.public.toggleDesc")}
            </p>
          </div>
          <Switch
            checked={report.permitirPublico}
            onchange={(valor) => (valor ? (confirmandoPublico = "switch") : alternarPublico(false))}
            disabled={ocupado || !model.prefs}
            label={t("settings.fw.public.toggleTitle")}
          />
        </div>
        {#if report.permitirPublico}
          <p class="text-xs text-[var(--color-warning)]">{t("settings.fw.public.toggleWarning")}</p>
        {/if}
      </div>
    </Card>
  {/if}

  {#if confirmandoPublico}
    <PublicNetworkConfirm
      onCancel={() => (confirmandoPublico = null)}
      onConfirm={() => {
        const origen = confirmandoPublico;
        confirmandoPublico = null;
        if (origen === "notice") void permitirPublico();
        else void alternarPublico(true);
      }}
    />
  {/if}

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
