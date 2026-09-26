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
// Ficheros que llaman a `invokeCommand` sin estar en `lib/api`.
const OTROS_FICHEROS = ["src/lib/window.ts"];

/** Ficheros del frontend que invocan comandos: todo `lib/api/*.ts` más los sueltos. */
function ficherosQueInvocan() {
  const deApi = readdirSync(API_DIR)
    .filter((f) => f.endsWith(".ts") && !f.endsWith(".test.ts"))
    .map((f) => join(API_DIR, f));
  return [...deApi, ...OTROS_FICHEROS];
}

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
  for (const ruta of ficherosQueInvocan()) {
    const fichero = ruta;
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

// ---------------------------------------------------------------------------------------
// Argumentos (2026-09-26)
//
// Segundo fallo de la misma familia, más grave que el de los nombres: `transport.ts`
// envolvía todo argumento como `{ request: entrada }`, pero Tauri hace casar las claves del
// objeto con los NOMBRES DE LOS PARÁMETROS de la función Rust, en camelCase
// (`invoke("f", { invokeMessage })` ↔ `fn f(invoke_message)`). Con el envoltorio, cada
// comando con parámetros (guardar un ajuste, conectar por IP, iniciar una prueba…) fallaba
// en la app real con un `NB-INTERNAL-001` genérico, y ningún test lo veía porque
// `setTransportMock` sustituye el transporte entero.
//
// Aquí se compara, para cada llamada, el conjunto de claves que envía el frontend con los
// parámetros de la función Rust.
// ---------------------------------------------------------------------------------------

/** Parte `texto` por las comas de nivel superior (fuera de (), <>, [] y {}). */
function partirPorComas(texto) {
  const partes = [];
  let profundidad = 0;
  let actual = "";
  for (let i = 0; i < texto.length; i++) {
    const c = texto[i];
    if ("({[<".includes(c) && !(c === "<" && texto[i - 1] === "-")) profundidad++;
    else if (")}]>".includes(c) && !(c === ">" && texto[i - 1] === "-")) profundidad--;
    if (c === "," && profundidad === 0) {
      partes.push(actual);
      actual = "";
    } else actual += c;
  }
  if (actual.trim()) partes.push(actual);
  return partes.map((p) => p.trim()).filter(Boolean);
}

/** Contenido entre el paréntesis que abre en `desde` y su pareja. */
function entreParentesis(texto, desde) {
  let profundidad = 0;
  for (let i = desde; i < texto.length; i++) {
    if (texto[i] === "(") profundidad++;
    else if (texto[i] === ")" && --profundidad === 0) return texto.slice(desde + 1, i);
  }
  return null;
}

const aCamel = (s) => s.replace(/^_+/, "").replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase());

/** Parámetros de cada #[tauri::command]: { requeridos, opcionales } en camelCase. */
function parametrosDeComandosRust() {
  const comandos = new Map();
  const recorrer = (dir) => {
    for (const nombre of readdirSync(dir, { withFileTypes: true })) {
      const ruta = join(dir, nombre.name);
      if (nombre.isDirectory()) recorrer(ruta);
      else if (nombre.name.endsWith(".rs")) {
        const src = readFileSync(ruta, "utf-8");
        for (const m of src.matchAll(
          /#\[tauri::command[^\]]*\]\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*(?:<[^>]*>)?\s*\(/g,
        )) {
          const firma = entreParentesis(src, m.index + m[0].length - 1);
          if (firma === null) continue;
          const requeridos = new Set();
          const opcionales = new Set();
          for (const p of partirPorComas(firma)) {
            const dos = p.indexOf(":");
            if (dos < 0) continue;
            const id = p
              .slice(0, dos)
              .replace(/\bmut\b/, "")
              .trim();
            const tipo = p.slice(dos + 1).trim();
            // Los que inyecta Tauri no vienen del frontend.
            if (/\b(State|AppHandle|WebviewWindow|Window|Webview)\b/.test(tipo)) continue;
            (/^Option\s*</.test(tipo) || id.startsWith("_") ? opcionales : requeridos).add(
              aCamel(id),
            );
          }
          comandos.set(m[1], { requeridos, opcionales });
        }
      }
    }
  };
  recorrer("src-tauri/src");
  return comandos;
}

/** Claves de un literal `{ a, b: x, ...c }`, o null si `expr` no es un literal de objeto. */
function clavesDeLiteral(expr) {
  const e = expr.trim();
  if (!e.startsWith("{") || !e.endsWith("}")) return null;
  const claves = new Set();
  for (const parte of partirPorComas(e.slice(1, -1))) {
    if (parte.startsWith("...")) return null; // no se puede saber qué añade
    const m = /^["']?([A-Za-z_$][\w$]*)["']?\s*(?::|$)/.exec(parte);
    if (m) claves.add(m[1]);
  }
  return claves;
}

/** Llamadas `invokeCommand("cmd", ARG, …)`: { comando, argumento } con ARG en texto. */
function llamadasConArgumento() {
  const llamadas = [];
  for (const fichero of ficherosQueInvocan()) {
    const src = readFileSync(fichero, "utf-8");
    for (const m of src.matchAll(/invokeCommand(?:<[^(]*?>)?\(/g)) {
      const cuerpo = entreParentesis(src, m.index + m[0].length - 1);
      if (cuerpo === null) continue;
      const [comando, argumento] = partirPorComas(cuerpo);
      const nombre = /^\s*"([^"]+)"\s*$/.exec(comando ?? "")?.[1];
      if (nombre) llamadas.push({ nombre, argumento: argumento ?? "undefined", fichero });
    }
  }
  return llamadas;
}

const rust = parametrosDeComandosRust();
let fallosArgumentos = 0;
for (const { nombre, argumento, fichero } of llamadasConArgumento()) {
  const parametros = rust.get(nombre);
  if (!parametros) continue; // ya lo notificó la comprobación de nombres
  const arg = argumento.trim();
  const sinArgumentos = arg === "undefined" || arg === "{}";
  const claves = sinArgumentos ? new Set() : clavesDeLiteral(arg);

  if (claves === null) {
    console.error(
      `✗ "${nombre}" (${fichero}): el argumento \`${arg.slice(0, 40)}\` no es un literal de objeto. ` +
        `Tauri casa las claves con los parámetros de Rust: escríbelo como { ${[...parametros.requeridos].join(", ")} }.`,
    );
    fallosArgumentos++;
    continue;
  }
  const faltan = [...parametros.requeridos].filter((p) => !claves.has(p));
  const sobran = [...claves].filter(
    (c) => !parametros.requeridos.has(c) && !parametros.opcionales.has(c),
  );
  if (faltan.length || sobran.length) {
    console.error(
      `✗ "${nombre}" (${fichero}): Rust espera { ${[...parametros.requeridos, ...[...parametros.opcionales].map((o) => o + "?")].join(", ")} } ` +
        `y el frontend envía { ${[...claves].join(", ")} }` +
        (faltan.length ? `; faltan: ${faltan.join(", ")}` : "") +
        (sobran.length ? `; sobran: ${sobran.join(", ")}` : "") +
        ".",
    );
    fallosArgumentos++;
  }
}

if (fallosArgumentos > 0) {
  console.error(`\n✗ ${fallosArgumentos} comando(s) IPC con argumentos que no casan con Rust.`);
  process.exit(1);
}
console.log(
  "✓ Argumentos IPC verificados: las claves que envía el frontend casan con los parámetros de cada comando Rust.",
);
