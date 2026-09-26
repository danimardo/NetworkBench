import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import type { Page } from "@playwright/test";

// `@axe-core/playwright` no está entre las dependencias del proyecto y no se ha añadido
// una nueva sin autorización (`.agents/rules/universal/seguridad.md`): `axe-core` ya
// estaba declarado como devDependency y sin usar en ningún sitio. Se inyecta su bundle
// directamente en la página, igual que haría el wrapper por dentro.
const require = createRequire(import.meta.url);
const AXE_SOURCE = readFileSync(require.resolve("axe-core/axe.min.js"), "utf-8");

export interface AxeViolation {
  id: string;
  impact: string | null;
  help: string;
  helpUrl: string;
  nodes: { target: string[]; failureSummary: string | null }[];
}

/** Ejecuta axe-core sobre la página actual y devuelve solo las violaciones. */
export async function runAxe(page: Page): Promise<AxeViolation[]> {
  await page.addScriptTag({ content: AXE_SOURCE });
  const resultado = await page.evaluate(async () => {
    // @ts-expect-error -- axe se inyecta en `window` por el script anterior, sin tipos.
    const salida = await window.axe.run(document, {
      resultTypes: ["violations"],
    });
    return salida.violations;
  });
  return resultado as AxeViolation[];
}

/** Mensaje legible para un fallo de test, con el nodo y el motivo de cada violación. */
export function describeViolations(violaciones: AxeViolation[]): string {
  return violaciones
    .map((v) => {
      const nodos = v.nodes.map((n) => `  · ${n.target.join(" ")}: ${n.failureSummary}`).join("\n");
      return `[${v.impact}] ${v.id} — ${v.help} (${v.helpUrl})\n${nodos}`;
    })
    .join("\n\n");
}
