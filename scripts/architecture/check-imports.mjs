import { readdirSync, readFileSync, statSync } from "node:fs";
import { resolve, join, relative } from "node:path";

const srcDir = resolve("src");

function walk(dir) {
  let files = [];
  if (!readdirSync) return files;
  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) {
      files = files.concat(walk(fullPath));
    } else if (/\.(ts|js|svelte)$/.test(entry)) {
      files.push(fullPath);
    }
  }
  return files;
}

const files = walk(srcDir);
let hasErrors = false;

for (const file of files) {
  const content = readFileSync(file, "utf-8");
  const relPath = relative(srcDir, file).replace(/\\/g, "/");

  // Regla 1: @tauri-apps/api solo se importa directamente en lib/api
  if (!relPath.startsWith("lib/api/") && /from\s+['"]@tauri-apps\/api/.test(content)) {
    console.error(
      `✗ Violación de arquitectura: @tauri-apps/api importado fuera de src/lib/api en ${relPath}`,
    );
    hasErrors = true;
  }

  // Regla 2: Imports profundos entre features
  const featureMatch = relPath.match(/^features\/([^/]+)\//);
  if (featureMatch) {
    const currentFeature = featureMatch[1];
    const deepImportRegex = /from\s+['"](?:\.\.\/)+features\/([^/'"]+)\/([^'"]+)['"]/g;
    let m;
    while ((m = deepImportRegex.exec(content)) !== null) {
      const targetFeature = m[1];
      const targetSubpath = m[2];
      if (targetFeature !== currentFeature && targetSubpath !== "index" && targetSubpath !== "") {
        console.error(
          `✗ Violación de arquitectura: Import profundo a feature '${targetFeature}' (${targetSubpath}) en ${relPath}. Solo se permite importar el index público.`,
        );
        hasErrors = true;
      }
    }
  }
}

if (hasErrors) {
  process.exit(1);
}

console.log(`✓ Comprobación de imports y fronteras de arquitectura superada.`);
