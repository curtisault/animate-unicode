/*
 * <unicel-art>: any piece in the library, as one tag. Importing this module
 * defines the tag; on a server, where there is no DOM, it does nothing.
 *
 *   <script type="module" src="/element.js"></script>
 *   <unicel-art piece="donut"></unicel-art>
 *
 * Attributes:
 *   piece    a piece's slug: "donut", "braille-wave"
 *   fps      overrides the piece's frame rate
 *   options  JSON overriding the piece's option defaults: '{"speed":2}'
 *   label    what the picture shows, for screen readers; the piece's name otherwise
 *   mono     draws a coloured piece as text in one ink, like any other
 *   motion   plays even when the reader prefers reduced motion; only for a
 *            page that gives the reader its own way to stop it
 *   time     holds the piece, paused, on the frame at this many seconds;
 *            changing it seeks without reloading, removing it plays on from there
 *
 * `currentTime` is the play time of the frame shown, in seconds.
 *
 * Text pieces draw into a <pre> in the element's colour and font size; the
 * coloured ones draw onto a <canvas> as wide as the element. Whatever the
 * element holds before it loads (a first frame rendered on the server, say)
 * stays until the piece is ready.
 *
 * A tag loads nothing until it first comes within a screen of the viewport,
 * then loads its piece's category (one wasm module each), so a page of many
 * pieces fetches each category only as its pieces come near.
 */
import { mount, type Meta, type MountOptions, type Playback } from "./mount.ts";
import { player as make, type InitOutput, type Player } from "./wasm.ts";

// The bundled font (web/fonts, built by scripts/font.py from Iosevka Fixed Extended, SIL OFL 1.1): ASCII, box
// drawing, blocks, geometric shapes, arrows, braille, sextants and octants, every glyph 0.6 em wide. It leads the
// stack so a piece's every glyph comes from one face with one width, and rows line up on any platform; wide glyphs
// (CJK, emoji) are outside its range and come from the system. Located next to this module, so it travels with the
// npm package and a bundler copies it like any other asset. The range must match scripts/font.py's RANGES.
const FONT_URL = new URL("../fonts/unicel-mono.woff2", import.meta.url).href;
const UNICODE_RANGE = "U+0020-007E,U+00B0,U+00B7,U+2022,U+2190-21FF,U+2500-25FF,U+2800-28FF,U+1CD00-1CDE5,U+1CEA0-1CEAF,U+1FB00-1FBFF";

// :where gives these no specificity, so any rule of the page's own wins.
const STYLE =
  `@font-face{font-family:"unicel mono";src:url(${FONT_URL}) format("woff2");unicode-range:${UNICODE_RANGE};font-display:swap}` +
  ':where(unicel-art){display:block}:where(unicel-art>pre){margin:0;font:inherit;font-family:"unicel mono",ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;line-height:1.2;letter-spacing:0;white-space:pre;font-variant-ligatures:none}';

const Base = (typeof HTMLElement === "undefined" ? class {} : HTMLElement) as typeof HTMLElement;

export class UnicelArt extends Base {
  static observedAttributes = ["piece", "fps", "options", "label", "mono", "motion", "time"];
  #playback: Playback | null = null;
  #player: Player | null = null;
  #run = 0;
  #near: Promise<void> | null = null;

  connectedCallback() {
    void this.#start();
  }

  disconnectedCallback() {
    this.#run++;
    this.#teardown();
  }

  attributeChangedCallback(name: string) {
    if (!this.isConnected) return;
    // A new time only seeks: the piece is already made.
    if (name === "time" && this.#playback) this.#hold();
    else void this.#start();
  }

  /** The play time of the frame shown, in seconds. */
  get currentTime(): number {
    return this.#playback?.time() ?? 0;
  }

  #hold() {
    const time = this.getAttribute("time");
    try {
      this.#playback?.hold(time === null || time === "" || Number.isNaN(+time) ? null : Math.max(0, +time));
    } catch (error) {
      console.error(`<unicel-art> ${this.getAttribute("piece")} stopped:`, error);
    }
  }

  #teardown() {
    this.#playback?.stop();
    this.#playback = null;
    const player = this.#player;
    this.#player = null;
    try {
      player?.free();
    } catch {
      // A player whose piece panicked stays borrowed and cannot be freed; its memory is lost either way.
    }
  }

  async #start() {
    const run = ++this.#run;
    const slug = this.getAttribute("piece") ?? "";
    let exports: InitOutput;
    let player: Player;
    try {
      await (this.#near ??= near(this));
      if (run !== this.#run) return;
      ({ player, exports } = await make(slug, this.getAttribute("options") ?? ""));
    } catch (error) {
      console.warn(`<unicel-art> could not load ${slug}:`, error);
      // A piece that was playing is not the one asked for any more: stop it and
      // clear its drawing. With nothing playing yet, whatever the element held
      // (a server-rendered still) stays as the fallback.
      if (run === this.#run && this.#player) {
        this.#teardown();
        this.replaceChildren();
      }
      return;
    }
    // Another attribute change, or removal, while this one loaded.
    if (run !== this.#run) {
      player.free();
      return;
    }
    const meta = JSON.parse(player.meta_json()) as Meta;
    const fps = this.getAttribute("fps");
    const options: MountOptions = { motion: this.hasAttribute("motion") };
    if (fps !== null && fps !== "" && !Number.isNaN(+fps)) options.fps = +fps;
    const el = document.createElement(meta.palette && !this.hasAttribute("mono") ? "canvas" : "pre");
    el.setAttribute("role", "img");
    el.setAttribute("aria-label", this.getAttribute("label") || meta.name);
    this.#teardown();
    this.replaceChildren(el);
    this.#player = player;
    try {
      this.#playback = mount(el, exports, player, options);
    } catch (error) {
      console.error(`<unicel-art> ${slug} stopped on its first frame:`, error);
      return;
    }
    if (this.hasAttribute("time")) this.#hold();
  }
}

/** Resolves once the element is within a screen of the viewport, at once where there is no IntersectionObserver. */
function near(el: Element): Promise<void> {
  if (typeof IntersectionObserver === "undefined") return Promise.resolve();
  return new Promise((resolve) => {
    const seen = new IntersectionObserver(
      (entries) => {
        if (!entries.some((e) => e.isIntersecting)) return;
        seen.disconnect();
        resolve();
      },
      { rootMargin: "100% 0px" },
    );
    seen.observe(el);
  });
}

/** Defines the tag, once. Importing this module calls it for "unicel-art". */
export function define(tag = "unicel-art") {
  if (typeof customElements === "undefined" || customElements.get(tag)) return;
  const style = document.createElement("style");
  style.textContent = tag === "unicel-art" ? STYLE : STYLE.replaceAll("unicel-art", tag);
  document.head.append(style);
  customElements.define(tag, tag === "unicel-art" ? UnicelArt : class extends UnicelArt {});
}

define();

declare global {
  interface HTMLElementTagNameMap {
    "unicel-art": UnicelArt;
  }
}
