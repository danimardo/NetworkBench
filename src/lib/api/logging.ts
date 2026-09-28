import type { LogEventParams, LogLevelName } from "../logging";

/**
 * Envía un evento de log del frontend al comando Rust `diagnostics_log_frontend_event`,
 * que lo escribe en `networkbench.log` con `origin: "frontend"`.
 *
 * Deliberadamente NO pasa por `invokeCommand` (`transport.ts`): ese wrapper llama a
 * `logger.error(...)` en su propio catch, y usarlo aquí crearía un bucle si este mismo
 * envío fallara. Un fallo se descarta en silencio: perder una línea de log nunca debe
 * romper ni ensuciar la aplicación.
 */
export async function logFrontendEvent(
  event: LogEventParams & { level: LogLevelName },
): Promise<void> {
  try {
    const tauri = await import("@tauri-apps/api/core");
    await tauri.invoke("diagnostics_log_frontend_event", {
      evento: {
        level: event.level,
        module: event.module,
        eventCode: event.eventCode,
        message: event.message,
        errorCode: event.errorCode,
        diagnosticId: event.diagnosticId,
        safeParams: event.safeParams ?? {},
      },
    });
  } catch {
    // Sin backend real (pruebas, navegador suelto) o comando no disponible: se ignora.
  }
}
