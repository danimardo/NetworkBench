<script lang="ts">
  /**
   * Barra de progreso determinada — pantalla de Ejecución (§16.4: "barra de
   * progreso de la dirección actual con «Tiempo restante aproximado»").
   * También sirve para el progreso agregado de Preparación si se necesita.
   *
   * `indeterminate` cubre los tramos sin porcentaje conocido (p. ej.
   * "Analizando resultados…", donde no hay una duración estimable) — nunca
   * texto de "Cargando…" genérico (§16.14): el texto lo pone quien use el
   * componente (fase real: "Calentando…", etc.), esta barra es solo el
   * indicador visual.
   *
   * C04 (auditoría v3, accesibilidad — "nombre accesible y animación por
   * transform en ProgressBar"):
   *   - `label` da el nombre accesible del propio `role="progressbar"` —
   *     antes no tenía ninguno, así que un lector de pantalla solo
   *     anunciaba el número ("27 %") sin decir de qué. Obligatorio a
   *     propósito, mismo criterio que `adapterType` en DeviceCard (C05):
   *     un progreso sin nombre no sirve de nada por sí solo, tiene que
   *     decidirlo quien lo instancia (p. ej. la fase real: "Calentando…").
   *   - El relleno determinado pasa de animar `width` (dispara layout en
   *     cada frame) a `transform: scaleX()` con `transform-origin: left`
   *     — solo compositor, igual que el resto de animaciones de la app.
   */
  interface Props {
    /** 0–100. Ignorado si `indeterminate`. */
    value?: number;
    indeterminate?: boolean;
    tone?: "accent" | "success" | "warning" | "danger";
    /** Nombre accesible del progreso, p. ej. "Calentando la prueba". */
    label: string;
  }

  let { value = 0, indeterminate = false, tone = "accent", label }: Props = $props();
  let clamped = $derived(Math.max(0, Math.min(100, value)));
</script>

<div
  class="nb-progress nb-progress-{tone}"
  role="progressbar"
  aria-label={label}
  aria-valuenow={indeterminate ? undefined : clamped}
  aria-valuemin="0"
  aria-valuemax="100"
>
  {#if indeterminate}
    <div class="nb-progress-indeterminate"></div>
  {:else}
    <div class="nb-progress-fill" style="transform: scaleX({clamped / 100})"></div>
  {/if}
</div>

<style>
  .nb-progress {
    position: relative;
    width: 100%;
    height: 6px;
    border-radius: var(--radius-pill);
    background: var(--surface-field);
    overflow: hidden;
  }

  .nb-progress-fill {
    /* C04: `width` → `transform: scaleX()`. Con `width` el navegador
       recalcula layout en cada frame de la transición (afecta a lo que
       tenga al lado); `transform` es solo compositor. El ancho es del
       100 % siempre — lo que cambia visualmente es la escala. */
    width: 100%;
    height: 100%;
    border-radius: var(--radius-pill);
    transform-origin: left;
    transition: transform var(--duration-base) ease;
  }

  .nb-progress-accent .nb-progress-fill {
    background: var(--gradient-button-primary);
  }
  .nb-progress-success .nb-progress-fill {
    background: var(--color-success);
  }
  .nb-progress-warning .nb-progress-fill {
    background: var(--color-warning);
  }
  .nb-progress-danger .nb-progress-fill {
    background: var(--color-danger);
  }

  .nb-progress-indeterminate {
    position: absolute;
    inset: 0;
    width: 40%;
    border-radius: var(--radius-pill);
    background: var(--gradient-button-primary);
    animation: nb-progress-slide var(--duration-progress-indeterminate) var(--ease-standard)
      infinite;
  }

  @keyframes nb-progress-slide {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(350%);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-progress-fill {
      transition: none;
    }
    /* nb-progress-fill sigue creciendo (scaleX cambia igual, solo sin
       animación) — nada más que ajustar aquí para el caso determinado. */
    .nb-progress-indeterminate {
      animation: none;
      width: 100%;
      opacity: 0.5;
    }
  }
</style>
