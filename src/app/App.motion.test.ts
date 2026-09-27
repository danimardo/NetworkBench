import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

// El contenedor de cada pantalla (`.nb-screen-host`) usa `in:scale` para el cambio de
// sección del sidebar. Con el `transform-origin` por defecto (el centro del propio
// contenido, no de la ventana visible), en una pantalla más alta que el viewport la
// cabecera y las primeras tarjetas —por encima de ese centro— se desplazan hacia arriba
// durante el zoom, lo que se percibe como "sube desde abajo" (hallazgo real del
// propietario) aunque sea un escalado puro, sin ningún translateY. Anclado arriba, la
// cabecera queda fija y solo crece el contenido hacia abajo.
//
// jsdom no calcula el `transform-origin` real de una hoja de estilos (`<style>` scoped de
// Svelte incluido): igual que tokens.motion.test.ts, esto se comprueba sobre el propio
// texto fuente en vez de montar el componente y leer `getComputedStyle`.

const ruta = resolve(process.cwd(), "src/app/App.svelte");
const fuente = readFileSync(ruta, "utf8");

describe("App.svelte: origen del zoom de sección", () => {
  it("ancla `.nb-screen-host` arriba, no en el centro del contenido", () => {
    const bloque = /\.nb-screen-host\s*\{([^]*?)\}/.exec(fuente);
    expect(bloque, "debe existir una regla para .nb-screen-host en <style>").not.toBeNull();
    expect(bloque![1]).toMatch(/transform-origin:\s*top\b/);
  });
});
