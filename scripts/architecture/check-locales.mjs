import { readFileSync } from "node:fs";
import { resolve } from "node:path";

function flattenKeys(obj, prefix = "") {
  let keys = [];
  for (const [k, v] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === "object" && !Array.isArray(v)) {
      keys = keys.concat(flattenKeys(v, fullKey));
    } else {
      keys.push(fullKey);
    }
  }
  return keys;
}

const esPath = resolve("locales/es.json");
const enPath = resolve("locales/en.json");

const es = JSON.parse(readFileSync(esPath, "utf-8"));
const en = JSON.parse(readFileSync(enPath, "utf-8"));

const esKeys = new Set(flattenKeys(es));
const enKeys = new Set(flattenKeys(en));

let hasErrors = false;

for (const key of esKeys) {
  if (!enKeys.has(key)) {
    console.error(`✗ Clave presente en es.json pero ausente en en.json: ${key}`);
    hasErrors = true;
  }
}

for (const key of enKeys) {
  if (!esKeys.has(key)) {
    console.error(`✗ Clave presente en en.json pero ausente en es.json: ${key}`);
    hasErrors = true;
  }
}

// Paridad de formatos: toda clave con marcadores {placeholder} debe usar exactamente
// los mismos nombres en los dos idiomas. Un texto puede reordenar el marcador dentro
// de la frase (es/en difieren en orden de palabras); lo que no puede es perder uno,
// añadir uno que el otro idioma no tiene, o cambiarle el nombre (SC-009).
const PLACEHOLDER = /\{[a-zA-Z_]+\}/g;

function flattenValues(obj, prefix = "") {
  let out = {};
  for (const [k, v] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === "object" && !Array.isArray(v)) {
      out = { ...out, ...flattenValues(v, fullKey) };
    } else {
      out[fullKey] = v;
    }
  }
  return out;
}

const esValues = flattenValues(es);
const enValues = flattenValues(en);

for (const key of esKeys) {
  if (!enKeys.has(key)) continue; // ya reportado arriba
  const esVal = esValues[key];
  const enVal = enValues[key];
  if (typeof esVal !== "string" || typeof enVal !== "string") continue;

  const esPh = [...esVal.matchAll(PLACEHOLDER)].map((m) => m[0]).sort();
  const enPh = [...enVal.matchAll(PLACEHOLDER)].map((m) => m[0]).sort();
  if (JSON.stringify(esPh) !== JSON.stringify(enPh)) {
    console.error(
      `✗ Marcadores distintos en "${key}": es=[${esPh}] en=[${enPh}]. ` +
        `Un valor perdido o renombrado rompe la interpolación en ese idioma.`,
    );
    hasErrors = true;
  }
}

// Plurales: si alguna clave usara sufijos de plural (_one/_other/_few/_many/_zero/_two,
// convención habitual de i18n), ambos idiomas deberían declarar el mismo conjunto de
// formas. Hoy no existe ninguna: se comprueba para que, el día que se introduzcan,
// este script las cubra sin cambios.
const PLURAL_SUFFIXES = ["_zero", "_one", "_two", "_few", "_many", "_other"];
const pluralBases = (keys) =>
  new Set(
    [...keys]
      .filter((k) => PLURAL_SUFFIXES.some((s) => k.endsWith(s)))
      .map((k) => PLURAL_SUFFIXES.reduce((acc, s) => acc.replace(new RegExp(`${s}$`), ""), k)),
  );
const esPluralBases = pluralBases(esKeys);
const enPluralBases = pluralBases(enKeys);
for (const base of new Set([...esPluralBases, ...enPluralBases])) {
  const esForms = [...esKeys].filter((k) => k.startsWith(base)).sort();
  const enForms = [...enKeys].filter((k) => k.startsWith(base)).sort();
  if (JSON.stringify(esForms) !== JSON.stringify(enForms)) {
    console.error(`✗ Formas de plural distintas para "${base}": es=[${esForms}] en=[${enForms}]`);
    hasErrors = true;
  }
}
if (esPluralBases.size === 0 && enPluralBases.size === 0) {
  console.log("ℹ Sin claves de plural en ningún idioma; nada que verificar todavía (SC-009).");
}

if (hasErrors) {
  process.exit(1);
}

console.log(`✓ Paridad de claves locales comprobada (${esKeys.size} claves sincronizadas).`);
console.log(
  `✓ Paridad de formatos (marcadores {placeholder}) comprobada en ambos idiomas (SC-009).`,
);
