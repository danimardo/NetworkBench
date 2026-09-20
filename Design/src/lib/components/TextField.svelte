<script lang="ts">
  /**
   * Campo de texto WinUI-style (§16.14: "nunca `<select>` nativo... con
   * aspecto y comportamiento WinUI 3"). Primer uso en H1: "Dirección del
   * equipo" en Conectar manualmente (§7.2), pero genérico para cualquier
   * campo de texto futuro (alias de favorito, nombre visible en Ajustes...).
   *
   * Estados: reposo, hover, foco, error, deshabilitado, solo lectura —
   * todos vía tokens de `--surface-field-*`/`--border-field-*`
   * (DESIGN_TOKENS.md § Diálogos, overlay y formularios). El error se marca
   * con borde+halo Y con icono+texto (nunca solo color, §16.10).
   */
  import Icon from "./Icon.svelte";

  interface Props {
    label: string;
    value?: string;
    placeholder?: string;
    /** Texto de ayuda bajo el campo cuando NO hay error. */
    hint?: string;
    /** Si se da, sustituye a `hint` y pinta el campo en estado de error. */
    error?: string;
    disabled?: boolean;
    readonly?: boolean;
    type?: "text" | "search";
    id?: string;
    oninput?: (value: string) => void;
  }

  let {
    label,
    value = $bindable(""),
    placeholder = "",
    hint = "",
    error = "",
    disabled = false,
    readonly = false,
    type = "text",
    id = `nb-field-${Math.random().toString(36).slice(2, 9)}`,
    oninput,
  }: Props = $props();

  let hasError = $derived(!!error);
</script>

<div class="nb-field" class:nb-field-disabled={disabled}>
  <label class="nb-field-label" for={id}>{label}</label>
  <div class="nb-field-control" class:nb-field-control-error={hasError}>
    {#if type === "search"}
      <span class="nb-field-icon"><Icon name="search" size={14} /></span>
    {/if}
    <input
      {id}
      {type}
      {placeholder}
      {disabled}
      {readonly}
      bind:value
      oninput={(e) => oninput?.((e.target as HTMLInputElement).value)}
      aria-invalid={hasError || undefined}
      aria-describedby={hint || error ? `${id}-desc` : undefined}
    />
  </div>
  {#if error}
    <p id="{id}-desc" class="nb-field-desc nb-field-desc-error">
      <Icon name="alert-triangle" size={12} />
      {error}
    </p>
  {:else if hint}
    <p id="{id}-desc" class="nb-field-desc">{hint}</p>
  {/if}
</div>

<style>
  .nb-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-family: var(--font-ui);
  }

  .nb-field-label {
    font-size: 12.5px;
    font-weight: var(--font-weight-control);
    color: var(--color-text-secondary);
  }

  .nb-field-control {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-3);
    height: 40px;
    border-radius: var(--radius-sm);
    background: var(--surface-field);
    border: 1px solid var(--border-field);
    transition: background var(--duration-fast) ease, border-color var(--duration-fast) ease, box-shadow var(--duration-fast) ease;
  }

  .nb-field-control:hover {
    background: var(--surface-field-hover);
    border-color: var(--border-field-hover);
  }

  .nb-field-control:focus-within {
    border-color: var(--border-field-focus);
    box-shadow: var(--shadow-field-focus);
  }

  .nb-field-control-error {
    border-color: var(--border-field-error);
  }
  .nb-field-control-error:focus-within {
    box-shadow: var(--shadow-field-error);
  }

  .nb-field-icon {
    display: flex;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .nb-field-control input {
    flex: 1;
    min-width: 0;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    /* §16.14: cursor de texto solo en campos editables. */
    cursor: text;
    user-select: text;
  }

  .nb-field-control input::placeholder {
    color: var(--color-field-placeholder);
  }

  .nb-field-control input:disabled {
    cursor: default;
  }

  .nb-field-disabled .nb-field-control {
    background: var(--surface-field-disabled);
    opacity: 0.7;
  }

  .nb-field-desc {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: var(--font-size-2xs);
    color: var(--color-text-muted);
  }

  .nb-field-desc-error {
    color: var(--color-danger);
  }

  @media (prefers-reduced-motion: reduce) {
    .nb-field-control { transition: none; }
  }
</style>
