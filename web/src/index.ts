/*
 * animate-unicode: the TS shell over the Rust pieces.
 *
 * `pieces` lists every piece, `player()` makes one in its category's wasm
 * module (loading that module the first time), `mount` draws one in an
 * element. The <unicode-art> tag is in "animate-unicode/element".
 */
export { mount, WIDE_TAIL, type Meta, type MountOptions } from "./mount.ts";
export { moduleOf, pieces, player, wasm, type Bindings, type InitOutput, type Module, type Player } from "./wasm.ts";
