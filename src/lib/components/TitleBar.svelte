<script lang="ts">
  /**
   * Barra de título propia (§16.13). La ventana se crea con `decorations: false`
   * y esta barra sustituye por completo la decoración nativa: arrastre por la
   * zona izquierda, doble clic para maximizar/restaurar, y tres botones propios
   * cuyo cierre SIEMPRE pasa por `CloseRequested` — el comando `close()` de la
   * ventana de Tauri es justo lo que dispara ese evento, así que Rust sigue
   * pudiendo interceptarlo para el diálogo de §5.2.
   *
   * Requiere `"decorations": false` en la ventana de `tauri.conf.json`.
   *
   * NOTA IMPORTANTE PARA QUIEN INTEGRE ESTO EN LA VENTANA REAL: en los
   * mockups (y en `HomeScreenExample.svelte`) esta barra vive dentro de un
   * contenedor con su propio `border-radius`/`border`, porque ahí se está
   * simulando una ventana flotante dentro de un navegador. En la app real la
   * ventana Tauri ES la ventana del sistema operativo: NO le pongas tú un
   * `border-radius` al elemento raíz. En Windows 11, DWM ya redondea las
   * esquinas de una ventana sin decoración; en Windows 10 no las redondea, y
   * está bien que no lo haga (§16.13 lo dice explícitamente). Si se replica
   * aquí ese redondeo, se duplica o se desincroniza con lo que pinta DWM.
   *
   * DESVIACIÓN DE SPEC MARCADA PARA APROBAR (revisión de entrega H1, apdo. 21
   * del correo del desarrollador): §16.13 pide que la barra use siempre el
   * plano opaco `--solid` (el mismo color que el lienzo) y que "no se perciba
   * como una barra distinta". La "Propuesta definitiva" del diseñador usa en
   * su lugar un degradado propio (`--gradient-titlebar`) como parte de la
   * identidad visual aprobada. Es una decisión de diseño consciente, no un
   * descuido — pero como contradice la letra de §16.13, queda documentada
   * aquí y en `README.md` § Registro de cambios para que el desarrollador la
   * dé por buena (o pida el plano opaco liso) antes de construir sobre ella.
   * Lo que SÍ se ha corregido para cumplir la spec al pie de la letra:
   * anchura de botón 46→48px y eliminación de la línea de separación bajo la
   * barra (`border-bottom`) — ninguna de las dos dependía del degradado.
   */
  import { onMount } from "svelte";

  interface Props {
    appName?: string;
    /** Textos localizados — sustituir por las claves i18n reales (§4). */
    labels?: {
      minimize: string;
      maximize: string;
      restore: string;
      close: string;
    };
  }

  let {
    appName = "NetworkBench",
    labels = {
      minimize: "Minimizar",
      maximize: "Maximizar",
      restore: "Restaurar",
      close: "Cerrar",
    },
  }: Props = $props();

  let isMaximized = $state(false);
  let appWindow: Awaited<ReturnType<typeof getWin>> | null = null;

  async function getWin() {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    return getCurrentWindow();
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        appWindow = await getWin();
        isMaximized = await appWindow.isMaximized();
        // El estado maximizado se sincroniza escuchando eventos de la
        // ventana, nunca asumiéndolo tras el clic (§16.13).
        unlisten = await appWindow.onResized(async () => {
          if (appWindow) isMaximized = await appWindow.isMaximized();
        });
      } catch {
        // Fuera de Tauri (p. ej. `vite dev` en el navegador): sin-op.
      }
    })();

    return () => unlisten?.();
  });

  function handleMinimize() {
    appWindow?.minimize();
  }

  function handleMaximizeToggle() {
    appWindow?.toggleMaximize();
  }

  function handleClose() {
    // Dispara CloseRequested; el lado Rust decide diálogo / minimizar a
    // bandeja / salir según closeAction (§5.2). Nunca `process.exit`.
    appWindow?.close();
  }

  function handleDoubleClick() {
    handleMaximizeToggle();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="titlebar" data-tauri-drag-region ondblclick={handleDoubleClick}>
  <div class="titlebar-brand" data-tauri-drag-region>
    <span class="titlebar-mark" aria-hidden="true"></span>
    <span class="titlebar-name">{appName}</span>
  </div>

  <div class="titlebar-controls">
    <button class="winctrl" type="button" aria-label={labels.minimize} onclick={handleMinimize}>
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      >
        <path d="M5 12h14" />
      </svg>
    </button>

    <button
      class="winctrl"
      type="button"
      aria-label={isMaximized ? labels.restore : labels.maximize}
      onclick={handleMaximizeToggle}
    >
      {#if isMaximized}
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        >
          <rect x="7" y="3" width="14" height="14" rx="1.5" />
          <rect x="3" y="7" width="14" height="14" rx="1.5" fill="var(--color-bg-base)" />
        </svg>
      {:else}
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        >
          <rect x="5" y="5" width="14" height="14" rx="1.5" />
        </svg>
      {/if}
    </button>

    <button
      class="winctrl winctrl-close"
      type="button"
      aria-label={labels.close}
      onclick={handleClose}
    >
      <svg
        width="13"
        height="13"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.9"
        stroke-linecap="round"
      >
        <path d="M6 6l12 12M18 6L6 18" />
      </svg>
    </button>
  </div>
</div>

<style>
  .titlebar {
    height: var(--titlebar-height);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-sizing: border-box;
    padding-left: var(--space-4);

    background: var(--gradient-titlebar);
    /* §16.13: "sin línea de separación: no debe percibirse como una barra
       distinta". Antes había un border-bottom de 1px — eliminado. */

    -webkit-user-select: none;
    user-select: none;
  }

  .titlebar-brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 100%;
    /* Toda esta franja es región de arrastre; los botones quedan fuera. */
    flex: 1 1 auto;
    min-width: 0;
  }

  .titlebar-mark {
    width: 17px;
    height: 17px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--gradient-brand-mark);
  }

  .titlebar-name {
    font-family: var(--font-ui);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-body);
    color: var(--color-text-tertiary);
    letter-spacing: 0.2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .titlebar-controls {
    display: flex;
    align-items: center;
    height: 100%;
    flex-shrink: 0;
  }

  .winctrl {
    width: 48px;
    height: var(--titlebar-height);
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--color-winctrl-icon);
    cursor: default;
    padding: 0;
    transition:
      background var(--duration-fast) ease,
      color var(--duration-fast) ease;
  }

  .winctrl:hover {
    background: var(--winctrl-hover-bg);
  }

  .winctrl:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: -2px;
  }

  .winctrl-close:hover {
    background: var(--winctrl-close-hover-bg);
    color: var(--winctrl-close-hover-text);
  }

  @media (prefers-reduced-motion: reduce) {
    .winctrl {
      transition: none;
    }
  }
</style>
