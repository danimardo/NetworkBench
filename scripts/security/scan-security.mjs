import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

console.log("=== Escaneo de Seguridad, Secretos y Licencias (T131) ===\n");

let failed = false;

// 1. Escaneo de secretos en el árbol de código fuente
const SECRET_PATTERNS = [
  /-----BEGIN\s+PRIVATE\s+KEY-----/i,
  /-----BEGIN\s+RSA\s+PRIVATE\s+KEY-----/i,
  /-----BEGIN\s+EC\s+PRIVATE\s+KEY-----/i,
  /ghp_[a-zA-Z0-9]{36}/,
  /AKIA[0-9A-Z]{16}/,
  /password\s*[:=]\s*["'][^"']{6,}["']/i,
];

const SCAN_DIRS = ["src", "src-tauri/src", "tests", "scripts"];
const IGNORED_EXTS = [".png", ".jpg", ".ico", ".exe", ".db", ".lock"];

function scanDir(dir) {
  if (!fs.existsSync(dir)) return;
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name !== "node_modules" && entry.name !== "target" && entry.name !== ".git") {
        scanDir(full);
      }
    } else if (entry.isFile()) {
      const ext = path.extname(entry.name);
      if (IGNORED_EXTS.includes(ext)) continue;

      const content = fs.readFileSync(full, "utf8");
      for (const pattern of SECRET_PATTERNS) {
        if (pattern.test(content)) {
          // Excluir menciones explícitas en reglas de saneamiento o tests de sanitización
          if (
            content.includes("sanitize_param_key") ||
            full.includes("redact") ||
            full.includes("scan-security")
          ) {
            continue;
          }
          console.error(`✗ Posible secreto detectado en: ${full}`);
          failed = true;
        }
      }
    }
  }
}

for (const d of SCAN_DIRS) {
  scanDir(d);
}

if (!failed) {
  console.log("✓ Escaneo de secretos: ningún secreto hardcodeado detectado en el código fuente.");
}

// 2. Verificación de licencias en THIRD_PARTY_NOTICES.md
const noticesPath = "THIRD_PARTY_NOTICES.md";
if (!fs.existsSync(noticesPath)) {
  console.error("✗ THIRD_PARTY_NOTICES.md no existe.");
  failed = true;
} else {
  const notices = fs.readFileSync(noticesPath, "utf8");
  const requiredNotices = [
    "Microsoft NTTTCP",
    "Rust Dependencies",
    "Frontend & Build Dependencies",
  ];
  for (const r of requiredNotices) {
    if (!notices.includes(r)) {
      console.error(`✗ THIRD_PARTY_NOTICES.md no contiene la sección: ${r}`);
      failed = true;
    }
  }
  if (!failed) {
    console.log("✓ Atribuciones de terceros y licencias verificadas en THIRD_PARTY_NOTICES.md.");
  }
}

// 3. Inventario de dependencias directas
function contarDependencias() {
  const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
  const js =
    Object.keys(pkg.dependencies ?? {}).length + Object.keys(pkg.devDependencies ?? {}).length;

  const cargo = fs.readFileSync("src-tauri/Cargo.toml", "utf8");
  const bloque = cargo.split(/^\[dependencies\]$/m)[1] ?? "";
  const soloDeps = bloque.split(/^\[/m)[0] ?? "";
  const rust = soloDeps.split("\n").filter((l) => /^[a-zA-Z0-9_-]+\s*=/.test(l.trim())).length;

  return { js, rust };
}

const inventario = contarDependencias();
console.log(
  `\n✓ Inventario de dependencias directas: ${inventario.js} JS/TS, ${inventario.rust} Rust.`,
);

// 4. Escaneo de vulnerabilidades conocidas
//
// Criterio: una vulnerabilidad es bloqueante; un aviso (crate sin mantenimiento,
// unsoundness) se informa y no detiene la verificación. Cambiar ese criterio exige
// decisión explícita, no un ajuste silencioso del umbral.
function auditar(comando, desc) {
  console.log(`\n── ${desc}`);
  try {
    // 2>&1: cargo audit emite el recuento de avisos por stderr, y en la rama de éxito
    // execSync solo devuelve stdout. Sin esto el recuento se perdería en silencio.
    const salida = execSync(`${comando} 2>&1`, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    console.log(salida.trim().split("\n").slice(-3).join("\n"));
    return { ok: true, salida };
  } catch (err) {
    const salida = `${err.stdout ?? ""}${err.stderr ?? ""}`;
    console.error(salida.trim());
    return { ok: false, salida };
  }
}

const auditRust = auditar("cargo audit --file src-tauri/Cargo.lock", "cargo audit (Rust)");
const avisosRust = /(\d+) allowed warnings found/.exec(auditRust.salida);
if (avisosRust) {
  console.log(`  aviso: ${avisosRust[1]} crates sin mantenimiento o unsound (no bloqueante).`);
}
if (!auditRust.ok) {
  console.error("✗ cargo audit encontró vulnerabilidades.");
  failed = true;
}

const auditJs = auditar("pnpm audit", "pnpm audit (JS/TS)");
if (!auditJs.ok) {
  console.error("✗ pnpm audit encontró vulnerabilidades.");
  failed = true;
}

if (failed) {
  process.exit(1);
} else {
  console.log("\n✓ Auditoría de seguridad superada sin vulnerabilidades conocidas.");
}
