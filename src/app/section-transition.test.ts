import { describe, it, expect } from "vitest";
import { sectionEnterParams, sectionExitParams } from "./section-transition";

// Parámetros puros de la transición de sección (Inicio/Historial/Ajustes/…). Se prueban
// aparte de App.svelte porque `svelte/transition` necesita un navegador real
// (`requestAnimationFrame`) para reproducirse; lo que sí se puede fijar sin uno es qué
// parámetros calcula la app según "Reducir movimiento".

describe("sectionEnterParams", () => {
  it("es un zoom marcado (parte de menos del tamaño real) cuando no se reduce el movimiento", () => {
    const p = sectionEnterParams(false);
    expect(p.start).toBeLessThan(1);
    // "Zoom más marcado", no un ajuste casi imperceptible: bien por debajo del 95 %.
    expect(p.start).toBeLessThanOrEqual(0.9);
    expect(p.duration).toBeGreaterThan(0);
    expect(p.easing(0)).toBe(0);
    expect(p.easing(1)).toBe(1);
  });

  it("la curva es ease-in-out: arranca despacio (sin tirón) y también termina despacio", () => {
    const p = sectionEnterParams(false);
    // Simétrica en el punto medio, a diferencia de una ease-out pura.
    expect(p.easing(0.5)).toBeCloseTo(0.5, 5);
    // Arranque lento: a los primeros instantes apenas ha avanzado (nada de tirón inicial).
    expect(p.easing(0.1)).toBeLessThan(0.1);
    // Tramo final igual de lento: cerca del final ya casi ha terminado.
    expect(p.easing(0.9)).toBeGreaterThan(0.9);
  });

  it("es instantánea con reduceMotion: sin zoom ni duración", () => {
    const p = sectionEnterParams(true);
    expect(p.start).toBe(1);
    expect(p.duration).toBe(0);
    // El easing sigue siendo una función válida (svelte/transition la llama igual);
    // con duración 0 no llega a importar, pero no debe lanzar ni deformar el valor.
    expect(p.easing(0.5)).toBe(0.5);
  });
});

describe("sectionExitParams", () => {
  it("usa la misma curva ease-in-out que la entrada, con una duración próxima a la suya", () => {
    const salida = sectionExitParams(false);
    const entrada = sectionEnterParams(false);
    expect(salida.duration).toBeGreaterThan(0);
    // Antes la salida (120ms) era mucho más corta que la entrada (300ms): el contenido
    // antiguo desaparecía bastante antes de que el nuevo terminara de crecer. Ahora deben
    // quedar cerca (como mucho, un 40 % de diferencia respecto a la entrada).
    expect(Math.abs(salida.duration - entrada.duration)).toBeLessThanOrEqual(
      entrada.duration * 0.4,
    );
    expect(salida.easing(0.5)).toBeCloseTo(0.5, 5);
  });

  it("es instantánea con reduceMotion", () => {
    const p = sectionExitParams(true);
    expect(p.duration).toBe(0);
    expect(p.easing(0.5)).toBe(0.5);
  });
});
