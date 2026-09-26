#!/usr/bin/env node
/**
 * Comprueba que ningún texto visible de la interfaz esté escrito a mano en español.
 *
 * La interfaz es bilingüe (Historias.md §4): todo texto que ve una persona sale de
 * `locales/*.json` a través de `t()`. Un literal en una plantilla o en un script no cambia
 * de idioma cuando se cambia el idioma, y es justo el fallo que ya ocurrió con Inicio y el
 * menú lateral. `check-locales.mjs` vigila que las claves de un idioma estén en el otro;
 * este vigila que los textos lleguen a las claves.
 *
 * Qué mira, en `src/**` (sin tests):
 *   - nodos de texto de las plantillas Svelte;
 *   - atributos visibles o leídos por lectores de pantalla (`label`, `title`, `aria-label`…);
 *   - cadenas dentro de expresiones `{…}` de las plantillas;
 *   - cadenas de los `<script>` y de los `.ts` que parecen español.
 *
 * Qué NO es: una garantía de traducción completa. Detecta español por tildes y por palabras
 * de uso común; una frase en español sin ninguna de ellas pasaría. Es una red, no una prueba.
 *
 * Excepciones legítimas: nombres propios y siglas (`PERMITIDOS`) y los nombres de idioma,
 * que se escriben en su propio idioma a propósito.
 */
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";

const RAIZ = process.env.I18N_RAIZ ?? "src";

/** Textos que son nombres propios, siglas o el nombre de un idioma en su propio idioma. */
const PERMITIDOS = new Set([
  "NetworkBench",
  "Microsoft NTTTCP",
  "NTTTCP",
  "Español",
  "English",
  "Info",
  "Debug",
  "Warn",
  "Streams",
  "PowerShell",
  "Preflight",
  "mDNS",
  // Identificadores de ruta y de estado: son claves internas, nunca se muestran.
  "inicio",
  "historial",
  "ajustes",
  "error",
]);

/**
 * Rutas cuyos textos no son interfaz. Cada una lleva su motivo:
 *   - `src/lib/contracts/`: mensajes de validación de Zod. Los produce la validación de datos
 *     que cruzan el IPC; solo el formulario de opciones avanzadas muestra uno (`issues[0]`),
 *     y queda como deuda conocida, no como algo que este check dé por resuelto.
 */
const RUTAS_EXCLUIDAS = [
  "src/lib/contracts/",
  // Mensaje de validación Zod de un dato del backend (el código de emparejamiento), no de un formulario.
  "src/lib/api/peers.ts",
  // Su único texto en español es el `msg` que se manda al registro de desarrollo.
  "src/lib/api/snapshot.svelte.ts",
];

/** Palabras muy frecuentes en español que casi nunca aparecen en un identificador técnico. */
const PALABRAS = new RegExp(
  "\\b(de|del|la|las|el|los|un|una|unos|en|con|sin|por|para|que|se|su|sus|al|no|hay|ha|han|fue|es|est[aá]|son|como|m[aá]s|muy|todo|todos|esta|este|estos|desde|hacia|entre|sobre|ver|abrir|cerrar|guardar|cancelar|aceptar|repetir|reintentar|continuar|conectar|buscar|equipo|equipos|prueba|medici[oó]n|cargando|resultado|resultados|velocidad|puerto|cliente|servidor|conocido|favorito|favoritos|confianza|copiar|copiado|minimizar|maximizar|restaurar|disponible|ocupado|especificado|evaluable|desde|hacia|activado|desactivado|activar|desactivar|aplicar|eliminar|iniciar|detener|completada|completado|cancelada|correcto|pendiente|comprobando|opciones|ajuste|configuraci[oó]n|advertencia|aviso|problema|[oó]ptimo)\\b",
  "i",
);

const ACENTOS = /[áéíóúñüÁÉÍÓÚÑ¿¡]/;

function ficheros(dir, salida = []) {
  for (const nombre of readdirSync(dir)) {
    const ruta = join(dir, nombre);
    if (statSync(ruta).isDirectory()) ficheros(ruta, salida);
    else if (
      /\.(svelte|ts)$/.test(nombre) &&
      !/\.(test|spec)\.ts$/.test(nombre) &&
      !/\.d\.ts$/.test(nombre)
    )
      salida.push(ruta);
  }
  return salida;
}

