import { access, readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

const distRoot = path.resolve("dist");
const wasmRoot = path.join(distRoot, "wasm");
const wasmEntry = path.join(wasmRoot, "index.html");
const maxWasmBytes = 10 * 1024 * 1024;

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

const wasmFiles = (await readdir(wasmRoot))
  .filter((file) => file.endsWith(".wasm"))
  .map((file) => path.join(wasmRoot, file));

if (wasmFiles.length !== 1) {
  console.error(`Expected one generated WASM binary, found ${wasmFiles.length}.`);
  process.exitCode = 1;
} else {
  const { size } = await stat(wasmFiles[0]);
  const sizeMiB = (size / 1024 / 1024).toFixed(2);
  const maxMiB = (maxWasmBytes / 1024 / 1024).toFixed(0);

  if (size > maxWasmBytes) {
    console.error(`WASM size check: ${sizeMiB} MiB exceeds ${maxMiB} MiB.`);
    process.exitCode = 1;
  } else {
    console.log(`WASM size check: ${sizeMiB} MiB (limit ${maxMiB} MiB).`);
  }
}

const htmlFiles = await listHtml(distRoot);

// A site-wide count did not catch the Codex walkthrough being removed during
// a documentation migration. Check this required guide in every language.
for (const prefix of ["", "zh/", "ja/"]) {
  const guide = await readFile(path.join(distRoot, prefix, "codex/configure/index.html"), "utf8");
  if (!/<iframe\b[^>]*src="[^\"]*\/wasm\/index\.html\?[^\"]*app=Codex/.test(guide)) {
    console.error(`Missing Codex WASM walkthrough: /${prefix}codex/configure/`);
    process.exitCode = 1;
  }
}
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
