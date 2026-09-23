<script lang="ts">
  /**
   * Tooltip propio (§15 de la biblioteca de componentes del correo de
   * ampliación; también resuelve el punto 21.5 "sustituir las dependencias
   * de tooltips nativos por el patrón diseñado"). Hasta esta entrega varios
   * sitios usaban `title="..."` del navegador — funciona, pero es el
   * tooltip del SISTEMA (con su propio retraso, tipografía y posición, sin
   * relación visual con la app) en un producto cuya regla nº1 de §16.14 es
   * "no debe parecer una página web". Este componente lo sustituye:
   * aparece con el mismo material `glass-overlay` que el resto de overlays,
   * respeta reposo/hover/foco y `prefers-reduced-motion`.
   *
   * Uso:
   *   <Tooltip text="Prueba en curso">
   *     {#snippet children(tooltipId)}
   *       <button aria-disabled="true" aria-describedby={tooltipId}>...</button>
   *     {/snippet}
   *   </Tooltip>
   *
   * Accesible: el elemento envuelto necesita su propio `aria-label` o texto
   * visible — este tooltip es un refuerzo visual para ratón/foco de
   * teclado, NUNCA la única forma de que un lector de pantalla sepa qué es
   * el control (§16.11). Con teclado aparece al recibir el foco, igual que
   * con `:hover`, así que información equivalente llega sin ratón.
   *
   * C04 (auditoría v3, accesibilidad — "aria-describedby y cierre con
   * Escape en Tooltip, alcanzable por teclado"):
   *   - `children` ahora es un snippet PARAMETRIZADO: recibe el `id` único
   *     de la propia burbuja (`$props.id()`, estable e igual en servidor y
   *     cliente) para que quien instancia el Tooltip lo ponga como
   *     `aria-describedby` en SU elemento — este componente no controla qué
   *     renderiza `children()`, así que no puede añadir el atributo por su
   *     cuenta. Ver Sidebar.svelte y DeviceCard.svelte para el patrón.
   *   - La visibilidad dejó de ser pura CSS (`:hover`/`:focus-within`) y
   *     pasó a un `$state` explícito, porque el patrón WAI-ARIA de tooltip
   *     pide que Escape OCULTE la burbuja sin mover el foco fuera del
   *     control envuelto (a diferencia de un diálogo, donde Escape sí
   *     cierra y devuelve el foco a otro sitio) — con CSS puro no hay forma
   *     de expresar "oculto por Escape pero el foco sigue aquí", porque
   *     `:focus-within` seguiría siendo cierto.
   */
  import type { Snippet } from "svelte";

  interface Props {
    text: string;
    /** Lado por el que se abre respecto al elemento envuelto. */
    placement?: "top" | "bottom" | "left" | "right";
    /** Recibe el id de la burbuja para enlazar `aria-describedby`. */
    children: Snippet<[string]>;
  }

  let { text, placement = "top", children }: Props = $props();

  let visible = $state(false);
  const tooltipId = $props.id();

  function show() {
    visible = true;
  }
  function hide() {
    visible = false;
  }
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && visible) {
      // No además cerrar un diálogo/overlay padre con el mismo Escape: éste
      // solo oculta la burbuja, el foco se queda donde estaba.
      e.stopPropagation();
      visible = false;
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<span
  class="nb-tooltip-wrap nb-tooltip-{placement}"
  role="group"
  onmouseenter={show}
  onmouseleave={hide}
  onfocusin={show}
  onfocusout={hide}
  onkeydown={handleKeydown}
>
  {@render children(tooltipId)}
  <span id={tooltipId} class="nb-tooltip-bubble" class:is-visible={visible} role="tooltip"
    >{text}</span
  >
</span>

<style>
  .nb-tooltip-wrap {
    position: relative;
    display: inline-flex;
  }

  .nb-tooltip-bubble {
    position: absolute;
    z-index: 40;
    padding: 6px 10px;
    border-radius: var(--radius-xs);
    background: var(--gradient-dialog);
    border: 1px solid var(--border-dialog);
    box-shadow: var(--shadow-md);
    backdrop-filter: blur(var(--blur-card)) saturate(var(--material-saturate));
    -webkit-backdrop-filter: blur(var(--blur-card)) saturate(var(--material-saturate));
    color: var(--color-text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-2xs);
    font-weight: var(--font-weight-control);
    line-height: 1.3;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transform: translateY(2px) scale(0.98);
    transition:
      opacity var(--duration-fast) ease,
      transform var(--duration-fast) ease;
    transition-delay: 0s;
  }

  @supports not (backdrop-filter: blur(1px)) {
    .nb-tooltip-bubble {
      background: var(--surface-solid-overlay);
    }
  }

  /* C04: visibilidad ahora la decide `visible` en JS ($state), no
     `:hover`/`:focus-within` — ver comentario del <script> sobre por qué
     (Escape necesita poder ocultar sin que `:focus-within` lo contradiga). */
  .nb-tooltip-bubble.is-visible {
    opacity: 1;
    transform: translateY(0) scale(1);
    transition-delay: var(
      --duration-tooltip-delay
    ); /* retraso corto, como cualquier tooltip WinUI */
  }

  .nb-tooltip-top .nb-tooltip-bubble {
    bottom: calc(100% + 7px);
    left: 50%;
    transform: translate(-50%, 2px) scale(0.98);
  }
  .nb-tooltip-top .nb-tooltip-bubble.is-visible {
    transform: translate(-50%, 0) scale(1);
  }

  .nb-tooltip-bottom .nb-tooltip-bubble {
    top: calc(100% + 7px);
    left: 50%;
    transform: translate(-50%, -2px) scale(0.98);
  }
  .nb-tooltip-bottom .nb-tooltip-bubble.is-visible {
    transform: translate(-50%, 0) scale(1);
  }

  .nb-tooltip-left .nb-tooltip-bubble {
    right: calc(100% + 7px);
    top: 50%;
    transform: translate(2px, -50%) scale(0.98);
  }
  .nb-tooltip-left .nb-tooltip-bubble.is-visible {
    transform: translate(0, -50%) scale(1);
  }

  .nb-tooltip-right .nb-tooltip-bubble {
    left: calc(100% + 7px);
    top: 50%;
    transform: translate(-2px, -50%) scale(0.98);
  }
  .nb-tooltip-right .nb-tooltip-bubble.is-visible {
    transform: translate(0, -50%) scale(1);
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-tooltip-bubble {
      transition: opacity var(--duration-fast) ease;
      transition-delay: 0s;
      transform: none !important;
    }
  }
</style>
