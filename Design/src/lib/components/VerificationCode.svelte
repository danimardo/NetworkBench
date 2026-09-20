<script lang="ts">
  /**
   * Código de verificación de seis cifras (§9.1). NO es un campo donde se
   * teclea nada — ambos equipos calculan el mismo código y lo MUESTRAN; el
   * usuario solo lo compara a ojo con el otro equipo y pulsa "Coincide" o
   * "No coincide". Cifras en tabular-nums para que no "bailen" al
   * renderizarse, y `user-select: text` porque es un valor que tiene
   * sentido copiar/leer en voz alta (§16.14: selección de texto permitida
   * en valores marcados como copiables).
   */
  interface Props {
    /** Las seis cifras, ya formateadas o en crudo — se reparten solas en "123 456". */
    code: string;
  }

  let { code }: Props = $props();
  let digits = $derived(code.replace(/\D/g, "").padEnd(6, "•").split(""));
</script>

<div class="nb-vcode" aria-label="Código de verificación {code}">
  {#each digits as digit, i (i)}
    <span class="nb-vcode-digit" class:nb-vcode-gap={i === 3}>{digit}</span>
  {/each}
</div>

<style>
  .nb-vcode {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: var(--space-4) var(--space-5);
    border-radius: var(--radius-lg);
    background: var(--surface-field);
    border: 1px solid var(--border-field);
    user-select: text;
  }

  .nb-vcode-digit {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 28px;
    font-weight: 600;
    color: var(--color-text-primary);
    letter-spacing: 1px;
  }

  .nb-vcode-gap {
    margin-left: 10px;
  }
</style>
