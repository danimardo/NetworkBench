<script lang="ts">
  /**
   * Referencia de composición — NO es la pantalla de Inicio real (esa vive en
   * `routes/` y se conecta a `lib/stores/` y `lib/api/`, §25). Esto solo
   * demuestra cómo encajan los átomos del sistema de diseño, reproduce la
   * "Propuesta definitiva" con datos de muestra, e incluye un `ThemeToggle`
   * para comprobar en caliente que TODO cambia de tema sin tocar ni una
   * línea de estos componentes — si algo no cambia de color al pulsarlo, es
   * que quedó algún literal sin convertir a token en algún componente.
   *
   * El `<div class="nb-window">` con `border-radius`/`border` es SOLO para
   * esta vista de referencia (simula una ventana flotante en el navegador).
   * En la app real ese contenedor no existe: `TitleBar.svelte` ya explica
   * por qué (DWM se encarga del redondeo de esquinas en Win11).
   */
  import AppBackground from "./AppBackground.svelte";
  import TitleBar from "./TitleBar.svelte";
  import Sidebar from "./Sidebar.svelte";
  import Card from "./Card.svelte";
  import Button from "./Button.svelte";
  import StatusPill from "./StatusPill.svelte";
  import DeviceCard from "./DeviceCard.svelte";
  import Icon from "./Icon.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";

  let activeNav = $state("inicio");
</script>

