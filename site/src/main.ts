/*
 * Boots the site: loads the wasm module, defines <unicode-art>, then starts
 * Elm with every piece's Meta as flags. Elm renders the tags; the shell plays
 * them.
 */
import "animate-unicode/element";
import { metas_json, wasm } from "animate-unicode";
import { Elm } from "./Main.elm";

await wasm();

Elm.Main.init({
  node: document.getElementById("app"),
  flags: { pieces: JSON.parse(metas_json()) },
});
