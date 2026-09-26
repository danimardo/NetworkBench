import type { SupportedLocale } from "./index";

/**
 * Idioma activo como estado reactivo de Svelte 5.
 *
 * Vive aparte de `index.ts` porque las runes (`$state`) solo se compilan en ficheros
 * `.svelte.ts`. `t()` lee `localeState.current`: cualquier plantilla o `$derived` que la
 * llame queda suscrita al idioma y se redibuja sola al cambiarlo, sin salir de la pantalla.
 * Con una variable normal `t()` devolvía el texto nuevo, pero nadie se enteraba.
 */
export const localeState = $state<{ current: SupportedLocale }>({ current: "es" });
