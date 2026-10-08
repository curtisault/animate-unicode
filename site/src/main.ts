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

// Dev only (Vite drops this from production builds): `debugPanic()` in the
// browser console adds a <unicode-art> playing a deliberately broken piece. It
// draws for a second, then panics; the debug wasm build that `mise run dev`
// serves prints the panic's message and source location. That tag stops; the
// others keep going.
if (import.meta.env.DEV) {
  Object.assign(window, {
    debugPanic: () => {
      const tag = document.createElement("unicode-art");
      tag.setAttribute("piece", "debug-panic");
      tag.style.cssText = "position:fixed;right:1rem;bottom:1rem;outline:1px dashed red";
      document.body.append(tag);
      return tag;
    },
  });
}
