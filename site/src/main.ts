/*
 * Boots the site: defines <unicel-art>, then starts Elm with every piece's
 * Meta as flags, from the shell's manifest, so no wasm loads before the page
 * shows. Elm renders the tags; each tag loads its category's module as it
 * comes near the screen, and plays it.
 */
import "unicel/element";
import { pieces } from "unicel";
import { Elm } from "./Main.elm";
import { ansi, gif, png, save, text, video, type Take } from "./exports";

const app = Elm.Main.init({
  node: document.getElementById("app"),
  flags: { pieces },
});

// The copy buttons: Elm has no clipboard of its own.
app.ports.copy?.subscribe((text) => {
  navigator.clipboard?.writeText(String(text)).catch(() => {});
});

/** The piece the page is about: the big one in its well. */
const big = () => document.querySelector<HTMLElement & { currentTime?: number }>(".well.big unicel-art");

interface Ask {
  ask: "time" | "download";
  kind?: "png" | "video" | "gif" | "txt" | "ansi" | "text";
  slug?: string;
  options?: string;
  mono?: boolean;
  paper?: boolean;
  time?: number | null;
}

// What Elm asks of the page: the big piece's play time, to pause it there,
// and a file of it.
app.ports.request?.subscribe((value) => {
  const ask = value as Ask;
  if (ask.ask === "time") app.ports.answer?.send({ time: big()?.currentTime ?? 0 });
  if (ask.ask === "download") void download(ask);
});

async function download(ask: Ask) {
  const kind = ask.kind!;
  const well = document.querySelector(".well.big");
  const style = well ? getComputedStyle(well) : null;
  const take: Take = {
    slug: ask.slug!,
    options: ask.options ?? "",
    mono: ask.mono ?? false,
    paper: ask.paper ?? false,
    ink: getComputedStyle(big() ?? document.body).color,
    well: style?.backgroundColor ?? "#fff",
    time: ask.time ?? big()?.currentTime ?? 0,
  };
  const part = (done: number) => app.ports.answer?.send({ making: kind, done });
  try {
    if (kind === "png") save(await png(take), `${take.slug}.png`);
    if (kind === "txt") save(new Blob([await text(take)], { type: "text/plain;charset=utf-8" }), `${take.slug}.txt`);
    if (kind === "ansi") save(new Blob([await ansi(take)], { type: "text/plain;charset=utf-8" }), `${take.slug}.ans`);
    // "copy text": the frame to the clipboard, not a file.
    if (kind === "text") await navigator.clipboard.writeText(await text(take));
    if (kind === "gif") save(await gif(take, part), `${take.slug}.gif`);
    if (kind === "video") {
      const { blob, ext } = await video(take, part);
      save(blob, `${take.slug}.${ext}`);
    }
    app.ports.answer?.send({ made: kind });
  } catch (error) {
    console.error(`could not make the ${kind} of ${take.slug}:`, error);
    app.ports.answer?.send({ failed: kind });
  }
}

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
