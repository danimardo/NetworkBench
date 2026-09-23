import { readdirSync, readFileSync, statSync } from "node:fs";
import { resolve, join, relative } from "node:path";

const srcDir = resolve("src");

function walk(dir) {
  let files = [];
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

  // Prohibir console.log en funcionalidad (permitir solo en lib/logging si es el wrapper)
  if (!relPath.startsWith("lib/logging/") && /\bconsole\.(log|debug|info)\s*\(/.test(content)) {
    console.error(
      `✗ Violación de logging: Se encontró uso directo de console.* en ${relPath}. Use el wrapper de logging.`,
    );
    hasErrors = true;
  }
}

if (hasErrors) {
  process.exit(1);
}

console.log(`✓ Comprobación de logger único superada.`);
