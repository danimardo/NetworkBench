/**
 * Controlador de tema — Sistema / Claro / Oscuro (§23, Ajustes → General → Tema).
 *
 * Toda la lógica de qué colores usar vive en `tokens.css`
 * (`:root` = oscuro, `:root[data-theme="light"]` = claro). Este fichero
 * SOLO decide qué atributo `data-theme` poner en <html> y no conoce ni un
 * color — así, si mañana se añade un tercer tema, esto no cambia.
 *
 * Uso en cualquier componente/pantalla:
 *
 *   import { theme } from '$lib/design-system/theme.svelte';
 *   theme.setMode('light');   // o 'dark' | 'system'
 *   theme.resolved            // 'light' | 'dark' — el que está aplicado AHORA
 *
 * `ThemeToggle.svelte` ya es la UI de Ajustes para esto; no hace falta
 * releer este fichero para construir esa pantalla.
 */

export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const STORAGE_KEY = "networkbench.theme";

// TODO(integración real): sustituir el par localStorage.getItem/setItem de
// abajo por lectura/escritura de `settings.json` vía el comando Tauri
// correspondiente (§19.1 — Ajustes en %APPDATA%\NetworkBench\settings.json).
// localStorage es válido como placeholder porque el resto del sistema de
// diseño no depende de dónde se guarde la preferencia, solo de que
// `ThemeController.mode` acabe con el valor correcto al arrancar.

function readStoredMode(): ThemeMode {
  if (typeof window === "undefined") return "system";
  const stored = window.localStorage.getItem(STORAGE_KEY);
  return stored === "light" || stored === "dark" || stored === "system" ? stored : "system";
}

function systemPrefersLight(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-color-scheme: light)").matches;
}

class ThemeController {
  mode = $state<ThemeMode>(readStoredMode());
  #systemIsLight = $state(systemPrefersLight());

  resolved: ResolvedTheme = $derived(
    this.mode === "system" ? (this.#systemIsLight ? "light" : "dark") : this.mode
  );

  constructor() {
    if (typeof window === "undefined") return;

    const mql = window.matchMedia("(prefers-color-scheme: light)");
    mql.addEventListener("change", (e) => {
      this.#systemIsLight = e.matches;
    });

    // Aplica data-theme en <html> cada vez que `resolved` cambia — nunca se
    // actualiza a mano desde fuera de esta clase.
    $effect.root(() => {
      $effect(() => {
        document.documentElement.setAttribute("data-theme", this.resolved);
      });
    });
  }

  setMode(mode: ThemeMode) {
    this.mode = mode;
    if (typeof window !== "undefined") {
      window.localStorage.setItem(STORAGE_KEY, mode);
    }
  }
}

/** Instancia única compartida por toda la app. */
export const theme = new ThemeController();
