import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';

const file = resolve('build/index.js');
if (!existsSync(file)) {
  console.error(`patch-build: file not found: ${file}`);
  process.exit(1);
}

let src = readFileSync(file, 'utf8');
const before = 'import source wasmModule from "./index_bg.wasm";';
const after = 'import wasmModule from "./index_bg.wasm";';

if (src.includes(before)) {
  src = src.replace(before, after);
  writeFileSync(file, src, 'utf8');
  console.log('patch-build: fixed invalid wasm import in build/index.js');
} else if (!src.includes(after)) {
  console.warn('patch-build: expected pattern not found; no changes made');
} else {
  console.log('patch-build: wasm import already correct');
}
