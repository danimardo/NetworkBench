import { describe, it, expect } from "vitest";
import { sampleBatchSchema, samplePointSchema } from "../../src/lib/api/samples";

// Contrato del evento `session://sample-batch` (T150). El backend construye este
// payload en `src-tauri/src/sampling/aggregate.rs::SampleBatch` (serde camelCase); este
// test fija del lado TypeScript la misma forma que `ipc::events::tests` fija del lado
// Rust, para que un cambio de nombre de campo en cualquiera de los dos lados falle en
// alguno de los dos sitios, no en producción.
describe("Contrato del evento session://sample-batch (T150)", () => {
  const loteValido = {
    sessionId: "b6f7a3d0-1111-4c2a-9a2e-000000000001",
    direction: "forward",
    samples: [
      { tMs: 0, direction: "forward", bps: 940_000_000, cpuPercent: 12.5, gap: false },
      { tMs: 260, direction: "forward", bps: 935_000_000, cpuPercent: null, gap: false },
    ],
    latestBps: 935_000_000,
    latestCpuPercent: null,
    hasGaps: false,
  };

  it("acepta un lote con la forma real que emite el backend", () => {
    const resultado = sampleBatchSchema.safeParse(loteValido);
    expect(resultado.success).toBe(true);
  });

  it("acepta una muestra individual con cpuPercent ausente", () => {
    const resultado = samplePointSchema.safeParse({
      tMs: 1000,
      direction: "reverse",
      bps: 0,
      gap: true,
    });
    expect(resultado.success).toBe(true);
  });

  it("rechaza un lote al que le falta un campo obligatorio (sessionId)", () => {
    const sinSessionId: Record<string, unknown> = { ...loteValido };
    delete sinSessionId.sessionId;
    const resultado = sampleBatchSchema.safeParse(sinSessionId);
    expect(resultado.success).toBe(false);
  });

  it("rechaza un lote con un nombre de campo distinto al esperado", () => {
    // Simula el mismo error que corrigió T160: un campo renombrado en un extremo
    // (aquí, `latest_bps` en vez de `latestBps`) debe fallar la validación, no colarse.
    const conNombreEquivocado: Record<string, unknown> = { ...loteValido };
    delete conNombreEquivocado.latestBps;
    conNombreEquivocado.latest_bps = loteValido.latestBps;
    const resultado = sampleBatchSchema.safeParse(conNombreEquivocado);
    expect(resultado.success).toBe(false);
  });
});
