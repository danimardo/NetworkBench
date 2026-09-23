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

if (hasErrors) {
  process.exit(1);
}

console.log(`✓ Paridad de claves locales comprobada (${esKeys.size} claves sincronizadas).`);
