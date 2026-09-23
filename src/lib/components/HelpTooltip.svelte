<script lang="ts">
  import Tooltip from "./Tooltip.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    text: string;
    label?: string;
    placement?: "top" | "bottom" | "left" | "right";
  }

  let { text, label = "Ayuda", placement = "top" }: Props = $props();
</script>

<span class="help-tooltip-container">
  <Tooltip {text} {placement}>
    {#snippet children(tooltipId)}
      <button type="button" class="help-btn" aria-label={label} aria-describedby={tooltipId}>
        <Icon name="info" size={14} />
      </button>
    {/snippet}
  </Tooltip>
</span>

<style>
  .help-tooltip-container {
    display: inline-flex;
    align-items: center;
    vertical-align: middle;
    margin-left: var(--spacing-1, 4px);
  }

  .help-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    padding: 0;
    border: none;
    border-radius: var(--radius-full, 9999px);
    background: var(--bg-hover, rgba(255, 255, 255, 0.08));
    color: var(--text-muted, #94a3b8);
    cursor: help;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  .help-btn:hover,
  .help-btn:focus-visible {
    background: var(--bg-active, rgba(255, 255, 255, 0.15));
    color: var(--text-primary, #ffffff);
    outline: 2px solid var(--accent, #8b5cf6);
    outline-offset: 1px;
  }
</style>