function pareceEspanol(texto) {
  const t = texto.trim();
  if (t.length < 3) return false;
  if (PERMITIDOS.has(t)) return false;
  if (!/[A-Za-zÁÉÍÓÚáéíóúñÑ]{3,}/.test(t)) return false;
  // Siglas o constantes: TCP, UDP, PDF, MBIT_S…
  if (/^[A-Z0-9 ._/()+-]+$/.test(t)) return false;
  // Rutas, URLs, identificadores y valores técnicos.
  if (/^(https?:|\.{0,2}\/|[a-z][a-zA-Z0-9]*([.:_-][a-zA-Z0-9]+)+$|#|--|[a-z]+-[a-z-]+$)/.test(t))
    return false;
  return ACENTOS.test(t) || PALABRAS.test(t);
}

/** Sustituye por espacios (conserva las líneas) lo que no interesa mirar. */
function blanquear(texto, regex) {
  return texto.replace(regex, (m) => m.replace(/[^\n]/g, " "));
}

function lineaDe(texto, indice) {
  return texto.slice(0, indice).split("\n").length;
}

const hallazgos = [];
function anotar(fichero, texto, indice, que) {
  hallazgos.push(`${fichero}:${lineaDe(texto, indice)}  ${que}`);
}

/** Cadenas «...», '...' y `...` de un trozo de código, sin comentarios ni imports. */
function cadenasDeCodigo(codigo) {
  let s = blanquear(codigo, /\/\*[\s\S]*?\*\//g);
  s = blanquear(s, /(^|[^:"'`\\])\/\/[^\n]*/g);
  const salida = [];
  for (const m of s.matchAll(/(["'`])((?:\\.|(?!\1)[^\\\n])*)\1/g)) {
    const linea = s.slice(
      s.lastIndexOf("\n", m.index) + 1,
      s.indexOf("\n", m.index) === -1 ? s.length : s.indexOf("\n", m.index),
    );
    // Registros de desarrollo, no texto de interfaz.
    if (
      /\b(logger\.|console\.|eventCode|module:|message:|throw new Error|tracing)|^\s*import\b|\bfrom\s+["']/.test(
        linea,
      )
    )
      continue;
    if (/\bt\(\s*$/.test(s.slice(Math.max(0, m.index - 4), m.index))) continue;
    salida.push({ texto: m[2], indice: m.index });
  }
  return salida;
}

for (const ruta of ficheros(RAIZ)) {
  const fichero = relative(".", ruta).split(sep).join("/");
  if (RUTAS_EXCLUIDAS.some((r) => fichero.startsWith(r))) continue;
  const src = readFileSync(ruta, "utf8");

  if (ruta.endsWith(".ts")) {
    for (const { texto, indice } of cadenasDeCodigo(src))
      if (pareceEspanol(texto))
        anotar(fichero, src, indice, `cadena en español: "${texto.slice(0, 70)}"`);
    continue;
  }

  const scripts = [...src.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/g)];
  for (const m of scripts) {
    const desplazamiento = m.index + m[0].indexOf(m[1]);
    for (const { texto, indice } of cadenasDeCodigo(m[1]))
      if (pareceEspanol(texto))
        anotar(fichero, src, desplazamiento + indice, `cadena en español: "${texto.slice(0, 70)}"`);
  }

  // Plantilla: sin script, estilo ni comentarios (las líneas se conservan).
  let tpl = blanquear(src, /<script[\s\S]*?<\/script>/g);
  tpl = blanquear(tpl, /<style[\s\S]*?<\/style>/g);
  tpl = blanquear(tpl, /<!--[\s\S]*?-->/g);

  // Nodos de texto. Las expresiones {…} se vacían (no se descartan) para ver también el
  // texto que las rodea: «Cliente local: {valor}».
  let sinExpresiones = tpl;
  for (let i = 0; i < 4; i++) sinExpresiones = blanquear(sinExpresiones, /\{[^{}]*\}/g);
  for (const m of sinExpresiones.matchAll(/>([^<>]+)</g)) {
    const texto = m[1].replace(/\s+/g, " ").trim();
    if (pareceEspanol(texto))
      anotar(fichero, src, m.index + 1, `texto en la plantilla: "${texto.slice(0, 70)}"`);
  }
  // Atributos que ve o lee una persona.
  for (const m of tpl.matchAll(
    /\b(label|title|placeholder|aria-label|alt|text|hint|description|subtitle|tooltip)="([^"{}]+)"/g,
  ))
    if (pareceEspanol(m[2]))
      anotar(fichero, src, m.index, `atributo ${m[1]}="${m[2].slice(0, 60)}"`);
  // Cadenas dentro de expresiones {…}.
  for (const m of tpl.matchAll(/\{([^{}]*)\}/g))
    for (const { texto } of cadenasDeCodigo(m[1]))
      if (pareceEspanol(texto))
        anotar(fichero, src, m.index, `cadena en una expresión: "${texto.slice(0, 60)}"`);
}

if (hallazgos.length > 0) {
  console.error(`✗ ${hallazgos.length} texto(s) de interfaz sin pasar por t():`);
  for (const h of hallazgos) console.error(`  ${h}`);
  console.error(
    "\nMuévelos a locales/es.json y locales/en.json y llámalos con t(). Si es un nombre propio o una",
  );
  console.error(
    "sigla, añádelo a PERMITIDOS en scripts/architecture/check-i18n-literals.mjs con su motivo.",
  );
  process.exit(1);
}
console.log("✓ Ningún texto de interfaz escrito a mano fuera de t().");
