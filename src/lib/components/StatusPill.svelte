<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * Estado con icono/color + texto — NUNCA solo color (§16.10, accesibilidad).
   * Los tonos son siempre los semánticos (success/warning/danger/info),
   * independientes del acento violeta de marca: así "Disponible" se
   * reconoce de un vistazo como estado y no se confunde con una acción. Los
   * tonos ya son distintos entre temas (tokens.css) para mantener el
   * contraste sobre fondo claro u oscuro; este componente no lo sabe.
   */
  type Tone = "success" | "warning" | "danger" | "info" | "neutral";

  interface Props {
    tone?: Tone;
    /** Forma "pill" translúcida (p. ej. "Visible en la red") o inline (junto a un valor). */
    variant?: "pill" | "inline";
    children: Snippet;
  }

  let { tone = "neutral", variant = "pill", children }: Props = $props();
</script>

<span class="nb-status nb-status-{variant} nb-tone-{tone}">
  <span class="nb-status-dot" aria-hidden="true"></span>
  <span class="nb-status-label">{@render children()}</span>
</span>

<style>
  .nb-status {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-family: var(--font-ui);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-control);
    color: var(--color-text-secondary);
  }

  .nb-status-pill {
    padding: 7px var(--space-4);
    border-radius: var(--radius-pill);
    background: var(--pill-bg);
    border: 1px solid var(--pill-border);
    backdrop-filter: blur(var(--blur-pill));
    -webkit-backdrop-filter: blur(var(--blur-pill));
  }

  /* C01 (auditoría v3): respaldo opaco §16.10. El pill no es ninguno de
     los tres materiales con nombre de §16.10 (`--pill-bg` es su propio
     token, no comparte gradiente con Card/Sidebar/Dialog) — tiene su
     propio token de respaldo, `--surface-solid-pill` (ver tokens.css). */
  @supports not (backdrop-filter: blur(1px)) {
    .nb-status-pill {
      background: var(--surface-solid-pill);
    }
  }

  .nb-status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    background: currentColor;
    box-shadow: 0 0 0 3px color-mix(in srgb, currentColor 18%, transparent);
  }

  .nb-status-inline .nb-status-dot {
    width: 6px;
    height: 6px;
  }

  .nb-tone-success {
    color: var(--color-success);
  }
  .nb-tone-warning {
    color: var(--color-warning);
  }
  .nb-tone-danger {
    color: var(--color-danger);
  }
  .nb-tone-info {
    color: var(--color-info);
  }
  .nb-tone-neutral {
    color: var(--color-text-secondary);
  }

  /* El texto siempre en el color de texto normal, no en el tono, salvo
     cuando el pill entero es el propio indicador de estado. */
  .nb-status-inline .nb-status-label {
    color: currentColor;
    font-weight: var(--font-weight-subtitle);
  }
  .nb-status-pill .nb-status-label {
    color: var(--color-text-secondary);
  }
</style>
