<script lang="ts">
  interface Props {
    /** Lado en píxeles. */
    size?: number;
    /** Con marco: el cuadrado redondeado oscuro del icono del ejecutable (Acerca de). Sin él, solo la esfera (barra de título). */
    framed?: boolean;
  }

  let { size = 24, framed = false }: Props = $props();
</script>

<!-- Decorativo: el nombre de la app ya va escrito al lado, así que no se anuncia. -->
<span
  class="nb-appicon"
  class:nb-appicon-framed={framed}
  style:width="{size}px"
  style:height="{size}px"
  aria-hidden="true"
>
  <span class="nb-appicon-sphere">
    <svg viewBox="0 0 24 24" fill="none" stroke-linecap="round" stroke-linejoin="round">
      <path d="M4 12h4l2.5-6 3 12 2.5-6h4" />
    </svg>
  </span>
</span>

<style>
  .nb-appicon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .nb-appicon-framed {
    border-radius: 22%;
    background: var(--color-bg-base);
  }

  .nb-appicon-sphere {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: var(--gradient-brand-mark);
  }

  .nb-appicon-framed .nb-appicon-sphere {
    width: 72%;
    height: 72%;
  }

  svg {
    width: 64%;
    height: 64%;
    stroke: var(--color-nav-active-text);
    stroke-width: 2.6;
  }
</style>
