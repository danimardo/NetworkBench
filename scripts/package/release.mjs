#!/usr/bin/env node
//
// Construye el instalador de NetworkBench en dos fases.
//
// Por qué dos fases: `tauri-build` valida los recursos declarados en `tauri.conf.json`
// DURANTE la compilación del crate, y el helper elevado de cortafuegos es un segundo
// binario de ese mismo crate. En un clon limpio eso es circular —el crate no compila
// porque falta un recurso que solo existe tras compilar el crate—.
//
// La fase 1 compila el helper con el recurso apartado; la fase 2 lo coloca y construye
// el instalador con la configuración íntegra. El desarrollador ejecuta un solo comando.
//
// Alternativa descartada: mover el helper a un crate propio. Lo resolvería de raíz,
// pero contradice ADR-002 («un único crate src-tauri con dos binarios, sin workspace
// multi-crate») y esa enmienda no se ha tramitado.
//
//   node scripts/package/release.mjs
//
// NO publica nada: no crea tags, no sube artefactos y no firma. Solo deja el
// instalador en src-tauri/target/<triple>/release/bundle/nsis/.

import { execFileSync } from "node:child_process";
import fs from "node:fs";

const TRIPLE = "x86_64-pc-windows-msvc";
const CONF = "src-tauri/tauri.conf.json";
const RECURSO_HELPER = "resources/networkbench-firewall-helper.exe";
const HELPER_COMPILADO = `src-tauri/target/${TRIPLE}/release/networkbench-firewall-helper.exe`;

function paso(n, texto) {
  console.log(`\n── Fase ${n}: ${texto}`);
}

function ejecutar(cmd, args) {
  const linea = `${cmd} ${args.join(" ")}`;
  console.log(`$ ${linea}`);
  // `shell: true` hace falta en Windows para resolver `pnpm` y `cargo`, que son
  // envoltorios .cmd. Los argumentos son literales de este fichero, no entrada externa.
  execFileSync(linea, { stdio: "inherit", shell: true });
}

const confOriginal = fs.readFileSync(CONF, "utf8");
let confRestaurada = false;

/** Devuelve `tauri.conf.json` a su estado original pase lo que pase. */
function restaurar() {
  if (!confRestaurada) {
    fs.writeFileSync(CONF, confOriginal);
    confRestaurada = true;
  }
}
process.on("exit", restaurar);
process.on("SIGINT", () => {
  restaurar();
  process.exit(130);
});

try {
  console.log("=== Construcción del instalador de NetworkBench ===");

  paso(1, "compilar el helper elevado con el recurso apartado");
  const sinHelper = JSON.parse(confOriginal);
  sinHelper.bundle.resources = sinHelper.bundle.resources.filter((r) => r !== RECURSO_HELPER);
  if (sinHelper.bundle.resources.length === confOriginal.split(RECURSO_HELPER).length - 1) {
    console.warn(`  aviso: ${RECURSO_HELPER} no estaba declarado; revisa ${CONF}`);
  }
  fs.writeFileSync(CONF, `${JSON.stringify(sinHelper, null, 2)}\n`);

  // El motor debe estar antes de cualquier compilación: es un sidecar declarado.
  ejecutar("node", ["scripts/package/prepare-bundle.mjs", "--solo-motor"]);
  ejecutar("cargo", [
    "build",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--release",
    "--target",
    TRIPLE,
    "--bin",
    "networkbench-firewall-helper",
  ]);

  if (!fs.existsSync(HELPER_COMPILADO)) {
    throw new Error(`La fase 1 no produjo ${HELPER_COMPILADO}`);
  }

  paso(2, "colocar los binarios y construir el instalador");
  restaurar();
  ejecutar("node", ["scripts/package/prepare-bundle.mjs"]);
  ejecutar("pnpm", ["tauri", "build"]);

  const bundle = `src-tauri/target/${TRIPLE}/release/bundle/nsis`;
  const instaladores = fs.existsSync(bundle)
    ? fs.readdirSync(bundle).filter((f) => f.endsWith(".exe"))
    : [];

  if (instaladores.length === 0) {
    throw new Error(`No se generó ningún instalador en ${bundle}`);
  }

  console.log("\n✓ Instalador generado:");
  for (const nombre of instaladores) {
    const ruta = `${bundle}/${nombre}`;
    const { size } = fs.statSync(ruta);
    console.log(`  ${ruta} (${(size / 1048576).toFixed(1)} MiB)`);
  }
  console.log("\nNo se ha firmado ni publicado nada. Eso requiere autorización aparte.");
} finally {
  restaurar();
}
