import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

// `tokens.css` es CSS global, no un componente: jsdom no calcula animaciones reales de una
// hoja de estilos externa, así que estas comprobaciones son sobre el propio texto fuente,
// igual que ya hacen otros scripts del proyecto (p. ej. verify-tokens.mjs). Lo que importa
// aquí es que las piezas necesarias existan y no se pierdan en un refactor futuro: los
// keyframes, las utilidades y las dos vías de "reducir movimiento" (la del sistema operativo
// y la de la propia app).

const ruta = resolve(process.cwd(), "src/lib/design-system/tokens.css");
const css = readFileSync(ruta, "utf8");

describe("tokens.css: entrada escalonada y pop (ThrottleWatch → NetworkBench)", () => {
  it("define los keyframes y las utilidades", () => {
    expect(css).toContain("@keyframes nb-zoom-in");
    expect(css).toContain("@keyframes nb-pop");
    expect(css).toMatch(/\.nb-enter\s*\{[^}]*animation:\s*nb-zoom-in/);
    expect(css).toMatch(/\.nb-pop\s*\{[^}]*animation:\s*nb-pop/);
  });

  it("la entrada es un zoom centrado, sin desplazamiento vertical (antes subía desde abajo)", () => {
    const bloque = /@keyframes nb-zoom-in\s*\{([^]*?)\}\s*\n\s*\n/.exec(css);
    expect(bloque, "debe existir el bloque @keyframes nb-zoom-in").not.toBeNull();
    const cuerpo = bloque![1];
    expect(cuerpo).not.toContain("translateY");
    expect(cuerpo).toContain("scale(0.92)");
  });

  it("usa su propia duración y curva, no --duration-slow/--ease-standard compartidas con hover/foco", () => {
    expect(css).toMatch(/--duration-enter:\s*\d+ms/);
    expect(css).toMatch(/--ease-enter:\s*cubic-bezier/);
    expect(css).toMatch(/\.nb-enter\s*\{[^}]*var\(--duration-enter\)[^}]*var\(--ease-enter\)/);
  });

  it("la entrada escalonada usa --nb-i con un tope, para que una lista larga no tarde eones", () => {
    expect(css).toMatch(/--stagger-max-index:\s*8/);
    expect(css).toMatch(
      /animation-delay:\s*calc\(min\(var\(--nb-i,\s*0\),\s*var\(--stagger-max-index\)\)/,
    );
  });

  it("colapsa a instantáneo con prefers-reduced-motion del sistema", () => {
    const bloque =
      /@media \(prefers-reduced-motion: reduce\) \{[^}]*\.nb-enter[^}]*\.nb-pop[^}]*\}/s.exec(css);
    expect(bloque, "debe existir un bloque @media que colapse .nb-enter y .nb-pop").not.toBeNull();
  });

  it("colapsa a instantáneo con el ajuste propio de la app (data-reduce-motion)", () => {
    expect(css).toContain(':root[data-reduce-motion="true"] .nb-enter');
    expect(css).toContain(':root[data-reduce-motion="true"] .nb-pop');
  });

  it("el easing con rebote existe como token, no como valor suelto en el componente", () => {
    expect(css).toMatch(/--ease-spring:\s*cubic-bezier/);
  });
});
