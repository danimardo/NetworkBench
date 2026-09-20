<script lang="ts">
  import type { Snippet } from "svelte";

  type Variant = "default" | "hero" | "selected";

  interface Props {
    variant?: Variant;
    /** Si se da, la card entera es un <button> (tarjetas de equipo clicables). */
    onclick?: (e: MouseEvent) => void;
    disabled?: boolean;
    children: Snippet;
  }

  let { variant = "default", onclick, disabled = false, children }: Props = $props();

  let isInteractive = $derived(!!onclick);

  // Una card clicable (DeviceCard) puede llevar dentro OTROS controles
  // interactivos propios (el botón de favorito, el escudo con tooltip) —
  // por eso NO puede ser un <button> real: <button> dentro de <button> es
  // HTML inválido y el navegador lo "arregla" cerrando el de fuera en
  // cuanto encuentra el de dentro, lo que descuadra el layout entero (bug
  // real, visto en la maqueta navegable al añadir el botón de favorito
  // dentro de la tarjeta — corregido aquí para que no vuelva a pasar). En
  // su lugar: un <div role="button" tabindex="0"> con el mismo
  // comportamiento de teclado que un botón (Enter/Espacio activan).
  function handleKeydown(e: KeyboardEvent) {
    if (disabled) return;
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onclick?.(e as unknown as MouseEvent);
    }
  }
</script>

{#if isInteractive}
  <div
    class="nb-card nb-card-{variant}"
    class:nb-card-disabled={disabled}
    role="button"
    tabindex={disabled ? -1 : 0}
    aria-disabled={disabled || undefined}
    onclick={disabled ? undefined : onclick}
    onkeydown={handleKeydown}
  >
    {@render children()}
  </div>
{:else}
  <!-- `disabled` también se respeta sin onclick: una tarjeta de solo lectura
       (p. ej. un equipo incompatible listado, no clicable) debe atenuarse
       igual que una interactiva deshabilitada. -->
  <div class="nb-card nb-card-{variant}" class:nb-card-disabled={disabled} aria-disabled={disabled || undefined}>
    {@render children()}
  </div>
{/if}

<style>
  .nb-card {
    display: flex;
    flex-direction: column;
    text-align: left;
    box-sizing: border-box;
    font-family: var(--font-ui);
    color: var(--color-text-primary);
    cursor: default;
    transition:
      background-color var(--duration-base) ease,
      border-color var(--duration-base) ease,
      transform var(--duration-base) ease,
      box-shadow var(--duration-slow) ease;
  }

  .nb-card-disabled {
    opacity: 0.68;
  }

  .nb-card:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  /* Card normal (equipo disponible, chip de favorito…) */
  .nb-card-default {
    gap: var(--space-3);
    padding: var(--space-4);
    border-radius: var(--radius-lg);
    background: var(--gradient-card-default);
    border: 1px solid var(--border-default);
    backdrop-filter: blur(var(--blur-card));
    -webkit-backdrop-filter: blur(var(--blur-card));
    box-shadow: var(--shadow-card-default);
  }

  [role="button"].nb-card-default:hover:not(.nb-card-disabled) {
    background: var(--gradient-card-default-hover);
    border-color: var(--border-hover);
    transform: translateY(-1px);
  }

  /* Card seleccionada/de confianza — un único nivel de borde+glow, sin abusar. */
  .nb-card-selected {
    gap: var(--space-3);
    padding: var(--space-4);
    border-radius: var(--radius-lg);
    background: var(--gradient-card-selected);
    border: 1px solid var(--border-accent-strong);
    box-shadow: var(--shadow-card-selected);
  }

  /* Card "hero" — el panel grande de la acción principal. */
  .nb-card-hero {
    gap: var(--space-4);
    padding: 26px 30px;
    border-radius: var(--radius-xl);
    background: var(--gradient-card-hero);
    border: 1px solid var(--border-card-hero);
    /* C03 (auditoría v3): era `saturate(125%)` literal, sin depender del
       tema — pero DESIGN_TOKENS.md § "Los tres materiales…" ya documentaba
       que `glass-card` (default Y hero) usa `--material-saturate` como
       cualquier otro material, no un valor propio. Corregido para que la
       implementación coincida con lo ya documentado, no al revés. */
    backdrop-filter: blur(var(--blur-hero)) saturate(var(--material-saturate));
    -webkit-backdrop-filter: blur(var(--blur-hero)) saturate(var(--material-saturate));
    box-shadow: var(--shadow-card-hero);
  }

  /* C01 (auditoría v3): respaldo opaco §16.10 — ya lo tenían Dialog/Toast/
     Tooltip, faltaba en Card. Default y hero comparten el mismo material
     `glass-card` (ver DESIGN_TOKENS.md § "Los tres materiales…"), así que
     los dos usan el mismo token de respaldo. Tiene que ir DESPUÉS de las
     reglas base de `.nb-card-default`/`.nb-card-hero` en el propio
     fichero — misma especificidad, gana la última declarada — y el hover
     pierde el degradado a propósito: sin blur real, mezclar dos fondos
     translúcidos encima de un color sólido ya no da el mismo resultado,
     así que se queda en el color base tanto en reposo como en hover. */
  @supports not (backdrop-filter: blur(1px)) {
    .nb-card-default,
    [role="button"].nb-card-default:hover:not(.nb-card-disabled) {
      background: var(--surface-solid-card);
    }
    .nb-card-hero {
      background: var(--surface-solid-card);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-card {
      transition: none;
    }
    [role="button"].nb-card-default:hover:not(.nb-card-disabled) {
      transform: none;
    }
  }
</style>
