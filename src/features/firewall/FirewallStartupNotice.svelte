<script lang="ts">
  import Button from "../../lib/components/Button.svelte";
  import Toast from "../../lib/components/Toast.svelte";
  import ToastLayer from "../../lib/components/ToastLayer.svelte";
  import PublicNetworkConfirm from "./PublicNetworkConfirm.svelte";
  import { t } from "../../lib/i18n";
  import type { FirewallStartupCheck } from "./startup.svelte";

  interface Props {
    check: FirewallStartupCheck;
    /** Lleva a Ajustes → Cortafuegos: solo cuando no se puede crear desde aquí (sin ayudante). */
    onReview: () => void;
  }

  let { check, onReview }: Props = $props();

  let confirmandoPublico = $state(false);

  const redes = $derived(check.redesPublicas.map((r) => `${r.nombre} (${r.interfaz})`).join(", "));
  /** Las dos acciones de la red pública (única rama con más de un botón): se centran como
   * grupo en el ancho de la tarjeta. Con un solo botón se deja alineado a la izquierda, con
   * el texto de arriba, como estaba. */
  const dosAcciones = $derived(check.puedeCrear && check.motivo !== "faltan");
</script>

<!--
  Aviso persistente, abajo a la derecha: flota sobre el contenido (no desplaza nada) y no pisa
  los avisos transitorios de Ajustes, que van al centro. Se cierra con la X («Ahora no»).
-->
{#if check.exito}
  <ToastLayer>
    <Toast tone="success">{check.exito}</Toast>
  </ToastLayer>
{/if}

{#if check.visible}
  <ToastLayer align="end">
    <Toast tone="warning" wide onDismiss={() => check.descartar()}>
      <div class="nb-fwnotice" data-testid="fw-startup-notice">
        <p class="nb-fwnotice-text">
          {check.motivo === "faltan"
            ? t("startup.fw.missing")
            : t("startup.fw.public", { networks: redes })}
        </p>
        {#if check.error}
          <p class="nb-fwnotice-error" role="alert">{check.error}</p>
        {/if}
        <div class="nb-fwnotice-actions" class:nb-fwnotice-actions-center={dosAcciones}>
          {#if !check.puedeCrear}
            <!-- Sin el ayudante elevado no se puede crear desde aquí: Ajustes tiene las
                 instrucciones manuales. -->
            <Button variant="primary" onclick={onReview} data-testid="fw-startup-review">
              {t("startup.fw.review")}
            </Button>
          {:else if check.motivo === "faltan"}
            <Button
              variant="primary"
              onclick={() => check.crearAhora()}
              disabled={check.creando}
              data-testid="fw-startup-create"
            >
              {check.creando ? t("startup.fw.creating") : t("startup.fw.create")}
            </Button>
          {:else}
            <Button
              variant="primary"
              onclick={() => (confirmandoPublico = true)}
              disabled={check.creando}
              data-testid="fw-startup-public-allow"
            >
              {check.creando ? t("startup.fw.creating") : t("settings.fw.public.allow")}
            </Button>
            <Button
              variant="secondary"
              onclick={() => check.abrirConfiguracionDeRed()}
              disabled={check.creando}
              data-testid="fw-startup-public-settings"
            >
              {t("settings.fw.public.openSettings")}
            </Button>
          {/if}
        </div>
      </div>
    </Toast>
  </ToastLayer>
{/if}

{#if confirmandoPublico}
  <PublicNetworkConfirm
    onCancel={() => (confirmandoPublico = false)}
    onConfirm={() => {
      confirmandoPublico = false;
      void check.permitirPublico();
    }}
  />
{/if}

<style>
  .nb-fwnotice {
    display: flex;
    flex-direction: column;
    /* Cada hijo a su tamaño: si no, el botón se estira a todo el ancho de la columna. */
    align-items: flex-start;
    gap: var(--space-3);
  }

  .nb-fwnotice-text,
  .nb-fwnotice-error {
    margin: 0;
  }

  .nb-fwnotice-error {
    color: var(--color-danger);
    font-size: var(--font-size-xs);
  }

  .nb-fwnotice-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    max-width: 100%;
  }

  /* Solo la rama de dos botones (redes públicas) se centra como grupo en el ancho de la
     tarjeta (480px con `wide`, ver Toast.svelte). Requiere que el propio contenedor ocupe
     el ancho disponible: si no, `justify-content` no tendría sitio de sobra que repartir. */
  .nb-fwnotice-actions-center {
    align-self: stretch;
    justify-content: center;
  }

  /* El texto de espera del UAC puede ser largo: se parte en lugar de desbordar la tarjeta. */
  .nb-fwnotice-actions :global(.nb-button) {
    white-space: normal;
    text-align: center;
  }
</style>
