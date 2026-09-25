import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";
import { sessionResultSchema } from "../../src/lib/contracts/session-result";
import { benchmarkPlanSchema } from "../../src/lib/contracts/plan";
import { peerSchema } from "../../src/lib/contracts/peer";
import { appErrorSchema } from "../../src/lib/contracts/errors";
import { appSnapshotSchema } from "../../src/lib/api/snapshot.svelte";
import { sampleBatchSchema } from "../../src/lib/api/samples";

// Lo que Rust REALMENTE serializa (generado por `src-tauri/tests/contract_fixtures.rs`),
// parseado con los esquemas que usa el frontend. Regenerar con:
//   NB_UPDATE_FIXTURES=1 cargo test --manifest-path src-tauri/Cargo.toml --test contract_fixtures
function fixture(nombre: string): unknown {
  const ruta = join(__dirname, "fixtures", "rust", `${nombre}.json`);
  return JSON.parse(readFileSync(ruta, "utf-8"));
}

function comprobar(esquema: z.ZodType, nombre: string) {
  const resultado = esquema.safeParse(fixture(nombre));
  if (!resultado.success) {
    throw new Error(`${nombre}: ${JSON.stringify(resultado.error.issues, null, 2)}`);
  }
  return resultado.data;
}

describe("Contratos: el JSON emitido por Rust cumple los esquemas de TypeScript (T160)", () => {
  it("resultado de sesión TCP", () => {
    comprobar(sessionResultSchema, "session-result-tcp");
  });

  it("resultado de sesión UDP (pérdida en `retransmission`)", () => {
    comprobar(sessionResultSchema, "session-result-udp");
  });

  it("plan TCP y UDP", () => {
    comprobar(benchmarkPlanSchema, "plan-tcp");
    comprobar(benchmarkPlanSchema, "plan-udp");
  });

  it("equipos en cada estado de confianza", () => {
    comprobar(z.array(peerSchema), "peers");
  });

  it("instantánea de la aplicación", () => {
    comprobar(appSnapshotSchema, "app-snapshot");
  });

  it("lote de muestras", () => {
    comprobar(sampleBatchSchema, "sample-batch");
  });

  it("errores de aplicación", () => {
    comprobar(z.array(appErrorSchema), "app-errors");
  });

  it("los fixtures no están vacíos ni son triviales", () => {
    const tcp = fixture("session-result-tcp") as { directions?: unknown[] };
    expect(tcp.directions?.length).toBe(2);
  });
});
