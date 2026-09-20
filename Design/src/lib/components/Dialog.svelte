<script lang="ts">
  /**
   * Diálogo modal propio (§16.14: "modales propios centrados en la
   * ventana... no ventanas nuevas del sistema"). Material `glass-overlay`
   * (ver DESIGN_TOKENS.md). Base de TODOS los diálogos de H1: aceptación de
   * prueba (§10.4), emparejamiento (§9.1), cierre con sesión activa (§5.2,
   * ya como referencia para H3), avisos de acción delicada (§13 del correo
   * de ampliación).
   *
   * Comportamiento ya resuelto aquí para que ningún diálogo concreto tenga
   * que reinventarlo (§13 del correo — "foco inicial y retorno del foco",
   * "Enter/Escape/clic exterior"):
   *   - Se centra en la ventana, con `--overlay-scrim` detrás.
   *   - Escape cierra (llama a `onClose`) salvo que `dismissible={false}`
   *     (usarlo SOLO para diálogos de acción obligatoria, p. ej. "se
   *     cancelará la prueba" — nunca para ocultar información al usuario).
   *   - Clic en el scrim exterior = igual que Escape.
   *   - El foco entra al primer elemento focuseable del diálogo al abrir y
   *     vuelve al elemento que lo abrió al cerrar (gestionado por quien
   *     instancia el diálogo, guardando `document.activeElement` antes de
   *     abrir — este componente solo mueve el foco DENTRO al montarse).
   *   - Animación de entrada/salida con la libertad de §16.10 ("momento
   *     premio" aparte: aquí es un diálogo, no el resultado — se mantiene
   *     sobria), respetando `prefers-reduced-motion`.
   *   - C04 (auditoría v3, accesibilidad): Tab/Shift+Tab quedan atrapados
   *     dentro del diálogo (antes solo se ponía el foco inicial, pero
   *     nada impedía que Tab lo sacara hacia el fondo — `aria-modal="true"`
   *     por sí solo es una pista para lectores de pantalla, no una
   *     barrera real de teclado). El resto de la ventana se marca
   *     `inert` mientras el diálogo está abierto, así que tampoco es
   *     alcanzable por un lector de pantalla en modo de exploración. Se
   *     asume que este componente cuelga cerca de la raíz de la pantalla
   *     (hijo directo del contenedor de ruta) — si en la integración real
   *     acaba anidado más adentro, hay que revisar qué nivel se marca
   *     `inert` para que siga siendo "todo menos el diálogo".
   *
   * Uso:
   *   <Dialog title="Solicitud de prueba de red" onClose={() => open = false}>
   *     {#snippet children()}...{/snippet}
   *     {#snippet actions()}<Button variant="ghost">Rechazar</Button><Button variant="primary">Aceptar</Button>{/snippet}
   *   </Dialog>
   */
  import { onMount, onDestroy } from "svelte";
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    /** Texto secundario bajo el título (opcional). */
    description?: string;
    onClose?: () => void;
    dismissible?: boolean;
    /** Ancho del diálogo — la mayoría son "sm", el de aceptación es "md". */
    size?: "sm" | "md" | "lg";
    children: Snippet;
    actions?: Snippet;
  }

  let { title, description = "", onClose, dismissible = true, size = "sm", children, actions }: Props = $props();

  let dialogEl = $state<HTMLDivElement | undefined>();
  let scrimEl = $state<HTMLDivElement | undefined>();
  // C04: elementos a los que se les quitó `inert` al cerrar, para no tocar
  // nada que ya estuviera `inert` por otro motivo antes de abrir este diálogo.
  let unInertedOnClose: HTMLElement[] = [];

  const FOCUSABLE_SELECTOR =
    'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

  function getFocusable(): HTMLElement[] {
    if (!dialogEl) return [];
    return Array.from(dialogEl.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
      (el) => el.offsetParent !== null // visible only — un botón oculto no debe formar parte del ciclo de Tab
    );
  }

  onMount(() => {
    // Foco inicial: el primer control focuseable del propio diálogo (nunca
    // el fondo). Si no hay ninguno (raro), el propio panel recibe el foco
    // para que el lector de pantalla anuncie el título igualmente.
    const focusable = getFocusable();
    (focusable[0] ?? dialogEl)?.focus();

    // C04: `inert` en todo lo que no sea este overlay, para que Tab y la
    // navegación de un lector de pantalla no puedan salir del diálogo por
    // ningún camino (no solo el foco por teclado). Se asume que `scrimEl`
    // cuelga cerca de la raíz — ver comentario de arriba sobre el supuesto.
    const root = scrimEl?.parentElement;
    if (root) {
      Array.from(root.children).forEach((el) => {
        const htmlEl = el as HTMLElement;
        if (htmlEl !== scrimEl && !htmlEl.inert) {
          htmlEl.inert = true;
          unInertedOnClose.push(htmlEl);
        }
      });
    }
  });

  onDestroy(() => {
    unInertedOnClose.forEach((el) => {
      el.inert = false;
    });
    unInertedOnClose = [];
  });

  function trapFocus(e: KeyboardEvent) {
    const focusable = getFocusable();
    if (focusable.length === 0) {
      // Nada focuseable dentro (raro, p. ej. un diálogo solo de lectura sin
      // botones todavía) — el foco no tiene dónde ir salvo quedarse en el
      // propio panel, así que Tab no hace nada en vez de escapar fuera.
      e.preventDefault();
      dialogEl?.focus();
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement as HTMLElement | null;
    const activeIsInside = !!active && !!dialogEl?.contains(active);
    if (e.shiftKey) {
      if (!activeIsInside || active === first) {
        e.preventDefault();
        last.focus();
      }
    } else {
      if (!activeIsInside || active === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && dismissible) {
      e.stopPropagation();
      onClose?.();
      return;
    }
    if (e.key === "Tab") {
      trapFocus(e);
    }
  }

  function handleScrimClick() {
    if (dismissible) onClose?.();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="nb-dialog-scrim" bind:this={scrimEl} onclick={handleScrimClick} role="presentation">
  <div
    bind:this={dialogEl}
    class="nb-dialog nb-dialog-{size}"
    role="dialog"
    aria-modal="true"
    aria-labelledby="nb-dialog-title"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="nb-dialog-head">
      <h2 id="nb-dialog-title" class="nb-dialog-title">{title}</h2>
      {#if description}
        <p class="nb-dialog-description">{description}</p>
      {/if}
    </div>

    <div class="nb-dialog-body">
      {@render children()}
    </div>

    {#if actions}
      <div class="nb-dialog-actions">
        {@render actions()}
      </div>
    {/if}
  </div>
</div>

<style>
  .nb-dialog-scrim {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
    background: var(--overlay-scrim);
    animation: nb-scrim-in var(--duration-slow) var(--ease-standard);
  }

  .nb-dialog {
    box-sizing: border-box;
    width: 100%;
    max-height: calc(100% - var(--space-8));
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-6);
    border-radius: var(--radius-xl);
    background: var(--gradient-dialog);
    border: 1px solid var(--border-dialog);
    box-shadow: var(--shadow-dialog);
    backdrop-filter: blur(var(--blur-modal)) saturate(var(--material-saturate));
    -webkit-backdrop-filter: blur(var(--blur-modal)) saturate(var(--material-saturate));
    color: var(--color-text-primary);
    font-family: var(--font-ui);
    animation: nb-dialog-in var(--duration-slow) var(--ease-standard);
  }

  @supports not (backdrop-filter: blur(1px)) {
    .nb-dialog {
      background: var(--surface-solid-overlay);
    }
  }

  .nb-dialog:focus-visible {
    outline: none; /* el diálogo mueve el foco a su primer control; no necesita su propio anillo */
  }

  .nb-dialog-sm { max-width: var(--dialog-width-sm); }
  .nb-dialog-md { max-width: var(--dialog-width-md); }
  .nb-dialog-lg { max-width: var(--dialog-width-lg); }

  .nb-dialog-head {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .nb-dialog-title {
    margin: 0;
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-title);
  }

  .nb-dialog-description {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .nb-dialog-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .nb-dialog-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-3);
    margin-top: var(--space-2);
  }

  @keyframes nb-scrim-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes nb-dialog-in {
    from { opacity: 0; transform: translateY(10px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-dialog-scrim,
    .nb-dialog {
      animation: none;
    }
  }
</style>
