<script lang="ts">
  /**
   * Fondo ambiental de la app: varias fuentes de luz difusas (radial-gradients)
   * más dos formas abstractas en los extremos inferiores. Deliberadamente NO es
   * una imagen: así se mantiene fiel a cualquier tamaño/resolución/escalado de
   * ventana (§16.9) y permite ajustar la intensidad desde tokens sin rehacer
   * ningún recurso gráfico.
   *
   * Cien por cien dirigido por tokens: `--gradient-app-bg`, `--gradient-blob-1`,
   * `--gradient-blob-2` y sus opacidades/sombras cambian solo con
   * `data-theme` (ver tokens.css) — este componente no sabe si está en claro
   * u oscuro.
   *
   * `state` deja preparado el enganche de §16.10 ("luz ambiental que puede
   * reaccionar suavemente al estado: neutro en reposo, cálido en éxito, ámbar
   * en advertencia") sin comprometer el reposo: por defecto es "idle" y el
   * fondo es exactamente el de la propuesta aprobada, en cualquiera de los
   * dos temas.
   */
  type BgState = "idle" | "success" | "warning" | "danger";

  interface Props {
    state?: BgState;
  }

  let { state = "idle" }: Props = $props();
</script>

<div class="app-background" data-state={state} aria-hidden="true"></div>

<style>
  .app-background {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    z-index: 0;

    background: var(--gradient-app-bg);
    transition: filter var(--duration-slow) var(--ease-standard);
  }

  /* Enganches de estado — a estrenar cuando el motor de interpretación
     (§13) exista; hoy no cambian nada visible por defecto. Deliberadamente
     un filtro (no un color nuevo): así funciona igual de bien encima del
     gradiente claro y del oscuro sin declarar un tercer juego de tokens. */
  .app-background[data-state="success"] {
    filter: saturate(1.05) hue-rotate(-6deg);
  }
  .app-background[data-state="warning"] {
    filter: saturate(1.08) hue-rotate(10deg);
  }
  .app-background[data-state="danger"] {
    filter: saturate(1.1) hue-rotate(18deg);
  }

  .app-background::before {
    content: "";
    position: absolute;
    width: 620px;
    height: 300px;
    left: 4%;
    bottom: -190px;
    border-radius: 50%;
    background: var(--gradient-blob-1);
    filter: blur(38px);
    opacity: var(--blob-1-opacity);
  }

  .app-background::after {
    content: "";
    position: absolute;
    width: 700px;
    height: 400px;
    right: -180px;
    bottom: -190px;
    border-radius: 50%;
    background: var(--gradient-blob-2);
    box-shadow: var(--shadow-blob-2);
    filter: blur(6px);
    opacity: var(--blob-2-opacity);
  }

  @media (prefers-reduced-motion: reduce) {
    .app-background {
      transition: none;
    }
  }
</style>
