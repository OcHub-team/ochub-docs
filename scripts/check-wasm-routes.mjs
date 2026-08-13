import { access, readdir, readFile } from "node:fs/promises";
import path from "node:path";

const distRoot = path.resolve("dist");
const wasmEntry = path.join(distRoot, "wasm", "index.html");

async function listHtml(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await listHtml(absolute)));
    } else if (entry.isFile() && entry.name.endsWith(".html")) {
      files.push(absolute);
    }
  }

  return files;
}

await access(wasmEntry);

const htmlFiles = await listHtml(distRoot);
const failures = [];
let explicitWasmRoutes = 0;

for (const file of htmlFiles) {
  const source = await readFile(file, "utf8");
  if (source.includes('/wasm/?scene=')) {
    failures.push(path.relative(distRoot, file));
  }
  explicitWasmRoutes += source.split('/wasm/index.html?scene=').length - 1;
}

if (failures.length > 0) {
  console.error("Directory-style WASM routes remain in generated pages:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exitCode = 1;
} else if (explicitWasmRoutes === 0) {
  console.error("No explicit WASM renderer routes were found in generated pages.");
  process.exitCode = 1;
} else {
  console.log(
    `WASM route check: ${explicitWasmRoutes} generated links use /wasm/index.html.`,
  );
}
