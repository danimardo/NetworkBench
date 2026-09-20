<script lang="ts">
  /**
   * Control de Ajustes → General → "Tema": Sistema / Claro / Oscuro (§23).
   * Es la única pieza de UI que toca `theme.setMode(...)` directamente —
   * cualquier otra pantalla que necesite saber el tema actual debe leer
   * `theme.resolved`, nunca duplicar este control.
   */
  import { theme, type ThemeMode } from "../design-system/theme.svelte";

  const OPTIONS: { id: ThemeMode; label: string }[] = [
    { id: "system", label: "Sistema" },
    { id: "light", label: "Claro" },
    { id: "dark", label: "Oscuro" },
  ];
</script>

<div class="nb-theme-toggle" role="radiogroup" aria-label="Tema">
  {#each OPTIONS as option (option.id)}
    <button
      type="button"
      role="radio"
      aria-checked={theme.mode === option.id}
      class="nb-theme-option"
      class:nb-theme-option-active={theme.mode === option.id}
      onclick={() => theme.setMode(option.id)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .nb-theme-toggle {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--surface-secondary);
    border: 1px solid var(--border-default);
  }

  .nb-theme-option {
    padding: 7px 14px;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-control);
    color: var(--color-text-secondary);
    cursor: default;
    transition: background var(--duration-base) ease, color var(--duration-base) ease;
  }

  .nb-theme-option:hover:not(.nb-theme-option-active) {
    background: var(--hover-tint);
  }

  .nb-theme-option:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  .nb-theme-option-active {
    background: var(--gradient-nav-active);
    color: var(--color-nav-active-text);
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-theme-option {
      transition: none;
    }
  }
</style>
