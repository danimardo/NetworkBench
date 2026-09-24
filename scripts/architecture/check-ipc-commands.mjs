// Verifica que todo comando IPC invocado desde el frontend existe en el backend Rust.
//
// Nace de un fallo real detectado el 2026-09-24: `snapshot.svelte.ts` invocaba
// "app.getSnapshot", pero Tauri solo registra "app_get_snapshot" (el nombre exacto de
// la función anotada con #[tauri::command], sin transformación de mayúsculas ni
// separadores). El comando fallaba en cada arranque real de la aplicación, y ningún
// test lo detectaba: los tests de Vitest usan `setTransportMock`, que sustituye el
// transporte entero, así que un test podía mockear el mismo nombre incorrecto y pasar
// en verde sin comparar nunca contra lo que Rust registra de verdad.
//
// Este script sí compara contra la única fuente real: la lista de
// `tauri::generate_handler![...]` en src-tauri/src/lib.rs.

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const LIB_RS = "src-tauri/src/lib.rs";
const API_DIR = "src/lib/api";

function comandosRegistradosEnRust() {
  const contenido = readFileSync(LIB_RS, "utf-8");
  const bloque = contenido.match(/generate_handler!\s*\[([\s\S]*?)\]/);
  if (!bloque) {
    throw new Error(`No se encontró tauri::generate_handler![...] en ${LIB_RS}`);
  }
  // Cada entrada es `modulo::submodulo::...::funcion,` o solo `funcion,` cuando ya
  // está en el ámbito raíz. El nombre expuesto a JS es siempre el último segmento,
  // sin ningún prefijo de módulo — así es como Tauri la registra.
  const nombres = new Set();
  for (const linea of bloque[1].split(",")) {
    const identificador = linea.trim().split("::").pop();
    if (/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(identificador)) {
      nombres.add(identificador);
    }
  }
  return nombres;
}

function comandosInvocadosDesdeElFrontend() {
  const invocados = new Map(); // nombre -> [ficheros]
  for (const fichero of readdirSync(API_DIR)) {
    if (!fichero.endsWith(".ts")) continue;
    const ruta = join(API_DIR, fichero);
    const contenido = readFileSync(ruta, "utf-8");
    // Cubre invokeCommand("nombre" y invokeCommand<T>(\n  "nombre" en varias líneas.
    for (const m of contenido.matchAll(/invokeCommand(?:<[^>]*>)?\(\s*\n?\s*"([^"]+)"/g)) {
      const nombre = m[1];
      if (!invocados.has(nombre)) invocados.set(nombre, []);
      invocados.get(nombre).push(fichero);
    }
  }
  return invocados;
}

const registrados = comandosRegistradosEnRust();
const invocados = comandosInvocadosDesdeElFrontend();

let fallos = 0;
for (const [nombre, ficheros] of invocados) {
  if (!registrados.has(nombre)) {
    console.error(
      `✗ "${nombre}" (en ${ficheros.join(", ")}) no está registrado en ${LIB_RS}. ` +
        `Este comando fallaría en cada llamada real.`,
    );
    fallos++;
  }
}

if (fallos > 0) {
  console.error(`\n✗ ${fallos} comando(s) IPC sin backend registrado.`);
  process.exit(1);
}

console.log(
  `✓ Comandos IPC verificados: ${invocados.size} invocaciones del frontend coinciden con ` +
    `comandos reales en ${LIB_RS}.`,
);
