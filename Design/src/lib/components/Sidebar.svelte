<script lang="ts">
  import Icon from "./Icon.svelte";
  import Tooltip from "./Tooltip.svelte";
  import type { IconName } from "./icons";

  export interface NavItem {
    id: string;
    label: string;
    icon: IconName;
  }

  interface Props {
    items?: NavItem[];
    activeId: string;
    /**
     * Auditoría v3 (A07): "qué pasa si se pulsa Inicio durante una prueba
     * en curso — no debe poder abandonarse el recorrido sin querer".
     * "Inicio" (`id === "inicio"`) NUNCA se deshabilita, a propósito —
     * ver más abajo por qué — así que este callback puede recibir
     * `onNavigate("inicio")` estando `disabled` a `true`. Es
     * responsabilidad de quien use este componente interceptar ese caso
     * y mostrar una confirmación ("¿Salir de la prueba en curso?") en
     * vez de navegar directo — igual que hace `maqueta-navegable.html`
     * con su overlay `confirm-leave` y su estado `sessionActive`. Este
     * componente no sabe nada de sesiones ni de diálogos de
     * confirmación; solo decide qué queda pulsable.
     */
    onNavigate: (id: string) => void;
    /** true durante una sesión activa: el resto de la navegación queda
     * deshabilitada con tooltip (§16.1) — desde `CONNECTING`, no desde
     * `PREPARING` (auditoría v3, A07: antes se pasaba `disabled` más
     * tarde de lo debido, dejando operativa la barra durante todo el
     * tramo de emparejamiento/espera de aceptación). */
    disabled?: boolean;
    disabledHint?: string;
  }

  let {
    items = [
      { id: "inicio", label: "Inicio", icon: "home" },
      { id: "historial", label: "Historial", icon: "history" },
      { id: "ajustes", label: "Ajustes", icon: "settings" },
    ],
    activeId,
    onNavigate,
    disabled = false,
    disabledHint = "Prueba en curso",
  }: Props = $props();
</script>

<nav class="nb-sidebar" aria-label="Navegación principal">
  {#each items as item (item.id)}
    {#if disabled && item.id !== activeId && item.id !== "inicio"}
      <!-- §16.1: "el resto de la barra se deshabilita con tooltip «Prueba en
           curso»". Antes usaba `title=` (tooltip nativo del navegador) — ver
           correo de ampliación, apdo. 21 punto 5: sustituido por Tooltip.svelte,
           el mismo patrón que el resto de la app.
           "inicio" queda fuera de esta rama a propósito (A07): no es un
           ítem inerte más — sigue siendo un botón normal de abajo, que
           dispara `onNavigate("inicio")` igual que cualquier otro clic;
           es quien usa este componente quien decide si eso navega o abre
           una confirmación.
           C04: usaba el atributo `disabled` nativo además de `aria-disabled`
           — pero `disabled` saca el botón del orden de tabulación, así que
           el propio Tooltip (que se muestra al recibir el foco) nunca era
           alcanzable por teclado, contradiciendo la nota de accesibilidad
           de Tooltip.svelte. Se cambia a solo `aria-disabled="true"`: sigue
           foco-alcanzable (el lector de pantalla anuncia "deshabilitado" y
           no hay `onclick` en esta rama, así que no hace nada al activarlo),
           y el atenuado visual pasa de `:disabled` a `[aria-disabled]` en el
           CSS de abajo. -->
      <Tooltip text={disabledHint} placement="right">
        {#snippet children(tooltipId)}
          <button
            type="button"
            class="nb-navitem"
            aria-current={undefined}
            aria-disabled="true"
            aria-describedby={tooltipId}
          >
            <Icon name={item.icon} size={19} />
            <span class="nb-navitem-label">{item.label}</span>
          </button>
        {/snippet}
      </Tooltip>
    {:else}
      <button
        type="button"
        class="nb-navitem"
        class:nb-navitem-active={item.id === activeId}
        aria-current={item.id === activeId ? "page" : undefined}
        onclick={() => onNavigate(item.id)}
      >
        <Icon name={item.icon} size={19} />
        <span class="nb-navitem-label">{item.label}</span>
      </button>
    {/if}
  {/each}
</nav>

<style>
  .nb-sidebar {
    width: var(--sidebar-width);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-1);
    padding-top: var(--space-5);
    box-sizing: border-box;

    background: var(--sidebar-bg);
    border-right: 1px solid var(--sidebar-border);
    backdrop-filter: blur(var(--blur-sidebar));
    -webkit-backdrop-filter: blur(var(--blur-sidebar));
  }

  /* C01 (auditoría v3): respaldo opaco §16.10 — material `glass-chrome`
     (ver DESIGN_TOKENS.md § "Los tres materiales…", que también nombra a
     TitleBar.svelte para este material; TitleBar no necesita esta misma
     regla porque no usa `backdrop-filter` — su fondo es un degradado
     propio sin blur, ya opaco por sí mismo). */
  @supports not (backdrop-filter: blur(1px)) {
    .nb-sidebar {
      background: var(--surface-solid-chrome);
    }
  }

  .nb-navitem {
    width: 60px;
    height: 58px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 5px;
    border-radius: var(--radius-md);
    color: var(--color-text-tertiary);
    cursor: default;
    transition: background var(--duration-base) ease, color var(--duration-base) ease;
  }

  .nb-navitem:hover:not(:disabled):not([aria-disabled="true"]):not(.nb-navitem-active) {
    background: var(--hover-tint);
  }

  /* C04: el ítem deshabilitado ya no lleva el atributo `disabled` nativo
     (ver comentario más arriba), así que el atenuado tiene que engancharse
     también a `[aria-disabled="true"]`, no solo a `:disabled`. */
  .nb-navitem:disabled,
  .nb-navitem[aria-disabled="true"] {
    opacity: 0.4;
  }

  .nb-navitem:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  .nb-navitem-active {
    color: var(--color-nav-active-text);
    background: var(--gradient-nav-active);
    box-shadow: var(--shadow-nav-active);
  }

  .nb-navitem-label {
    font-family: var(--font-ui);
    font-size: 10.5px;
    font-weight: var(--font-weight-subtitle);
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-navitem {
      transition: none;
    }
  }
</style>
