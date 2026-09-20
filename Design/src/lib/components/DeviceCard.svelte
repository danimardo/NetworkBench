<script lang="ts">
  /**
   * Tarjeta de equipo — Inicio (§16.2), selector (§16.3), "Otros equipos" (§7.3).
   *
   * CORRECCIÓN (revisión de entrega H1, apdo. 21 del correo del desarrollador,
   * apdo. 3 "En las tarjetas de equipo hay que separar estas propiedades,
   * porque pueden coincidir"): la v1 de este componente tenía un único enum
   * `status: "available" | "busy" | "trusted" | "incompatible"` que mezclaba
   * disponibilidad, compatibilidad y confianza en un solo valor, y encima
   * reutilizaba el `variant="selected"` de Card para "de confianza" — lo que
   * literalmente confundía "este equipo está seleccionado para la prueba"
   * con "confío en este equipo", que es justo lo que la revisión pedía
   * separar. Ahora son cinco propiedades independientes que SÍ pueden
   * coincidir (el ejemplo del correo: favorito + de confianza + ocupado al
   * mismo tiempo):
   *
   *   - `availability`  → estado real de red: disponible / ocupado / no accesible.
   *   - `compatible`    → versión de protocolo compatible (§8.4). Si es `false`
   *                       prevalece sobre `availability` en la fila de estado:
   *                       no se puede probar con ese equipo pase lo que pase.
   *   - `trust`         → nivel de confianza (§9.2): desconocido / conocido / de confianza.
   *                       Se muestra con icono de escudo, NUNCA con estrella,
   *                       para no confundirse visualmente con `favorite`.
   *   - `favorite`      → marcado por el usuario (§7.3). Estrella dorada junto al nombre.
   *   - `selected`      → estado de interacción puntual: esta tarjeta es la
   *                       elegida ahora mismo en una lista de selección
   *                       (§16.3). Es lo ÚNICO que activa `Card` variant="selected".
   *
   * Un equipo real puede tener las cinco a la vez; el layout las apila sin
   * que ninguna tape a otra.
   *
   * C04 (auditoría v3, accesibilidad): el escudo de confianza ganó un punto
   * no-color + foco propio (ver comentario junto a `.nb-device-trust` más
   * abajo). La separación favorito/tarjeta (`e.stopPropagation()` en el
   * botón de estrella) ya estaba resuelta desde antes de esta entrega —
   * verificado de nuevo, sin cambios.
   */
  import Card from "./Card.svelte";
  import Icon from "./Icon.svelte";
  import Tooltip from "./Tooltip.svelte";
  import type { IconName } from "./icons";

  export type Availability = "available" | "busy" | "unreachable";
  export type Trust = "unknown" | "known" | "trusted";
  export type AdapterType = "ethernet" | "wifi" | "other";

  interface Props {
    name: string;
    ip: string;
    /** Alias local (§6.2): si existe se muestra como "alias (displayName)". */
    alias?: string;
    /**
     * Auditoría v3 (C05): antes tenía `"wifi"` por defecto, lo que
     * contradecía el propio comentario de este fichero ("nunca asumir
     * Wi-Fi") en cuanto alguien omitía la prop por descuido — un valor
     * por defecto silencioso es justo "asumir". Ahora es obligatoria:
     * quien monte la tarjeta tiene que decidir explícitamente, y si de
     * verdad no se conoce el tipo de adaptador, pasar `"other"` a
     * propósito, no dejar que el compilador lo decida por omisión.
     */
    adapterType: AdapterType;
    /** Velocidad de enlace en Mbit/s (0 = desconocida/no se muestra). */
    linkSpeedMbps?: number;
    availability?: Availability;
    /** `false` → "Versión incompatible" (§8.4), tarjeta no seleccionable. */
    compatible?: boolean;
    trust?: Trust;
    favorite?: boolean;
    /** Esta tarjeta es la elegida ahora mismo en una lista de selección. */
    selected?: boolean;
    onclick?: (e: MouseEvent) => void;
    onToggleFavorite?: (e: MouseEvent) => void;
  }

  let {
    name,
    ip,
    alias = "",
    adapterType,
    linkSpeedMbps = 0,
    availability = "available",
    compatible = true,
    trust = "unknown",
    favorite = false,
    selected = false,
    onclick,
    onToggleFavorite,
  }: Props = $props();

  // §16.8 — formato de unidades: < 1000 Mbit/s sin decimales; ≥ 1 Gbit/s con
  // 2 decimales y coma como separador (idioma español; sustituir por el
  // formateador i18n real de la app, que decide por el idioma de la UI, no
  // por la configuración regional de Windows).
  function formatLinkSpeed(mbps: number): string {
    if (mbps <= 0) return "";
    if (mbps < 1000) return `${Math.round(mbps)} Mbit/s`;
    return `${(mbps / 1000).toFixed(2).replace(".", ",")} Gbit/s`;
  }

  const ADAPTER_ICON: Record<AdapterType, IconName> = {
    ethernet: "ethernet",
    wifi: "wifi",
    other: "adapter-other",
  };

  const AVAILABILITY_META: Record<Availability, { tone: "success" | "warning" | "danger"; label: string; icon?: IconName }> = {
    available: { tone: "success", label: "Disponible" },
    busy: { tone: "warning", label: "Ocupado" },
    unreachable: { tone: "danger", label: "No accesible", icon: "x-circle" },
  };

  // Un equipo ocupado, no accesible o incompatible no se puede elegir para
  // una prueba nueva (§5.5: `NB-PEER-003` si de todas formas se intentara).
  let isSelectable = $derived(compatible && availability === "available");
  let cardVariant = $derived(selected ? "selected" : "default");
  let statusMeta = $derived(!compatible
    ? { tone: "danger" as const, label: "Versión incompatible", icon: "alert-triangle" as IconName }
    : AVAILABILITY_META[availability]);
