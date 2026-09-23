<script lang="ts">
  /**
   * Notificación interna, DENTRO de la ventana de la app — no confundir con
   * el toast NATIVO de Windows (§10.4: solo cuando la app está minimizada/
   * en bandeja, [H3], y ese lo construye el sistema operativo a partir de
   * los datos que le pasa Rust, no este componente Svelte).
   *
   * Uso en H1: aviso "Prueba aceptada automáticamente" (§9.2) cuando la
   * ventana está abierta y un peer de confianza con aceptación automática
   * activada empieza una prueba sin preguntar.
   *
   * Se posiciona flotando sobre el contenido (no en el flujo del layout);
   * quien lo usa decide cuándo montarlo/desmontarlo y durante cuánto
   * tiempo — este componente no se autodestruye para que las pruebas
   * (Playwright, capturas) puedan verlo sin depender de un timer.
   */
  import Icon from "./Icon.svelte";
  import type { IconName } from "./icons";
  import type { Snippet } from "svelte";

  type Tone = "info" | "success" | "warning" | "danger";

  interface Props {
    tone?: Tone;
    icon?: IconName;
    onDismiss?: () => void;
    children: Snippet;
  }

  const TONE_ICON: Record<Tone, IconName> = {
    info: "info",
    success: "check",
    warning: "alert-triangle",
    danger: "x-circle",
  };

  let { tone = "info", icon, onDismiss, children }: Props = $props();
  let resolvedIcon = $derived(icon ?? TONE_ICON[tone]);
</script>

<div class="nb-toast nb-tone-{tone}" role="status" aria-live="polite">
  <span class="nb-toast-icon"><Icon name={resolvedIcon} size={16} /></span>
  <span class="nb-toast-text">{@render children()}</span>
  {#if onDismiss}
    <button type="button" class="nb-toast-dismiss" aria-label="Cerrar aviso" onclick={onDismiss}>
      <Icon name="close" size={13} />
    </button>
  {/if}
</div>

<style>
  .nb-toast {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    box-sizing: border-box;
    max-width: 380px;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--gradient-dialog);
    border: 1px solid var(--border-dialog);
    box-shadow: var(--shadow-md);
    backdrop-filter: blur(var(--blur-modal)) saturate(var(--material-saturate));
    -webkit-backdrop-filter: blur(var(--blur-modal)) saturate(var(--material-saturate));
    color: var(--color-text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    line-height: 1.4;
    animation: nb-toast-in var(--duration-slow) var(--ease-standard);
  }

  @supports not (backdrop-filter: blur(1px)) {
    .nb-toast {
      background: var(--surface-solid-overlay);
    }
  }

  .nb-toast-icon {
    flex-shrink: 0;
    margin-top: 1px;
  }
  .nb-tone-info .nb-toast-icon {
    color: var(--color-info);
  }
  .nb-tone-success .nb-toast-icon {
    color: var(--color-success);
  }
  .nb-tone-warning .nb-toast-icon {
    color: var(--color-warning);
  }
  .nb-tone-danger .nb-toast-icon {
    color: var(--color-danger);
  }

  .nb-toast-text {
    flex: 1;
    color: var(--color-text-secondary);
  }

  .nb-toast-dismiss {
    flex-shrink: 0;
    display: flex;
    color: var(--color-text-muted);
    border-radius: var(--radius-xs);
    padding: 2px;
    transition:
      color var(--duration-fast) ease,
      background var(--duration-fast) ease;
  }
  .nb-toast-dismiss:hover {
    color: var(--color-text-primary);
    background: var(--hover-tint);
  }
  .nb-toast-dismiss:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 1px;
  }

  @keyframes nb-toast-in {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-toast {
      animation: none;
    }
  }
</style>
