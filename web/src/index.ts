/*
 * animate-unicode: the TS shell over the Rust pieces.
 *
 * `wasm()` loads the module, `Player` is a piece in it, `mount` draws one in
 * an element. The <unicode-art> tag is in "animate-unicode/element".
 */
export { mount, WIDE_TAIL, type Meta, type MountOptions } from "./mount.ts";
export { Player, metas_json, slugs_json, wasm } from "./wasm.ts";
