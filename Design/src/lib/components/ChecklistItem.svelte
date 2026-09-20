<script lang="ts">
  /**
   * Fila de comprobación — pantalla de Preparación (§10.5, PREPARING):
   * "lista de pasos y estado (✓ / ○ / ✗)". Cinco estados en vez de los tres
   * de la spec porque en la práctica hacen falta para no dejar a medias
   * "en curso" y "necesita una acción" (p. ej. UAC rechazado, §14.4):
   *
   *   pending    → ○ hueco, texto atenuado (aún no le toca)
   *   running    → espiral animada (paso actual)
   *   done       → ✓ verde
   *   failed     → ✗ rojo + el motivo del error debajo, si se da
   *   action     → ⚠ ámbar + acción disponible (p. ej. "Reintentar como administrador")
   */
  import Icon from "./Icon.svelte";
  import type { Snippet } from "svelte";

  type ItemState = "pending" | "running" | "done" | "failed" | "action";

  interface Props {
    label: string;
    state: ItemState;
    /** Motivo del fallo o de la acción necesaria (p. ej. texto de NB-FW-004). */
    detail?: string;
    /** Acción disponible cuando `state === "action"` o `"failed"` con reintento. */
    actionSlot?: Snippet;
  }

  let { label, state, detail = "", actionSlot }: Props = $props();
</script>

<div class="nb-check-item nb-check-{state}">
  <span class="nb-check-icon" aria-hidden="true">
    {#if state === "pending"}
      <span class="nb-check-dot-empty"></span>
    {:else if state === "running"}
      <span class="nb-check-spinner"></span>
    {:else if state === "done"}
      <Icon name="check" size={14} />
    {:else if state === "failed"}
      <Icon name="x-circle" size={15} />
    {:else}
      <Icon name="alert-triangle" size={14} />
    {/if}
  </span>

  <div class="nb-check-text">
    <span class="nb-check-label">{label}</span>
    {#if detail}
      <span class="nb-check-detail">{detail}</span>
    {/if}
    {#if actionSlot}
      <div class="nb-check-action">{@render actionSlot()}</div>
    {/if}
  </div>
</div>

<style>
  .nb-check-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: 10px 2px;
  }

  .nb-check-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .nb-check-pending .nb-check-icon { color: var(--color-text-disabled); }
  .nb-check-running .nb-check-icon { color: var(--color-accent); }
  .nb-check-done .nb-check-icon    { color: var(--color-success); }
  .nb-check-failed .nb-check-icon  { color: var(--color-danger); }
  .nb-check-action .nb-check-icon  { color: var(--color-warning); }

  .nb-check-dot-empty {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 1.5px solid currentColor;
  }

  .nb-check-spinner {
    width: 15px;
    height: 15px;
    border-radius: 50%;
    border: 2px solid var(--border-default);
    border-top-color: var(--color-accent);
    animation: nb-spin var(--duration-spinner) linear infinite;
  }

  .nb-check-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-top: 1px;
  }

  .nb-check-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-control);
    color: var(--color-text-primary);
  }

  .nb-check-pending .nb-check-label {
    color: var(--color-text-muted);
  }

  .nb-check-detail {
    font-size: 12px;
    color: var(--color-text-secondary);
    line-height: 1.4;
  }

  .nb-check-failed .nb-check-detail {
    color: var(--color-danger);
  }

  .nb-check-action .nb-check-detail {
    color: var(--color-warning);
  }

  .nb-check-action {
    margin-top: 4px;
  }

  @keyframes nb-spin {
    to { transform: rotate(360deg); }
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-check-spinner {
      /* C03: la mitad de velocidad de --duration-spinner, no un segundo
         token — es una relación (el doble), no un valor independiente. */
      animation-duration: calc(var(--duration-spinner) * 2);
    }
  }
</style>
