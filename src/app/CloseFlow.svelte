<script lang="ts">
  import { onMount } from "svelte";
  import CloseChoiceDialog from "./CloseChoiceDialog.svelte";
  import CloseDialog from "./CloseDialog.svelte";
  import {
    alConfirmarSalidaConPrueba,
    alPreguntarAlCerrar,
    applyCloseChoice,
    confirmCloseWithActiveSession,
    type EleccionAlCerrar,
  } from "../lib/api/lifecycle";
  import { logger } from "../lib/logging";

  /**
   * Los diálogos del cierre de la ventana (§5.2). No decide nada: el backend intercepta el
   * cierre, elige qué hacer según el ajuste «Al cerrar la ventana» y solo cuando hay que
   * preguntar avisa por un evento. Aquí se muestra la pregunta y se devuelve la respuesta.
   */
  let preguntando = $state(false);
  let confirmandoPrueba = $state(false);

  onMount(() => {
    let vivo = true;
    const dejarDeEscuchar: (() => void)[] = [];

    void (async () => {
      const preguntas = await alPreguntarAlCerrar(() => (preguntando = true));
      const confirmaciones = await alConfirmarSalidaConPrueba(() => (confirmandoPrueba = true));
      // Si el componente ya se desmontó mientras se suscribía, no deja nada escuchando.
      if (vivo) dejarDeEscuchar.push(preguntas, confirmaciones);
      else {
        preguntas();
        confirmaciones();
      }
    })();

    return () => {
      vivo = false;
      dejarDeEscuchar.forEach((f) => f());
    };
  });

  async function elegir(eleccion: EleccionAlCerrar, recordar: boolean) {
    preguntando = false;
    try {
      await applyCloseChoice(eleccion, recordar);
    } catch (err) {
      logger.warn({
        module: "lifecycle",
        eventCode: "CLOSE_CHOICE_FAILED",
        message: `No se pudo aplicar la elección de cierre: ${String(err)}`,
      });
    }
  }

  async function confirmarSalida() {
    confirmandoPrueba = false;
    try {
      await confirmCloseWithActiveSession();
    } catch (err) {
      logger.warn({
        module: "lifecycle",
        eventCode: "CLOSE_CONFIRM_FAILED",
        message: `No se pudo cancelar la prueba y salir: ${String(err)}`,
      });
    }
  }
</script>

{#if preguntando}
  <CloseChoiceDialog onChoose={elegir} onDismiss={() => (preguntando = false)} />
{/if}

<CloseDialog
  open={confirmandoPrueba}
  oncancel={() => (confirmandoPrueba = false)}
  onconfirm={confirmarSalida}
/>
