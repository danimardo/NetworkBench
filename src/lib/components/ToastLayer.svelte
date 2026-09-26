<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<!--
  Capa para avisos (`Toast`): flota sobre el contenido, abajo y centrada en la zona a la
  derecha de la barra lateral, y **no ocupa sitio en el flujo**. Un banner dentro de la página
  empujaba los controles hacia abajo cada vez que aparecía.

  La capa entera deja pasar los clics; solo los avisos los reciben.
-->
<div class="nb-toastlayer">
  {@render children()}
</div>

<style>
  .nb-toastlayer {
    position: fixed;
    left: var(--sidebar-width);
    right: 0;
    bottom: var(--space-6);
    z-index: 40;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-4);
    pointer-events: none;
  }

  .nb-toastlayer :global(> *) {
    pointer-events: auto;
  }
</style>