</script>

<Card
  variant={cardVariant}
  disabled={!isSelectable}
  onclick={onclick && isSelectable ? onclick : undefined}
>
  <div class="nb-device-head">
    <span class="nb-device-icon">
      <Icon name="desktop" size={17} />
    </span>
    <div class="nb-device-id">
      <div class="nb-device-name-row">
        <!-- Auditoría v3 (A03): "consulta con nombre truncado" — el nombre
             ya se recorta con ellipsis cuando no cabe (`.nb-device-name`,
             text-overflow: ellipsis), pero no había forma de ver el
             nombre completo. `title` da el nombre completo (con el alias
             real detrás si lo hay) sin depender de Tooltip, que es
             overkill para un texto que ya lleva su propio recorte visual. -->
        <span class="nb-device-name" title={alias ? `${alias} (${name})` : name}>
          {#if alias}
            {alias} <span class="nb-device-name-real">({name})</span>
          {:else}
            {name}
          {/if}
        </span>
        {#if trust !== "unknown"}
          <!-- C04 (auditoría v3, accesibilidad): el escudo distinguía
               "conocido" de "de confianza" SOLO por color (regla 6 del
               README — "icono/punto + texto, nunca solo color"). Se añade
               `nb-device-trust-badge`, un punto no-color visible únicamente
               en `trusted`, y el propio escudo pasa a ser foco-alcanzable
               (`tabindex`, `role="img"`, `aria-label`) para que además de
               verse distinto, un lector de pantalla lo anuncie y el
               Tooltip (que ahora se muestra al recibir el foco, no por
               `:focus-within` — ver Tooltip.svelte) tenga algo que
               enfocar. -->
          <Tooltip text={trust === "trusted" ? "De confianza" : "Conocido"}>
            {#snippet children(tooltipId)}
              <span
                class="nb-device-trust"
                class:nb-device-trust-full={trust === "trusted"}
                tabindex="0"
                role="img"
                aria-label={trust === "trusted" ? "De confianza" : "Conocido"}
                aria-describedby={tooltipId}
              >
                <Icon name="shield" size={12} />
                {#if trust === "trusted"}
                  <span class="nb-device-trust-badge" aria-hidden="true"></span>
                {/if}
              </span>
            {/snippet}
          </Tooltip>
        {/if}
        <button
          type="button"
          class="nb-device-fav"
          class:nb-device-fav-on={favorite}
          aria-pressed={favorite}
          aria-label={favorite ? "Quitar de favoritos" : "Marcar como favorito"}
          onclick={(e) => {
            e.stopPropagation();
            onToggleFavorite?.(e);
          }}
        >
          <Icon name={favorite ? "star" : "star-outline"} size={13} />
        </button>
      </div>
      <span class="nb-device-ip">{ip}</span>
    </div>
  </div>

  <div class="nb-device-foot">
    {#if linkSpeedMbps > 0 && compatible}
      <span class="nb-device-link">
        <Icon name={ADAPTER_ICON[adapterType]} size={12} />
        {formatLinkSpeed(linkSpeedMbps)}
      </span>
    {:else}
      <span></span>
    {/if}

    <span
      class="nb-device-status"
      class:nb-tone-success={statusMeta.tone === "success"}
      class:nb-tone-warning={statusMeta.tone === "warning"}
      class:nb-tone-danger={statusMeta.tone === "danger"}
    >
      {#if statusMeta.icon}
        <Icon name={statusMeta.icon} size={12} />
      {:else}
        <span class="nb-device-status-dot" aria-hidden="true"></span>
      {/if}
      {statusMeta.label}
    </span>
  </div>
</Card>

<style>
  .nb-device-head {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .nb-device-icon {
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    background: var(--icon-chip-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .nb-device-id {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .nb-device-name-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .nb-device-name {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-title);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 0;
  }

  .nb-device-name-real {
    font-weight: var(--font-weight-body);
    color: var(--color-text-muted);
  }

  /* Confianza: escudo apagado (conocido) o relleno en acento (de confianza).
     Deliberadamente distinto en forma Y color del botón de favorito, que va
     justo al lado — son dos ejes distintos y pueden estar los dos activos. */
  .nb-device-trust {
    position: relative;
    display: inline-flex;
    color: var(--color-text-muted);
    flex-shrink: 0;
    border-radius: var(--radius-xs);
  }
  .nb-device-trust-full {
    color: var(--color-accent);
  }
  .nb-device-trust:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
  }
  /* C04: distinción no-color de "de confianza" — un punto pequeño, aparte
     del cambio de color del propio escudo (regla 6 del README). */
  .nb-device-trust-badge {
    position: absolute;
    top: -2px;
    right: -3px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-accent);
  }

  .nb-device-fav {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    margin-left: auto;
    flex-shrink: 0;
    color: var(--color-text-muted);
    border-radius: var(--radius-xs);
    transition: color var(--duration-fast) ease, background var(--duration-fast) ease;
  }
  .nb-device-fav:hover {
    background: var(--hover-tint);
    color: var(--color-accent);
  }
  .nb-device-fav-on {
    color: var(--color-accent);
  }
  .nb-device-fav:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 1px;
  }

  .nb-device-ip {
    font-size: var(--font-size-2xs);
    font-variant-numeric: tabular-nums;
    color: var(--color-text-muted);
  }

  .nb-device-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: var(--space-3);
  }

  .nb-device-link {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: var(--color-text-secondary);
  }

  .nb-device-status {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: var(--font-size-2xs);
    font-weight: var(--font-weight-title);
  }

  .nb-device-status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }

  .nb-tone-success { color: var(--color-success); }
  .nb-tone-warning { color: var(--color-warning); }
  .nb-tone-danger  { color: var(--color-danger); }
</style>
