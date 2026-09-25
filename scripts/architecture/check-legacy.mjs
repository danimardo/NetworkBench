import { readdirSync, readFileSync, statSync } from "node:fs";
import { resolve, join, relative } from "node:path";

// APIs que `docs/governance/CONVENTIONS.md` prohíbe introducir (Svelte 3/4, SvelteKit,
// entorno de Vite). Búsqueda por patrón: detecta el uso literal, no lo demuestra ausente
// por otras vías. Los comentarios de línea y de bloque se ignoran.
const srcDir = resolve("src");

const reglas = [
  { id: "export let", re: /^\s*export\s+let\s/m, solo: /\.svelte$/, por: "usa `$props()`" },
  { id: "$: reactivo", re: /^\s*\$:\s/m, solo: /\.svelte$/, por: "usa `$derived`/`$effect`" },
  {
    id: "createEventDispatcher",
    re: /\bcreateEventDispatcher\b/,
    por: "usa callbacks tipados en `$props()`",
  },
  { id: "<slot>", re: /<slot[\s>/]/, solo: /\.svelte$/, por: "usa snippets" },
  {
    id: "SvelteKit ($app/$env/+page/+layout)",
    re: /["'](\$app\/|\$env\/)|\+page|\+layout/,
    por: "la arquitectura es Svelte 5 sin SvelteKit",
  },
  {
    id: "process.env",
    re: /\bprocess\.env\b/,
    por: "la configuración pública se lee solo en `lib/config`",
  },
  {
    id: "import.meta.env fuera de lib/config",
    re: /\bimport\.meta\.env\b/,
    excepto: (rel) => rel.startsWith("lib/config/"),
    por: "léelo únicamente en `lib/config`",
  },
];

function walk(dir) {
  let files = [];
  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    if (statSync(fullPath).isDirectory()) {
      files = files.concat(walk(fullPath));
    } else if (/\.(ts|js|svelte)$/.test(entry)) {
      files.push(fullPath);
    }
  }
  return files;
}

function sinComentarios(texto) {
  return texto
    .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, " "))
    .replace(/<!--[\s\S]*?-->/g, (m) => m.replace(/[^\n]/g, " "))
    .replace(/^\s*\/\/.*$/gm, "");
}

let hasErrors = false;

for (const file of walk(srcDir)) {
  const rel = relative(srcDir, file).replace(/\\/g, "/");
  const contenido = sinComentarios(readFileSync(file, "utf-8"));

  for (const regla of reglas) {
    if (regla.solo && !regla.solo.test(file)) continue;
    if (regla.excepto?.(rel)) continue;
    if (regla.re.test(contenido)) {
      console.error(`✗ API prohibida (${regla.id}) en ${rel}: ${regla.por}.`);
      hasErrors = true;
    }
  }
}

if (hasErrors) {
  process.exit(1);
}

console.log("✓ Comprobación de APIs legacy y de entorno superada.");