<div class="nb-window">
  <TitleBar />

  <div class="nb-body">
    <AppBackground />

    <Sidebar activeId={activeNav} onNavigate={(id) => (activeNav = id)} />

    <main class="nb-main">
      <header class="nb-header">
        <div class="nb-header-id">
          <span class="nb-eyebrow">Este equipo</span>
          <div class="nb-header-line">
            <span class="nb-header-name">DESKTOP-DANIEL</span>
            <span class="nb-dim">·</span>
            <span class="nb-header-meta">Intel X550-T2</span>
            <span class="nb-dim">·</span>
            <span class="nb-header-meta nb-mono">10 Gbit/s</span>
            <span class="nb-dim">·</span>
            <span class="nb-header-meta nb-mono">192.168.10.15</span>
          </div>
        </div>
        <div class="nb-header-actions">
          <ThemeToggle />
          <StatusPill tone="success">Visible en la red</StatusPill>
        </div>
      </header>

      <Card variant="hero">
        <div class="nb-hero-body">
          <div class="nb-hero-text">
            <span class="nb-hero-eyebrow">Prueba estándar</span>
            <span class="nb-hero-title">Analiza tu conexión con otro equipo</span>
            <span class="nb-hero-sub">TCP en ambas direcciones · aprox. 1 minuto</span>
            <div class="nb-hero-last">
              <Icon name="check" size={13} color="var(--color-success)" />
              <span>Última prueba: SERVER-01, hoy 10:32</span>
              <Button variant="ghost">Repetir con SERVER-01</Button>
            </div>
          </div>
          {#snippet connectIcon()}<Icon name="connect" size={18} />{/snippet}
          <Button variant="primary" icon={connectIcon}>Analizar conexión</Button>
        </div>
      </Card>

      <section class="nb-section">
        <div class="nb-section-head">
          <div class="nb-section-title">
            <span>Equipos disponibles</span>
            <span class="nb-muted">3 en la red</span>
          </div>
          {#snippet linkIcon()}<Icon name="link" size={14} />{/snippet}
          <Button variant="secondary" icon={linkIcon}>Conectar manualmente</Button>
        </div>

        <div class="nb-device-row">
          <DeviceCard
            name="SERVER-01"
            ip="192.168.10.22"
            adapterType="ethernet"
            linkSpeedMbps={10000}
            availability="available"
            trust="trusted"
            favorite
          />
          <DeviceCard
            name="PORTATIL-MARIA"
            ip="192.168.10.40"
            adapterType="wifi"
            linkSpeedMbps={1000}
            availability="available"
            trust="known"
          />
          <!-- Deliberadamente ocupado + de confianza + favorito a la vez:
               es el ejemplo exacto de la revisión de entrega H1 ("un equipo
               puede ser favorito, de confianza y estar ocupado al mismo
               tiempo") — demuestra que las tres propiedades ahora son
               independientes y no se pisan entre sí. -->
          <DeviceCard
            name="NAS-BACKUP"
            ip="192.168.10.8"
            adapterType="ethernet"
            linkSpeedMbps={2500}
            availability="busy"
            trust="trusted"
            favorite
          />
        </div>
      </section>

      <section class="nb-section">
        <span class="nb-section-title-solo">Otros equipos</span>
        <div class="nb-chip-row">
          <button class="nb-chip">
            <Icon name="star" size={12} color="var(--color-accent)" />
            SERVER-01
          </button>
          <button class="nb-chip">PC-OFICINA</button>
          <button class="nb-chip nb-chip-muted">LAPTOP-PERU · hace 3 días</button>
        </div>
      </section>
    </main>
  </div>
</div>

<style>
  .nb-window {
    width: 100%;
    height: 100%;
    min-width: 800px;
    min-height: 600px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-main);
    /* Ver comentario superior: este marco es solo para esta vista de
       referencia dentro del navegador. */
    border-radius: 12px;
    border: 1px solid var(--window-border);
    font-family: var(--font-ui);
    color: var(--color-text-primary);
    overflow: hidden;
  }

  .nb-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: row;
    position: relative;
  }

  .nb-main {
    flex: 1;
    min-width: 0;
    padding: 36px 40px;
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
    position: relative;
    z-index: 1;
    box-sizing: border-box;
    overflow: auto;
  }

  .nb-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .nb-header-id {
    display: flex;
    flex-direction: column;
    gap: 7px;
    min-width: 0;
  }

  .nb-header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-shrink: 0;
  }

  .nb-eyebrow {
    font-size: 11px;
    font-weight: var(--font-weight-subtitle);
    letter-spacing: 0.6px;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .nb-header-line {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .nb-header-name {
    font-size: var(--font-size-2xl);
    font-weight: var(--font-weight-title);
  }

  .nb-header-meta {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }

  .nb-mono {
    font-variant-numeric: tabular-nums;
  }

  .nb-dim {
    font-size: var(--font-size-sm);
    color: var(--color-text-muted);
  }

  .nb-hero-body {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-6);
    width: 100%;
  }

  .nb-hero-text {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .nb-hero-eyebrow {
    font-size: 11px;
    font-weight: var(--font-weight-title);
    letter-spacing: 0.6px;
    text-transform: uppercase;
    color: var(--color-accent);
  }

  .nb-hero-title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-title);
  }

  .nb-hero-sub {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }

  .nb-hero-last {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: 4px;
    font-size: 12.5px;
    color: var(--color-text-secondary);
  }

  .nb-section {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .nb-section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .nb-section-title {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    font-size: 14.5px;
    font-weight: var(--font-weight-title);
  }

  .nb-section-title-solo {
    font-size: 13.5px;
    font-weight: var(--font-weight-title);
  }

  .nb-muted {
    font-size: 12.5px;
    font-weight: var(--font-weight-body);
    color: var(--color-text-muted);
  }

  .nb-device-row {
    display: flex;
    gap: var(--space-4);
  }

  .nb-device-row > :global(*) {
    flex: 1;
    min-width: 0;
  }

  .nb-chip-row {
    display: flex;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .nb-chip {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 9px 14px;
    border-radius: var(--radius-md);
    background: var(--surface-secondary);
    border: 1px solid var(--border-default);
    font-size: 12.5px;
    font-weight: var(--font-weight-control);
    color: var(--color-text-secondary);
    cursor: default;
  }

  .nb-chip-muted {
    color: var(--color-text-muted);
  }
</style>
