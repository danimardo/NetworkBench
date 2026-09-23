#!/usr/bin/env node
//
// Gate de cobertura de Rust según la decisión Q2 (constitución 0.7.0).
//
//   - 80 % de líneas en el total del crate.
//   - 90 % de líneas en cada módulo crítico.
//
// No promedia lenguajes ni métricas, y no hay umbral intermedio: un porcentaje
// inferior no se interpreta como aprobación. Hoy este gate falla a propósito;
// las cifras de la baseline están en VALIDACION.md.

import { execSync } from "node:child_process";
import fs from "node:fs";

const UMBRAL_TOTAL = 80;
const UMBRAL_CRITICO = 90;

// Módulos críticos según la constitución §VIII: estados y control, emparejamiento,
// planes, parser del motor, diagnóstico y saneamiento.
const CRITICOS = [
  "src/control/cleanup.rs",
  "src/control/domain.rs",
  "src/control/plan.rs",
  "src/control/ports.rs",
  "src/control/preflight.rs",
  "src/control/repeat.rs",
  "src/control/service.rs",
  "src/control/transport.rs",
  "src/diagnostic/capacity.rs",
  "src/diagnostic/rules.rs",
  "src/diagnostic/udp.rs",
  "src/engine/ntttcp/parser.rs",
  "src/export/redact.rs",
  "src/logging/diagnostics.rs",
  "src/model/plan.rs",
  "src/netinfo/resolve.rs",
  "src/pairing/mod.rs",
];

const SALIDA = "artifacts/coverage/rust/llvm-cov.json";

console.log("=== Cobertura de Rust (Q2: 80 % total, 90 % en módulos críticos) ===\n");

fs.mkdirSync("artifacts/coverage/rust", { recursive: true });

execSync(
  `cargo llvm-cov --manifest-path src-tauri/Cargo.toml --all-targets --json --output-path ${SALIDA}`,
  { stdio: ["ignore", "ignore", "inherit"] },
);

const informe = JSON.parse(fs.readFileSync(SALIDA, "utf8"));
const datos = informe.data[0];

const porcentaje = (r) => (r.count === 0 ? 100 : (r.covered / r.count) * 100);
const normaliza = (p) => p.replace(/\\/g, "/").replace(/^.*\/src-tauri\//, "");

let fallos = 0;

const total = porcentaje(datos.totals.lines);
const okTotal = total >= UMBRAL_TOTAL;
if (!okTotal) fallos++;
console.log(
  `${okTotal ? "✓" : "✗"} Total del crate: ${total.toFixed(2)} % de líneas (mínimo ${UMBRAL_TOTAL} %)`,
);

console.log(`\n── Módulos críticos (mínimo ${UMBRAL_CRITICO} % de líneas)`);
const medidos = new Map(
  datos.files.map((f) => [normaliza(f.filename), porcentaje(f.summary.lines)]),
);

for (const modulo of CRITICOS) {
  const pct = medidos.get(modulo);
  if (pct === undefined) {
    console.error(`  ✗ ${modulo}: sin datos de cobertura — ¿se ha movido o renombrado?`);
    fallos++;
    continue;
  }
  const ok = pct >= UMBRAL_CRITICO;
  if (!ok) fallos++;
  console.log(`  ${ok ? "✓" : "✗"} ${modulo}: ${pct.toFixed(2)} %`);
}

if (fallos > 0) {
  console.error(`\n✗ ${fallos} incumplimiento(s) del umbral Q2. Informe completo: ${SALIDA}`);
  process.exit(1);
}

console.log("\n✓ Cobertura de Rust conforme a Q2.");
