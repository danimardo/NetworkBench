/**
 * Parámetros de la transición al cambiar de sección (Inicio/Historial/Ajustes/…),
 * separados de `App.svelte` para poder probarlos sin montar un componente ni depender
 * de `requestAnimationFrame` (que `svelte/transition` necesita en un navegador real).
 *
 * Es un zoom: crece desde más pequeño mientras se desvanece (sin desplazamiento vertical),
 * con `svelte/transition`'s `scale` en la entrada y un fundido en la salida. Antes subía
 * desde abajo (`fly`); las tarjetas de dentro tienen su propio `.nb-enter` (también una
 * subida, en tokens.css) y los dos movimientos verticales a la vez daban una sensación de
 * "salto" (hallazgo real del propietario). El zoom no se solapa con esa subida: un
 * movimiento es de escala, el otro de posición.
 *
 * La curva es ease-in-out (arranca y termina despacio, acelera en el medio), no ease-out
 * puro: una cúbica ease-out arranca a máxima velocidad desde el primer fotograma, lo que
 * el propietario notó como un tirón brusco al cambiar de sección. La salida (fundido) usa
 * la misma curva y una duración más próxima a la de entrada — con 120ms de salida frente a
 * 300ms de entrada, el contenido antiguo desaparecía mucho antes de que el nuevo terminara
 * de crecer, y ese descompás también se sentía brusco.
 *
 * Instantáneo con `reduceMotion` — igual que `.nb-enter`/`.nb-pop` de tokens.css, pero esto
 * no es CSS: son los parámetros que lee `in:scale`/`out:fade`, así que la app decide la
 * duración en el momento en vez de con un selector.
 */

export interface EnterParams {
  /** Escala de partida (1 = tamaño normal, sin zoom). */
  start: number;
  duration: number;
  easing: (t: number) => number;
}

export interface ExitParams {
  duration: number;
  easing: (t: number) => number;
}

/**
 * Cúbica ease-in-out (equivalente a un "smoothstep" cúbico): despacio al empezar y al
 * terminar, más rápida en el tramo central. Sin depender de `svelte/easing`.
 */
function easeInOutCubic(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

export function sectionEnterParams(reduceMotion: boolean): EnterParams {
  if (reduceMotion) return { start: 1, duration: 0, easing: (t) => t };
  // "Zoom más marcado" (decisión del propietario): parte del 90 % de su tamaño, no del 96-98 %
  // casi imperceptible de un simple ajuste de escala.
  return { start: 0.9, duration: 360, easing: easeInOutCubic };
}

export function sectionExitParams(reduceMotion: boolean): ExitParams {
  if (reduceMotion) return { duration: 0, easing: (t) => t };
  return { duration: 220, easing: easeInOutCubic };
}
