// Loads the wasm-pack output for the Node tests. WASM_PKG picks the package
// directory, relative to the repository root (default web/pkg, what
// `mise run build:wasm` writes: a module per category and modules.js);
// `mise run test` also points it at a debug build in target/, one module
// "all" with every piece and the debug ones, for the panic test.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const root = fileURLToPath(new URL("../../", import.meta.url));
const dir = resolve(root, process.env.WASM_PKG ?? "web/pkg");

/** The manifest: every piece's meta, the module for each category, each module's import, and the fallback module. */
export const { pieces, modules, imports, fallback } = await import(pathToFileURL(resolve(dir, "modules.js")).href);

/** One module by name: its bindings and its instantiated wasm exports (for `memory`). Instantiated once; later calls get the same instance. */
export async function load(name) {
  const mod = await import(pathToFileURL(resolve(dir, name, "unicel.js")).href);
  const wasm = await mod.default({ module_or_path: readFileSync(resolve(dir, name, "unicel_bg.wasm")) });
  return { mod, wasm };
}

/** Every module, by name. */
export async function loadAll() {
  const all = new Map();
  for (const name of Object.keys(imports)) all.set(name, await load(name));
  return all;
}

/** The name of the module that plays a slug, as web/src/wasm.ts's moduleOf finds it. */
export function moduleOf(slug) {
  const meta = pieces.find((m) => m.slug === slug);
  return meta ? (modules[meta.category] ?? null) : fallback;
}

/** True when the tests run against a build with the `debug` feature. */
export const debugBuild = process.env.WASM_DEBUG === "1";
