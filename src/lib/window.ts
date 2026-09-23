import { z } from "zod";
import { invokeCommand } from "./api/transport";
import { logger } from "./logging";

export const windowGeometrySchema = z.object({
  x: z.number().int(),
  y: z.number().int(),
  width: z.number().int().min(800),
  height: z.number().int().min(600),
  isMaximized: z.boolean(),
});

export type WindowGeometry = z.infer<typeof windowGeometrySchema>;

export async function getWindowGeometry(): Promise<WindowGeometry | null> {
  try {
    return await invokeCommand<WindowGeometry | null>(
      "window_get_geometry",
      undefined,
      windowGeometrySchema.nullable(),
    );
  } catch {
    logger.warn({
      module: "window",
      eventCode: "GET_GEOMETRY_FAILED",
      message: "No se pudo obtener la geometría de la ventana",
    });
    return null;
  }
}

export async function saveWindowGeometry(geom: WindowGeometry): Promise<void> {
  try {
    await invokeCommand("window_save_geometry", geom);
  } catch {
    logger.error({
      module: "window",
      eventCode: "SAVE_GEOMETRY_FAILED",
      message: "Error al persistir la geometría de la ventana",
    });
  }
}

export async function restoreAndShowWindow(): Promise<WindowGeometry | null> {
  try {
    const applied = await invokeCommand<WindowGeometry>(
      "window_restore_and_show",
      undefined,
      windowGeometrySchema,
    );
    logger.info({
      module: "window",
      eventCode: "WINDOW_RESTORED_AND_SHOWN",
      message: "Ventana restaurada y visible",
      safeParams: {
        width: String(applied.width),
        height: String(applied.height),
      },
    });
    return applied;
  } catch {
    logger.error({
      module: "window",
      eventCode: "RESTORE_AND_SHOW_FAILED",
      message: "Fallo al restaurar y mostrar la ventana",
    });
    return null;
  }
}

let trackingInitialized = false;

export async function setupWindowTracking(): Promise<() => void> {
  if (trackingInitialized || typeof window === "undefined") {
    return () => {};
  }
  trackingInitialized = true;

  try {
    const tauriWindow = await import("@tauri-apps/api/window");
    const currentWin = tauriWindow.getCurrentWindow();

    let debounceTimer: ReturnType<typeof setTimeout> | null = null;

    const saveCurrentState = async () => {
      try {
        const isMax = await currentWin.isMaximized();
        const pos = await currentWin.innerPosition();
        const size = await currentWin.innerSize();

        if (size.width >= 800 && size.height >= 600) {
          await saveWindowGeometry({
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
            isMaximized: isMax,
          });
        }
      } catch {
        // Ignorar fallos de sondeo de ventana
      }
    };

    const debouncedSave = () => {
      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        void saveCurrentState();
      }, 500);
    };

    const unlistenResize = await currentWin.onResized(debouncedSave);
    const unlistenMove = await currentWin.onMoved(debouncedSave);

    return () => {
      if (debounceTimer) clearTimeout(debounceTimer);
      unlistenResize();
      unlistenMove();
      trackingInitialized = false;
    };
  } catch {
    // Si no estamos en runtime de Tauri (ej. tests unitarios o navegador web)
    return () => {
      trackingInitialized = false;
    };
  }
}
