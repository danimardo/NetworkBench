<script lang="ts">
  /**
   * Composición principal de la pantalla de Ejecución (§16.4: "dos
   * tarjetas de equipo unidas por una línea de conexión, con una flecha
   * que indica la dirección actual del envío").
   *
   * Auditoría v3 (A05): "falta el elemento principal que exige §16.4" —
   * antes Ejecución solo tenía velocidad + barra + gráfica, sin
   * representar los dos extremos de la prueba como tal.
   *
   * Sobre "sin invertir el significado al cambiar de perspectiva
   * local/remoto" (el otro punto de A05): este componente NO tiene ningún
   * prop de perspectiva. `local` es siempre este equipo — siempre se
   * pinta a la izquierda — y `remote` es siempre el otro — siempre a la
   * derecha. La app real no necesita un selector de perspectiva: cada
   * equipo simplemente renderiza su propia pantalla con sus propios
   * datos, y en esa pantalla "local" solo puede significar una cosa. La
   * dirección (`direction`) es la única pieza que cambia con el tiempo —
   * "outbound" (local → remote) o "inbound" (remote → local) — y nunca
   * mueve las tarjetas de sitio, solo la flecha y su etiqueta. Es
   * justamente separar estos dos ejes (qué tarjeta es "este equipo" vs.
   * hacia dónde va el envío ahora mismo) lo que hace imposible la
   * inversión que señalaba la auditoría.
   *
   * (La maqueta navegable sí añade, por encima de este componente, un
   * selector "Ver desde" — pero es solo para poder enseñar las dos
   * vistas posibles desde un único archivo de demo; no existe en la app.)
   */
  import Icon from "./Icon.svelte";
  import type { IconName } from "./icons";

  export type AdapterType = "ethernet" | "wifi" | "other";

  export interface ExecutionEndpoint {
    name: string;
    /** Modelo de NIC (§16.4: "modelo de NIC, velocidad de enlace"). */
    nicModel: string;
    adapterType: AdapterType;
    linkSpeedMbps: number;
  }

  interface Props {
    local: ExecutionEndpoint;
    remote: ExecutionEndpoint;
    /** Sentido del envío ACTUAL, nunca de la perspectiva. */
    direction: "outbound" | "inbound";
  }

  let { local, remote, direction }: Props = $props();

  const ADAPTER_ICON: Record<AdapterType, IconName> = {
    ethernet: "ethernet",
    wifi: "wifi",
    other: "adapter-other",
  };

  // Mismo formateador que DeviceCard.svelte (§16.8) — duplicado a
  // propósito en vez de extraído a un util compartido: es un componente
  // nuevo de esta misma ronda (H1) y una extracción prematura de dos
  // usos es más riesgo de acoplar mal que beneficio; revisar si aparece
  // un tercer uso.
  function formatLinkSpeed(mbps: number): string {
    if (mbps <= 0) return "";
    if (mbps < 1000) return `${Math.round(mbps)} Mbit/s`;
    return `${(mbps / 1000).toFixed(2).replace(".", ",")} Gbit/s`;
  }

  let linkLabel = $derived(
    direction === "outbound" ? `${local.name} → ${remote.name}` : `${remote.name} → ${local.name}`,
  );
</script>

<div class="nb-exec-link">
  <div class="nb-exec-node">
    <span class="nb-exec-node-icon"><Icon name="desktop" size={16} /></span>
    <div class="nb-exec-node-text">
      <span class="nb-exec-node-name">{local.name}</span>
      <span class="nb-exec-node-tag">Este equipo</span>
      <span class="nb-exec-node-nic">
        <Icon name={ADAPTER_ICON[local.adapterType]} size={12} />
        {local.nicModel} · {formatLinkSpeed(local.linkSpeedMbps)}
      </span>
    </div>
  </div>

  <div class="nb-exec-link-mid" data-dir={direction === "outbound" ? "ab" : "ba"}>
    <svg class="nb-exec-arrow" viewBox="0 0 120 24" aria-hidden="true">
      <line x1="4" y1="12" x2="106" y2="12" class="nb-exec-arrow-line" stroke-width="2.5" />
      <path d="M99 4 L114 12 L99 20 Z" class="nb-exec-arrow-head" />
    </svg>
    <span class="nb-exec-link-label">{linkLabel}</span>
  </div>

  <div class="nb-exec-node">
    <span class="nb-exec-node-icon"><Icon name="desktop" size={16} /></span>
    <div class="nb-exec-node-text">
      <span class="nb-exec-node-name">{remote.name}</span>
      <span class="nb-exec-node-nic">
        <Icon name={ADAPTER_ICON[remote.adapterType]} size={12} />
        {remote.nicModel} · {formatLinkSpeed(remote.linkSpeedMbps)}
      </span>
    </div>
  </div>
</div>

<style>
  .nb-exec-link {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .nb-exec-node {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1 1 0;
    min-width: 0;
    padding: 14px 16px;
    border-radius: var(--radius-lg);
    background: var(--gradient-card-default);
    border: 1px solid var(--border-default);
  }

  .nb-exec-node-icon {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: var(--icon-chip-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .nb-exec-node-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .nb-exec-node-name {
    font-size: 13px;
    font-weight: 650;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nb-exec-node-tag {
    font-size: 10.5px;
    font-weight: 650;
    color: var(--color-accent);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .nb-exec-node-nic {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nb-exec-link-mid {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    min-width: 96px;
  }

  .nb-exec-arrow {
    width: 90px;
    height: 20px;
    transition: transform var(--duration-base) ease;
  }

  .nb-exec-arrow-line {
    stroke: var(--color-chart-a-b);
  }
  .nb-exec-arrow-head {
    fill: var(--color-chart-a-b);
  }

  /* inbound: espejo horizontal del propio SVG en vez de redibujar
     coordenadas — la flecha apunta al revés sin mover ni un nodo de
     sitio, y el mismo criterio de §16.7/§19.4 (nunca solo color) se
     aplica aquí con el trazo discontinuo. */
  .nb-exec-link-mid[data-dir="ba"] .nb-exec-arrow {
    transform: scaleX(-1);
  }
  .nb-exec-link-mid[data-dir="ba"] .nb-exec-arrow-line {
    stroke: var(--color-chart-b-a);
    stroke-dasharray: 6 5;
  }
  .nb-exec-link-mid[data-dir="ba"] .nb-exec-arrow-head {
    fill: var(--color-chart-b-a);
  }

  .nb-exec-link-label {
    font-size: 10.5px;
    color: var(--color-text-muted);
    text-align: center;
    white-space: nowrap;
  }

  @media (max-width: 640px) {
    .nb-exec-link {
      flex-direction: column;
      align-items: stretch;
    }
    .nb-exec-node {
      width: 100%;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-exec-arrow {
      transition: none;
    }
  }
</style>
