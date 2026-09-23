#!/usr/bin/env node
import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve } from "node:path";

function run(command, desc) {
  console.log(`\n── [verify-app] ${desc}`);
  console.log(`$ ${command}`);
  try {
    execSync(command, { stdio: "inherit", shell: true });
    console.log(`✓ ${desc} superado.`);
  } catch (err) {
    console.error(`✗ ERROR: ${desc} falló.`);
    process.exit(1);
  }
}

console.log("=== Comprobación canónica de NetworkBench (pnpm verify) ===");

// 1. Verificación de tipos y Svelte (requerido previo a test/build)
run("pnpm check", "Tipos y componentes (pnpm check)");

// 2. Verificación de formato
run("pnpm format", "Formato Prettier (pnpm format)");

// 3. Linting
run("pnpm lint", "Linting ESLint (pnpm lint)");

// 4. Verificación de Rust (constitución §CI y Definition of Done, punto 2)
run(
  "cargo fmt --manifest-path src-tauri/Cargo.toml --check",
  "Formato de Rust (cargo fmt --check)",
);
run(
  "cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings",
  "Clippy con warnings como errores",
);
run("cargo check --manifest-path src-tauri/Cargo.toml", "Compilación de backend Rust");

// 5. Tests unitarios
run("pnpm test:unit", "Tests unitarios frontend (Vitest)");

// 6. Comprobaciones de arquitectura y convenciones
run("node scripts/architecture/check-locales.mjs", "Paridad de claves locales (es/en)");
run("node scripts/architecture/check-imports.mjs", "Límites e imports de arquitectura");
run("node scripts/architecture/check-logger.mjs", "Uso de logger único sin console.*");

console.log("\n========================================================");
console.log("✓ Verificación canónica completada con éxito.");
console.log("========================================================\n");
