import { logFrontendEvent } from "../api/logging";
import { logger } from "./index";

/**
 * Conecta `logger.setBridge(...)` con el comando Rust que escribe en `networkbench.log`
 * con `origin: "frontend"`.
 *
 * Hallazgo real de la auditoría de logging (2026-09-28): `setBridge` existía desde el
 * principio, pero nadie lo llamaba nunca — cualquier `logger.error(...)` del frontend
 * (incluidos los `IpcError` que ve la persona en pantalla) solo llegaba a la consola del
 * navegador, invisible en la aplicación empaquetada.
 */
export function installBackendLogBridge(): void {
  logger.setBridge((event) => {
    void logFrontendEvent(event);
  });
}
