<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "primary" | "secondary" | "ghost";

  interface Props extends HTMLButtonAttributes {
    variant?: Variant;
    disabled?: boolean;
    type?: "button" | "submit";
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
    /** Icono opcional a la izquierda del texto. */
    icon?: Snippet;
  }

  let {
    variant = "secondary",
    disabled = false,
    type = "button",
    onclick,
    children,
    icon,
    ...restProps
  }: Props = $props();
</script>

<button
  class="nb-button nb-button-{variant}"
  {type}
  {disabled}
  onclick={disabled ? undefined : onclick}
  {...restProps}
>
  {#if icon}
    <span class="nb-button-icon">{@render icon()}</span>
  {/if}
  <span class="nb-button-label">{@render children()}</span>
</button>

<style>
  .nb-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-control);
    white-space: nowrap;
    cursor: default; /* controles WinUI usan flecha, no mano (§16.14) */
    border-radius: var(--radius-md);
    padding: 10px var(--space-4);
    transition:
      background var(--duration-base) ease,
      border-color var(--duration-base) ease,
      box-shadow var(--duration-base) ease,
      transform var(--duration-base) ease;
  }

  .nb-button:active:not(:disabled) {
    transform: scale(0.985);
  }

  .nb-button:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  .nb-button:disabled {
    opacity: 0.5;
  }

  /* Primario: gradiente violeta, es el único botón con color intenso. */
  .nb-button-primary {
    color: var(--color-button-primary-text);
    border: 1px solid var(--border-button-primary);
    background: var(--gradient-button-primary);
    box-shadow: var(--shadow-button-primary);
    padding: 15px 26px;
    border-radius: var(--radius-lg);
  }

  .nb-button-primary:hover:not(:disabled) {
    background: var(--gradient-button-primary-hover);
    box-shadow: var(--shadow-button-primary-hover);
    transform: translateY(-1px);
  }

  .nb-button-primary:active:not(:disabled) {
    transform: translateY(0) scale(0.985);
    box-shadow: var(--shadow-button-primary-active);
  }

  /* Secundario: cristal tenue, para acciones de apoyo (Conectar manualmente…). */
  .nb-button-secondary {
    color: var(--color-text-secondary);
    background: var(--surface-secondary);
    border: 1px solid var(--border-default);
    backdrop-filter: blur(var(--blur-card));
    -webkit-backdrop-filter: blur(var(--blur-card));
  }

  .nb-button-secondary:hover:not(:disabled) {
    background: var(--surface-secondary-hover);
    border-color: var(--border-hover);
  }

  /* C01 (auditoría v3): respaldo opaco §16.10 — material `glass-card`
     (ver DESIGN_TOKENS.md § "Los tres materiales…"). Solo el secundario
     usa cristal — el primario ya es un degradado sólido de marca y el
     fantasma no tiene fondo, ninguno de los dos necesita esta regla. */
  @supports not (backdrop-filter: blur(1px)) {
    .nb-button-secondary,
    .nb-button-secondary:hover:not(:disabled) {
      background: var(--surface-solid-card);
    }
  }

  /* Fantasma: texto de acento sin fondo (p. ej. "Repetir con SERVER-01"). */
  .nb-button-ghost {
    color: var(--color-accent);
    background: transparent;
    border: 1px solid transparent;
    padding: 0;
    font-weight: var(--font-weight-subtitle);
  }

  .nb-button-ghost:hover:not(:disabled) {
    color: var(--color-accent-hover);
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-button {
      transition: none;
    }
    .nb-button-primary:hover:not(:disabled),
    .nb-button:active:not(:disabled) {
      transform: none;
    }
  }
</style>
