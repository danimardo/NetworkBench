<script lang="ts">
  import Card from "../../lib/components/Card.svelte";
  import Button from "../../lib/components/Button.svelte";
  import Segmented from "../../lib/components/Segmented.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import TextField from "../../lib/components/TextField.svelte";
  import { t } from "../../lib/i18n";
  import type { SettingsModel } from "./model.svelte";

  interface Props {
    model: SettingsModel;
  }

  let { model }: Props = $props();

  type LogLevel = "warn" | "info" | "debug";

  // Campos locales: el guardado es al salir del campo (como el puerto de control en
  // NetworkSettings), no en cada pulsación. Se sincronizan cuando llegan los ajustes por
  // primera vez, no en cada cambio posterior (para no pisar lo que la persona está
  // escribiendo si `model.prefs` se refresca por otra vía).
  let ooUrl = $state("");
  let ooOrg = $state("");
  let ooStream = $state("");
  let ooToken = $state("");
  let sincronizado = false;

  $effect(() => {
    if (model.prefs && !sincronizado) {
      ooUrl = model.prefs.openObserveUrl;
      ooOrg = model.prefs.openObserveOrg;
      ooStream = model.prefs.openObserveStream;
      ooToken = model.prefs.openObserveToken;
      sincronizado = true;
    }
  });

  function handleOpenObserveToggle(enabled: boolean) {
    void model.update({ openObserveEnabled: enabled });
  }

  function guardarCamposOpenObserve() {
    if (!model.prefs) return;
    if (
      ooUrl === model.prefs.openObserveUrl &&
      ooOrg === model.prefs.openObserveOrg &&
      ooStream === model.prefs.openObserveStream &&
      ooToken === model.prefs.openObserveToken
    ) {
      return;
    }
    void model.update({
      openObserveUrl: ooUrl,
      openObserveOrg: ooOrg,
      openObserveStream: ooStream,
      openObserveToken: ooToken,
    });
  }

  function probarOpenObserve() {
    void model.testOpenObserve({ url: ooUrl, org: ooOrg, stream: ooStream, token: ooToken });
  }

  // «Warn», «Info» y «Debug» son nombres de nivel, no texto de interfaz; solo el
  // «recomendado» se traduce.
  const logLevels = $derived<{ id: LogLevel; label: string }[]>([
    { id: "warn", label: `Warn (${t("settings.recommended")})` },
    { id: "info", label: "Info" },
    { id: "debug", label: "Debug" },
  ]);

  function handleLogLevelChange(level: LogLevel) {
    void model.update({ logLevel: level });
  }
</script>

