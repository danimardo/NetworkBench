/**
 * Controlador de tema y accesibilidad de movimiento — Sistema / Claro / Oscuro (§23, FR-037).
 *
 * El tema inicial obligatorio por FR-037 es "dark" (Oscuro).
 * Toda la lógica de qué colores usar vive en tokens.css
 * (:root = oscuro, :root[data-theme="light"] = claro).
 */

export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const STORAGE_KEY = "networkbench.theme";
const REDUCE_MOTION_KEY = "networkbench.reduce_motion";

function readStoredMode(): ThemeMode {
  if (typeof window === "undefined") return "dark";
  const stored = window.localStorage.getItem(STORAGE_KEY);
  // FR-037: Si no hay valor almacenado, el inicial por defecto es "dark"
  return stored === "light" || stored === "dark" || stored === "system" ? stored : "dark";
}

function readStoredReduceMotion(): boolean {
  if (typeof window === "undefined") return false;
  const stored = window.localStorage.getItem(REDUCE_MOTION_KEY);
  if (stored !== null) return stored === "true";
  if (typeof window.matchMedia === "function") {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  }
  return false;
}

function systemPrefersLight(): boolean {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") return false;
  return window.matchMedia("(prefers-color-scheme: light)").matches;
}

class ThemeController {
  mode = $state<ThemeMode>(readStoredMode());
  reduceMotion = $state<boolean>(readStoredReduceMotion());
  #systemIsLight = $state(systemPrefersLight());

  resolved: ResolvedTheme = $derived(
    this.mode === "system" ? (this.#systemIsLight ? "light" : "dark") : this.mode,
  );

  constructor() {
    if (typeof window === "undefined") return;

    if (typeof window.matchMedia === "function") {
      const mqlTheme = window.matchMedia("(prefers-color-scheme: light)");
      mqlTheme.addEventListener?.("change", (e) => {
        this.#systemIsLight = e.matches;
      });

      const mqlMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
      mqlMotion.addEventListener?.("change", (e) => {
        if (window.localStorage.getItem(REDUCE_MOTION_KEY) === null) {
          this.reduceMotion = e.matches;
        }
      });
    }

    // Aplica data-theme y data-reduce-motion en <html> cada vez que cambian
    $effect.root(() => {
      $effect(() => {
        document.documentElement.setAttribute("data-theme", this.resolved);
      });
      $effect(() => {
        if (this.reduceMotion) {
          document.documentElement.setAttribute("data-reduce-motion", "true");
        } else {
          document.documentElement.removeAttribute("data-reduce-motion");
        }
      });
    });
  }

  setMode(mode: ThemeMode) {
    this.mode = mode;
    if (typeof window !== "undefined") {
      window.localStorage.setItem(STORAGE_KEY, mode);
    }
  }

  setReduceMotion(enabled: boolean) {
    this.reduceMotion = enabled;
    if (typeof window !== "undefined") {
      window.localStorage.setItem(REDUCE_MOTION_KEY, String(enabled));
    }
  }
}

/** Instancia única compartida por toda la app. */
export const theme = new ThemeController();
