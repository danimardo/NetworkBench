/**
 * jsdom no implementa la Web Animations API (`Element.prototype.animate`). Svelte 5 la usa
 * internamente para `in:`/`out:` (p. ej. `fly`/`fade` de `svelte/transition`, usados en el
 * cambio de sección de `App.svelte`): sin este polyfill, cualquier test que dispare una
 * transición real (navegar de ruta, montar/desmontar un bloque con `in:`/`out:`) revienta con
 * `TypeError: element.animate is not a function`, aunque la lógica bajo prueba no tenga nada
 * que ver con la animación.
 *
 * El polyfill no reproduce la animación (jsdom no pinta nada): solo ofrece la forma mínima
 * que Svelte necesita — un objeto con `onfinish`/`oncancel`, `cancel()`, `finish()` y
 * `effect.getTiming().duration` — y termina la animación en la siguiente vuelta del bucle de
 * eventos, para que el código que espera a que "acabe" no se quede colgado.
 */
if (typeof Element !== "undefined" && !Element.prototype.animate) {
  Element.prototype.animate = function (
    _keyframes: unknown,
    options?: number | KeyframeAnimationOptions,
  ): Animation {
    const duration = typeof options === "number" ? options : (options?.duration ?? 0);
    const target: Partial<Animation> & { [key: string]: unknown } = {
      effect: { getTiming: () => ({ duration }) } as unknown as AnimationEffect,
      onfinish: null,
      oncancel: null,
      cancel() {},
      finish() {},
      play() {},
      pause() {},
    };
    // Siguiente vuelta del bucle de eventos: igual que una animación real, no termina en el
    // mismo tick en que se crea (Svelte espera exactamente ese orden).
    setTimeout(() => (target.onfinish as (() => void) | null)?.(), 0);
    return target as unknown as Animation;
  };
}
