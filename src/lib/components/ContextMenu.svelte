<script lang="ts">
  /**
   * Menú contextual propio (clic derecho, tecla de menú o Mayús+F10 sobre un elemento).
   *
   * Patrón WAI-ARIA «menu»: `role="menu"` con `role="menuitem"`. Teclado: ↑/↓ mueven,
   * Inicio/Fin saltan a los extremos, Enter/Espacio activan, Esc cierra, Tab cierra. El foco
   * entra en el primer elemento activable al abrir; quien lo instancia lo devuelve al
   * elemento de origen al cerrar (igual que con `Dialog`).
   *
   * Solo colores por token (`var(--…)`); el sistema de diseño no incorpora librerías.
   */
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import type { IconName } from "./icons";

  export interface ContextMenuItem {
    id: string;
    label: string;
    icon?: IconName;
    /** Acción destructiva: se pinta con el tono de peligro. */
    danger?: boolean;
    disabled?: boolean;
    /** Línea separadora antes de este elemento. */
    separatorBefore?: boolean;
  }

  interface Props {
    items: ContextMenuItem[];
    /** Coordenadas de ventana donde se abre (esquina superior izquierda). */
    x: number;
    y: number;
    label: string;
    onSelect: (id: string) => void;
    onClose: () => void;
  }

  let { items, x, y, label, onSelect, onClose }: Props = $props();

  let menuEl = $state<HTMLDivElement | undefined>();
  let left = $state(0);
  let top = $state(0);

  function botones(): HTMLButtonElement[] {
    return menuEl
      ? Array.from(
          menuEl.querySelectorAll<HTMLButtonElement>('button[role="menuitem"]:not(:disabled)'),
        )
      : [];
  }

  function mover(delta: number | "first" | "last") {
    const lista = botones();
    if (lista.length === 0) return;
    const actual = lista.indexOf(document.activeElement as HTMLButtonElement);
    let siguiente: number;
    if (delta === "first") siguiente = 0;
    else if (delta === "last") siguiente = lista.length - 1;
    else siguiente = (actual + delta + lista.length) % lista.length;
    lista[siguiente]?.focus();
  }

  function onKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        mover(1);
        break;
      case "ArrowUp":
        e.preventDefault();
        mover(-1);
        break;
      case "Home":
        e.preventDefault();
        mover("first");
        break;
      case "End":
        e.preventDefault();
        mover("last");
        break;
      case "Escape":
      case "Tab":
        e.preventDefault();
        onClose();
        break;
    }
  }

  onMount(() => {
    // Ajuste al área visible: el menú no debe quedar cortado por el borde de la ventana.
    const rect = menuEl?.getBoundingClientRect();
    const w = rect?.width ?? 0;
    const h = rect?.height ?? 0;
    left = Math.max(4, Math.min(x, window.innerWidth - w - 4));
    top = Math.max(4, Math.min(y, window.innerHeight - h - 4));
    mover("first");

    // Clic fuera, otro clic derecho, pérdida de foco de la ventana o desplazamiento cierran.
    const fuera = (e: Event) => {
      if (menuEl && !menuEl.contains(e.target as Node)) onClose();
    };
    window.addEventListener("pointerdown", fuera, true);
    window.addEventListener("contextmenu", fuera, true);
    window.addEventListener("blur", onClose);
    window.addEventListener("resize", onClose);
    return () => {
      window.removeEventListener("pointerdown", fuera, true);
      window.removeEventListener("contextmenu", fuera, true);
      window.removeEventListener("blur", onClose);
      window.removeEventListener("resize", onClose);
    };
  });
</script>

<div
  bind:this={menuEl}
  class="nb-ctxmenu"
  role="menu"
  aria-label={label}
  tabindex="-1"
  style:left="{left}px"
  style:top="{top}px"
  onkeydown={onKeydown}
>
  {#each items as item (item.id)}
    {#if item.separatorBefore}
      <div class="nb-ctxmenu-sep" role="separator"></div>
    {/if}
    <button
      type="button"
      role="menuitem"
      class="nb-ctxmenu-item"
      class:nb-ctxmenu-danger={item.danger}
      disabled={item.disabled}
      onclick={() => onSelect(item.id)}
    >
      {#if item.icon}
        <Icon name={item.icon} size={14} />
      {:else}
        <span class="nb-ctxmenu-noicon" aria-hidden="true"></span>
      {/if}
      <span>{item.label}</span>
    </button>
  {/each}
</div>

<style>
  .nb-ctxmenu {
    position: fixed;
    z-index: 60;
    min-width: 220px;
    padding: var(--space-1);
    display: flex;
    flex-direction: column;
    background: var(--gradient-dialog);
    border: 1px solid var(--border-dialog);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-dialog);
    outline: none;
  }

  .nb-ctxmenu-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    text-align: left;
    cursor: default;
  }

  .nb-ctxmenu-item:hover:not(:disabled),
  .nb-ctxmenu-item:focus-visible {
    background: var(--hover-tint);
    outline: none;
  }

  .nb-ctxmenu-item:focus-visible {
    box-shadow: inset 0 0 0 2px var(--color-focus-ring);
  }

  .nb-ctxmenu-item:disabled {
    color: var(--color-text-disabled);
  }

  .nb-ctxmenu-danger {
    color: var(--color-danger);
  }

  .nb-ctxmenu-noicon {
    width: 14px;
    flex-shrink: 0;
  }

  .nb-ctxmenu-sep {
    height: 1px;
    margin: var(--space-1) 0;
    background: var(--border-default);
  }
</style>
