#!/usr/bin/env node
//
// Prepara los binarios que el instalador debe llevar junto a la aplicación.
//
// Tauri llama «sidecar» a un ejecutable que se copia al lado del binario principal.
// Los espera en `src-tauri/binaries/<nombre>-<target-triple>.exe` y los instala sin el
// sufijo, de modo que en tiempo de ejecución quedan como `ntttcp.exe` y
// `networkbench-firewall-helper.exe` en el mismo directorio que la aplicación. Eso es
// justo lo que `app::init` y `firewall::helper_client` esperan encontrar.
//
// `src-tauri/binaries/` no se versiona: se genera aquí. El motor sí está en el
// repositorio, en `engine/`, con su licencia, su versión y su hash.

import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const TRIPLE = "x86_64-pc-windows-msvc";
const DESTINO = "src-tauri/binaries";

const MOTOR = "engine/ntttcp.exe";
const HASH_MOTOR = "engine/SHA256";
const HELPER = `src-tauri/target/${TRIPLE}/release/networkbench-firewall-helper.exe`;

let fallos = 0;

function error(mensaje) {
  console.error(`✗ ${mensaje}`);
  fallos++;
}

function sha256(ruta) {
  return createHash("sha256").update(fs.readFileSync(ruta)).digest("hex");
}

function copiar(origen, nombreDestino) {
  const destino = path.join(DESTINO, nombreDestino);
  fs.copyFileSync(origen, destino);
  const tam = fs.statSync(destino).size;
  console.log(`✓ ${origen} → ${destino} (${tam} bytes)`);
}

console.log("=== Preparación de binarios para el instalador ===\n");

fs.mkdirSync(DESTINO, { recursive: true });

// 1. Motor de medida, con su integridad comprobada antes de empaquetarlo.
//
// Verificar aquí evita que un binario alterado entre en el instalador. La aplicación
// lo vuelve a comprobar en cada ejecución (FR-063): son dos controles distintos, no
// una redundancia.
if (!fs.existsSync(MOTOR)) {
  error(`${MOTOR} NO PRESENTE. Sin motor no hay release: la puerta G1 sigue abierta.`);
} else {
  const esperado = fs.readFileSync(HASH_MOTOR, "utf8").trim();
  const real = sha256(MOTOR);
  if (real !== esperado) {
    error(`El SHA-256 de ${MOTOR} no coincide.\n    esperado ${esperado}\n    obtenido ${real}`);
  } else {
    console.log(`✓ Integridad del motor verificada (${real})`);
    copiar(MOTOR, `ntttcp-${TRIPLE}.exe`);
  }
}

// 2. Helper elevado de cortafuegos, segundo binario del mismo crate.
//
// Va como recurso y no como sidecar a propósito. `tauri-build` valida los sidecars
// durante la compilación del crate, y el helper *es* ese crate: exigirlo como sidecar
// crea una dependencia circular en la que nada puede compilarse la primera vez. Como
// recurso se resuelve al empaquetar, cuando el binario ya existe.
const soloMotor = process.argv.includes("--solo-motor");
if (soloMotor) {
  console.log("ℹ Solo motor: el helper se prepara tras compilar el crate.");
} else if (!fs.existsSync(HELPER)) {
  error(
    `${HELPER} NO PRESENTE.\n` +
      `    Compílalo antes: cargo build --manifest-path src-tauri/Cargo.toml --release ` +
      `--target ${TRIPLE} --bin networkbench-firewall-helper\n` +
      `    En la primera pasada usa: node scripts/package/prepare-bundle.mjs --solo-motor`,
  );
} else {
  copiar(HELPER, "networkbench-firewall-helper.exe");
}

if (fallos > 0) {
  console.error(`\n✗ ${fallos} problema(s). El instalador NO debe construirse así.`);
  process.exit(1);
}

console.log("\n✓ Binarios preparados. Ya se puede ejecutar el empaquetado de Tauri.");
