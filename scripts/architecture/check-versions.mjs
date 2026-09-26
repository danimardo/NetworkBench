#!/usr/bin/env node
/**
 * Comprueba que la versión de la aplicación sea la misma en todos los sitios donde vive.
 *
 * `package.json` es la fuente (`.agents/rules/proyecto/versiones.md`) y `tauri.conf.json` y
 * `Cargo.toml` la repiten: la pantalla «Acerca de», el snapshot y el handshake leen la de
 * Cargo (`env!("CARGO_PKG_VERSION")`), así que si una se queda atrás la aplicación muestra
 * una versión y el instalador lleva otra. Al subir de versión hay que tocar las tres a la
 * vez; este check falla si alguna no coincide.
 *
 * No comprueba que el número sea el «correcto»: eso lo decide quien publica.
 */
import { readFileSync } from "node:fs";

function leer(ruta) {
  return readFileSync(ruta, "utf8");
}

const fuentes = [];

const paquete = JSON.parse(leer("package.json"));
fuentes.push(["package.json", paquete.version]);

const tauri = JSON.parse(leer("src-tauri/tauri.conf.json"));
fuentes.push(["src-tauri/tauri.conf.json", tauri.version]);

const cargo = /\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m.exec(leer("src-tauri/Cargo.toml"));
fuentes.push(["src-tauri/Cargo.toml", cargo?.[1]]);

// Cargo.lock repite la versión del propio crate; si quedara atrás, Cargo la reescribe al
// compilar y el árbol de trabajo aparecería modificado sin que nadie lo hubiera tocado.
const nombre = /\[package\][\s\S]*?^name\s*=\s*"([^"]+)"/m.exec(leer("src-tauri/Cargo.toml"))?.[1];
const bloqueLock = new RegExp(
  `\\[\\[package\\]\\]\\s*name = "${nombre}"\\s*version = "([^"]+)"`,
).exec(leer("src-tauri/Cargo.lock"));
fuentes.push(["src-tauri/Cargo.lock", bloqueLock?.[1]]);

const faltan = fuentes.filter(([, v]) => !v);
if (faltan.length > 0) {
  console.error(`✗ No se pudo leer la versión de: ${faltan.map(([f]) => f).join(", ")}`);
  process.exit(1);
}

const distintas = new Set(fuentes.map(([, v]) => v));
if (distintas.size > 1) {
  console.error("✗ La versión de la aplicación no coincide entre ficheros:");
  for (const [fichero, version] of fuentes) console.error(`  ${version}  ${fichero}`);
  console.error("\nSúbelas todas a la vez (package.json es la fuente) y vuelve a ejecutar.");
  process.exit(1);
}

console.log(`✓ Versión coherente en ${fuentes.length} ficheros: ${[...distintas][0]}.`);
