<script lang="ts">
  import { t } from "../i18n";

  interface Props {
    checked: boolean;
    onchange: (checked: boolean) => void;
    /** Nombre accesible del ajuste; el estado lo anuncia `role="switch"` + `aria-checked`. */
    label: string;
    disabled?: boolean;
    /** Muestra «Activado/Desactivado» junto al interruptor, para no depender solo del color. */
    showState?: boolean;
    id?: string;
  }

  let { checked, onchange, label, disabled = false, showState = true, id }: Props = $props();
</script>

<span class="nb-switch-wrap">
  {#if showState}
    <span class="nb-switch-state" class:nb-switch-state-on={checked} aria-hidden="true">
      {checked ? t("common.on") : t("common.off")}
    </span>
  {/if}
  <button
    {id}
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label}
    {disabled}
    class="nb-switch"
    class:nb-switch-on={checked}
    onclick={disabled ? undefined : () => onchange(!checked)}
  >
    <span class="nb-switch-thumb"></span>
  </button>
</span>

<style>
  .nb-switch-wrap {
    display: inline-flex;
    align-items: center;
    gap: var(--space-3);
    flex-shrink: 0;
  }

  .nb-switch-state {
    min-width: 5.5em;
    text-align: right;
    font-family: var(--font-ui);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-control);
    color: var(--color-text-muted);
  }

  .nb-switch-state-on {
    color: var(--color-text-primary);
  }

  .nb-switch {
    position: relative;
    width: 44px;
    height: 24px;
    padding: 0;
    flex-shrink: 0;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border-default);
    background: var(--surface-field);
    cursor: default;
    transition:
      background var(--duration-base) ease,
      border-color var(--duration-base) ease;
  }

  .nb-switch-thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: var(--radius-pill);
    background: var(--color-text-muted);
    transition:
      transform var(--duration-base) var(--ease-standard),
      background var(--duration-base) ease;
  }

  .nb-switch:hover:not(:disabled) {
    border-color: var(--border-hover);
  }

  .nb-switch-on {
    background: var(--gradient-nav-active);
    border-color: var(--border-accent-strong);
  }

  .nb-switch-on .nb-switch-thumb {
    transform: translateX(20px);
    background: var(--color-nav-active-text);
  }

  .nb-switch:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  .nb-switch:disabled {
    opacity: 0.45;
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-switch,
    .nb-switch-thumb {
      transition: none;
    }
  }
</style>
