import { z } from "zod";
import { invokeCommand } from "./transport";
import { logger } from "../logging";

/**
 * Cierre de la ventana (`Historias.md` §5.2).
 *
 * El backend intercepta toda vía de cierre, decide y ejecuta; la interfaz solo **muestra** las
 * preguntas que él le pide por eventos y le devuelve lo que la persona eligió.
 */

/** Hay que preguntar: minimizar a la bandeja o cerrar. */
export const EVENTO_CIERRE_PREGUNTA = "app://close-ask";
/** Hay que confirmar que salir cancelará la prueba en curso. */
export const EVENTO_CIERRE_CONFIRMAR_SESION = "app://close-confirm-session";

export type EleccionAlCerrar = "minimize" | "exit";

const preguntaSchema = z.object({ sessionActive: z.boolean() });

async function escuchar(evento: string, alRecibir: (payload: unknown) => void) {
  try {
    // Import dinámico: sin Tauri (pruebas, navegador) no hay eventos y no pasa nada.
    const { listen } = await import("@tauri-apps/api/event");
    return await listen<unknown>(evento, (e) => alRecibir(e.payload));
  } catch {
    logger.warn({
      module: "lifecycle",
      eventCode: "CLOSE_EVENTS_UNAVAILABLE",
      message: `No se pudo escuchar ${evento}; el diálogo de cierre no saldrá`,
    });
    return () => {};
  }
}

/** Devuelve la función que deja de escuchar. */
export async function alPreguntarAlCerrar(
  alRecibir: (sesionActiva: boolean) => void,
): Promise<() => void> {
  return escuchar(EVENTO_CIERRE_PREGUNTA, (payload) => {
    const r = preguntaSchema.safeParse(payload);
    alRecibir(r.success ? r.data.sessionActive : false);
  });
}

/** Devuelve la función que deja de escuchar. */
export async function alConfirmarSalidaConPrueba(alRecibir: () => void): Promise<() => void> {
  return escuchar(EVENTO_CIERRE_CONFIRMAR_SESION, () => alRecibir());
}

/** Lo elegido en el diálogo. Con `remember`, queda como ajuste y no se vuelve a preguntar. */
export async function applyCloseChoice(choice: EleccionAlCerrar, remember: boolean): Promise<void> {
  await invokeCommand("app_close_apply", { choice, remember }, z.void().nullable());
}

/** Confirmado: se cancela la prueba en curso y se sale. */
export async function confirmCloseWithActiveSession(): Promise<void> {
  await invokeCommand("app_close_confirmed", undefined, z.void().nullable());
}