<div class="space-y-6">
  <!-- Nivel de Registro Temporal -->
  <Card variant="default" enterIndex={0}>
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.logLevel")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.logLevelDesc")}
          </p>
        </div>

        <Segmented
          options={logLevels}
          value={model.prefs?.logLevel}
          onchange={handleLogLevelChange}
          label={t("settings.logLevel")}
        />
      </div>

      {#if model.prefs?.logLevel === "debug"}
        <div class="rounded bg-[var(--surface-field)] p-3 text-xs text-[var(--color-warning)]">
          {t("settings.debugLevelWarning")}
        </div>
      {/if}
    </div>
  </Card>

  <!-- Actualizaciones del Sistema -->
  <Card variant="default" enterIndex={1}>
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.updates")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.updatesDesc")}
          </p>
        </div>

        <Button
          variant="secondary"
          onclick={() => model.checkForUpdates()}
          disabled={model.isCheckingUpdate}
        >
          {model.isCheckingUpdate ? t("common.checking") : t("settings.checkUpdatesBtn")}
        </Button>
      </div>

      {#if model.updateStatus}
        <div class="rounded bg-[var(--surface-field)] p-3 text-xs">
          {#if model.updateStatus.status === "upToDate"}
            <span class="text-[var(--color-success)]">{t("settings.upToDateMessage")}</span>
          {:else if model.updateStatus.status === "updateAvailable"}
            <div class="space-y-1">
              <span class="font-medium text-[var(--color-text-primary)]">
                {t("settings.updateAvailableTitle")}: {model.updateStatus.version}
              </span>
              <p class="text-[var(--color-text-secondary)]">{model.updateStatus.notes}</p>
            </div>
          {:else if model.updateStatus.status === "deferredDueToActiveSession"}
            <span class="text-[var(--color-warning)]">
              {t("settings.updateDeferredMessage")}
            </span>
          {/if}
        </div>
      {/if}
    </div>
  </Card>

  <!-- Registros: dónde escribe la aplicación de verdad, no dónde alguien podría buscarlos. -->
  <Card variant="default" enterIndex={2}>
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.logs")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">{t("settings.logsDesc")}</p>
        </div>
        <Button
          variant="secondary"
          onclick={() => model.revealLogFolder()}
          disabled={!model.diagnosticPaths}
        >
          {t("settings.openLogFolder")}
        </Button>
      </div>

      {#if model.diagnosticPaths}
        <div class="rounded bg-[var(--surface-field)] p-3 text-xs space-y-2">
          <div>
            <span class="text-[var(--color-text-muted)] block">{t("settings.appLogDir")}:</span>
            <span class="font-mono break-all" data-testid="app-log-dir"
              >{model.diagnosticPaths.appLogDir}</span
            >
          </div>
          <div>
            <span class="text-[var(--color-text-muted)] block"
              >{t("settings.firewallHelperLogPath")}:</span
            >
            <span class="font-mono break-all" data-testid="firewall-helper-log-path"
              >{model.diagnosticPaths.firewallHelperLogPath}</span
            >
          </div>
        </div>
      {/if}
    </div>
  </Card>

  <!-- OpenObserve: reenvío opt-in de diagnóstico a un servidor propio (constitución,
       enmienda 0.8.0). Apagado por defecto; los cuatro campos solo importan encendido. -->
  <Card variant="default" enterIndex={3}>
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between">
        <div class="pr-4">
          <h2 class="text-base font-semibold">{t("settings.openObserve.title")}</h2>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {t("settings.openObserve.desc")}
          </p>
        </div>
        <Switch
          checked={model.prefs?.openObserveEnabled ?? false}
          onchange={handleOpenObserveToggle}
          disabled={!model.prefs}
          label={t("settings.openObserve.title")}
        />
      </div>

      {#if model.prefs?.openObserveEnabled}
        <div class="grid gap-4 sm:grid-cols-2">
          <TextField
            label={t("settings.openObserve.url")}
            bind:value={ooUrl}
            placeholder="https://mi-openobserve.example"
            onblur={guardarCamposOpenObserve}
            id="oo-url"
          />
          <TextField
            label={t("settings.openObserve.org")}
            bind:value={ooOrg}
            onblur={guardarCamposOpenObserve}
            id="oo-org"
          />
          <TextField
            label={t("settings.openObserve.stream")}
            bind:value={ooStream}
            onblur={guardarCamposOpenObserve}
            id="oo-stream"
          />
          <TextField
            label={t("settings.openObserve.token")}
            type="password"
            bind:value={ooToken}
            onblur={guardarCamposOpenObserve}
            id="oo-token"
          />
        </div>

        <div class="flex items-center gap-3">
          <Button
            variant="secondary"
            onclick={probarOpenObserve}
            disabled={model.isTestingOpenObserve || !ooUrl || !ooOrg || !ooStream || !ooToken}
          >
            {model.isTestingOpenObserve ? t("common.checking") : t("settings.openObserve.testBtn")}
          </Button>
        </div>

        {#if model.openObserveTestResult}
          <div
            class="rounded bg-[var(--surface-field)] p-3 text-xs"
            class:text-[var(--color-success)]={model.openObserveTestResult.ok}
            class:text-[var(--color-danger)]={!model.openObserveTestResult.ok}
            role="status"
          >
            {model.openObserveTestResult.message}
          </div>
        {/if}
      {/if}
    </div>
  </Card>
</div>
