/*
 * The wasm modules: one for each category of pieces, loaded the first time a
 * piece of that category is asked for and shared by every tag on the page
 * after that, so a page loads only the categories it shows. web/pkg is what
 * `mise run build:wasm` writes, with modules.js listing every piece and the
 * module that plays it; it is not checked in. The dev build is one module,
 * "all", which also plays the debug pieces.
 */
import { fallback, imports, modules, pieces as metas, type Bindings, type InitOutput, type Player } from "../pkg/modules.js";
import type { Meta } from "./mount.ts";

/** A loaded module: its bindings (`Player` and the rest) and its instantiated exports (`memory`). */
export interface Module {
  bindings: Bindings;
  exports: InitOutput;
}

/** Every piece's meta, category by category, known without loading any wasm. */
export const pieces = metas as readonly Meta[];

const loading = new Map<string, Promise<Module>>();

/** A module by name: a category, or "all" in the dev build. The first call fetches and compiles it; later calls share that. */
export function wasm(name: string): Promise<Module> {
  let module = loading.get(name);
  if (!module) {
    const load = imports[name];
    if (!load) return Promise.reject(new Error(`there is no wasm module named "${name}"`));
    module = load().then(async (bindings) => ({ bindings, exports: await bindings.default() }));
    loading.set(name, module);
    // A module that failed to load (offline, say) is tried again next time it is asked for.
    module.catch(() => loading.get(name) === module && loading.delete(name));
  }
  return module;
}

/** The name of the module that plays a slug: its category's, or the dev build's for a slug no category lists. Null when none can. */
export function moduleOf(slug: string): string | null {
  const meta = pieces.find((m) => m.slug === slug);
  return meta ? (modules[meta.category] ?? null) : fallback;
}

/**
 * A piece by slug with JSON option overrides (`""` for none), made in its
 * category's module, which this loads if need be. Rejects for an unknown slug
 * or bad options, with the message `new Player` throws.
 */
export async function player(slug: string, options = ""): Promise<{ player: Player; exports: InitOutput }> {
  const name = moduleOf(slug);
  if (name === null) throw new Error(`there is no piece named "${slug}"`);
  const { bindings, exports } = await wasm(name);
  return { player: new bindings.Player(slug, options), exports };
}

export type { Bindings, InitOutput, Player };
