<script lang="ts">
  import Icon from "./Icon.svelte";

  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    options: readonly Option[];
    value: string;
    onchange: (value: string) => void;
    /** Nombre accesible: el desplegable no lleva etiqueta visible propia. */
    label: string;
    id?: string;
  }

  let { options, value, onchange, label, id }: Props = $props();
</script>

<!--
  <select> nativo (teclado y lector de pantalla gratis) con el aspecto de `TextField`: misma
  altura (40 px), mismo fondo y mismo borde. La lista emergente la dibuja WebView2; se le da
  fondo opaco y colores del tema a cada <option>, porque si no toma el fondo translúcido del
  campo sobre blanco y sale gris.
-->
<span class="nb-select">
  <select
    {id}
    aria-label={label}
    {value}
    onchange={(e) => onchange((e.currentTarget as HTMLSelectElement).value)}
  >
    {#each options as opt (opt.value)}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
  <span class="nb-select-chevron" aria-hidden="true"><Icon name="chevron-down" size={14} /></span>
</span>

<style>
  .nb-select {
    position: relative;
    display: inline-flex;
    align-items: center;
    height: 40px;
  }

  select {
    appearance: none;
    -webkit-appearance: none;
    height: 100%;
    padding: 0 calc(var(--space-3) + 22px) 0 var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--surface-field);
    border: 1px solid var(--border-field);
    color: var(--color-text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    cursor: default;
    transition:
      background var(--duration-fast) ease,
      border-color var(--duration-fast) ease;
  }

  select:hover {
    background: var(--surface-field-hover);
    border-color: var(--border-field-hover);
  }

  select:focus-visible {
    outline: none;
    border-color: var(--border-field-focus);
  }

  /* Opaco: la lista emergente no puede heredar la transparencia del campo. */
  select option {
    background-color: var(--color-bg-elevated);
    color: var(--color-text-primary);
  }

  .nb-select-chevron {
    position: absolute;
    right: var(--space-3);
    display: flex;
    color: var(--color-text-muted);
    pointer-events: none;
  }
</style>
