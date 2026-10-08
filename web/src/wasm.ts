/*
 * The wasm module, loaded once and shared by every tag on the page.
 * web/pkg is what `mise run build:wasm` writes; it is not checked in.
 */
import init, { type InitOutput } from "../pkg/animate_unicode.js";

let loading: Promise<InitOutput> | undefined;

/** The instantiated module. The first call fetches and compiles it; later calls share that. */
export function wasm(): Promise<InitOutput> {
  return (loading ??= init());
}

export { Player, metas_json, slugs_json } from "../pkg/animate_unicode.js";
