/*
 * Boots the site: defines <unicel-art>, then starts Elm with every piece's
 * Meta as flags, from the shell's manifest, so no wasm loads before the page
 * shows. Elm renders the tags; each tag loads its category's module as it
 * comes near the screen, and plays it.
 */
import "unicel/element";
import { pieces } from "unicel";
import { Elm } from "./Main.elm";

const app = Elm.Main.init({
  node: document.getElementById("app"),
  flags: { pieces },
});

// The copy buttons: Elm has no clipboard of its own.
app.ports.copy?.subscribe((text) => {
  navigator.clipboard?.writeText(String(text)).catch(() => {});
});

// Dev only (Vite drops this from production builds): `debugPanic()` in the
// browser console adds a <unicel-art> playing a deliberately broken piece. It
// draws for a second, then panics; the debug wasm build that `mise run dev`
// serves prints the panic's message and source location. That tag stops; the
// others keep going.
if (import.meta.env.DEV) {
  Object.assign(window, {
    debugPanic: () => {
      const tag = document.createElement("unicel-art");
      tag.setAttribute("piece", "debug-panic");
      tag.style.cssText = "position:fixed;right:1rem;bottom:1rem;outline:1px dashed red";
      document.body.append(tag);
      return tag;
    },
  });
}
