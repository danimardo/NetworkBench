<script lang="ts" generics="T extends string">
  interface Option {
    id: T;
    label: string;
  }

  interface Props {
    options: readonly Option[];
    /** Puede ser un valor fuera de las opciones (p. ej. un nivel guardado que la lista no ofrece): entonces no hay ninguna marcada. */
    value: string | undefined;
    onchange: (value: T) => void;
    label: string;
  }

  let { options, value, onchange, label }: Props = $props();

  let els: Record<string, HTMLButtonElement | undefined> = {};

  // Sin ninguna opción marcada (valor ausente o fuera de la lista), la primera hace de punto
  // de entrada: si no, el grupo entero quedaría fuera del orden de Tab.
  const hayMarcada = $derived(options.some((o) => o.id === value));

  /**
   * Grupo de radios (WAI-ARIA APG): las flechas mueven **y** seleccionan, y solo la opción
   * marcada está en el orden de Tab. Sin nada seleccionado, la primera hace de punto de entrada.
   */
  function handleKeydown(event: KeyboardEvent) {
    const actual = options.findIndex((o) => o.id === value);
    let siguiente = actual;
    if (event.key === "ArrowRight" || event.key === "ArrowDown")
      siguiente = (actual + 1) % options.length;
    else if (event.key === "ArrowLeft" || event.key === "ArrowUp")
      siguiente = (actual - 1 + options.length) % options.length;
    else return;

    event.preventDefault();
    const destino = options[siguiente];
    if (!destino) return;
    onchange(destino.id);
    els[destino.id]?.focus();
  }
</script>

<div
  class="nb-segmented-group"
  role="radiogroup"
  aria-label={label}
  tabindex="-1"
  onkeydown={handleKeydown}
>
  {#each options as opt, i (opt.id)}
    {@const seleccionada = opt.id === value}
    <button
      bind:this={els[opt.id]}
      type="button"
      role="radio"
      aria-checked={seleccionada}
      tabindex={seleccionada || (!hayMarcada && i === 0) ? 0 : -1}
      class="nb-segmented-item"
      class:nb-segmented-item-active={seleccionada}
      onclick={() => onchange(opt.id)}
    >
      {opt.label}
    </button>
  {/each}
</div>

<style>
  .nb-segmented-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--surface-secondary);
    border: 1px solid var(--border-default);
  }

  .nb-segmented-item {
    padding: 7px 14px;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-control);
    color: var(--color-text-secondary);
    cursor: default;
    background: transparent;
    border: none;
    white-space: nowrap;
    transition:
      background var(--duration-base) ease,
      color var(--duration-base) ease;
  }

  .nb-segmented-item:hover:not(.nb-segmented-item-active) {
    background: var(--hover-tint);
  }

  /* Misma marca de «seleccionado» que la barra lateral: así se distingue de un vistazo. */
  .nb-segmented-item-active {
    background: var(--gradient-nav-active);
    color: var(--color-nav-active-text);
    box-shadow: var(--shadow-nav-active);
  }

  .nb-segmented-item:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-segmented-item {
      transition: none;
    }
  }
</style>
