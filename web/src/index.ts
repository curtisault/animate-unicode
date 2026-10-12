/*
 * unicel: the TS shell over the Rust pieces.
 *
 * `pieces` lists every piece, `player()` makes one in its category's wasm
 * module (loading that module the first time), `mount` plays one in an
 * element, and `paint` draws one frame onto any canvas. The <unicel-art> tag
 * is in "unicel/element".
 */
export { mount, NO_BACK, WIDE_TAIL, type Meta, type MountOptions, type Playback } from "./mount.ts";
export { paint, type Look } from "./frame.ts";
export { moduleOf, pieces, player, wasm, type Bindings, type InitOutput, type Module, type Player } from "./wasm.ts";
