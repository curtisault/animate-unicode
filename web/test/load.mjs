// Loads the wasm-pack output for the Node tests. WASM_PKG picks the package
// directory, relative to the repository root (default web/pkg, what
// `mise run build:wasm` writes); `mise run test` also points it at a debug
// build in target/ for the panic test.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const root = fileURLToPath(new URL("../../", import.meta.url));
const dir = resolve(root, process.env.WASM_PKG ?? "web/pkg");

/** The bindings module and the instantiated wasm exports (for `memory`). */
export async function load() {
  const mod = await import(pathToFileURL(resolve(dir, "animate_unicode.js")).href);
  const wasm = await mod.default({ module_or_path: readFileSync(resolve(dir, "animate_unicode_bg.wasm")) });
  return { mod, wasm };
}

/** True when the tests run against a build with the `debug` feature. */
export const debugBuild = process.env.WASM_DEBUG === "1";
