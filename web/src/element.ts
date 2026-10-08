/*
 * <unicode-art>: any piece in the library, as one tag. Importing this module
 * defines the tag; on a server, where there is no DOM, it does nothing.
 *
 *   <script type="module" src="/element.js"></script>
 *   <unicode-art piece="donut"></unicode-art>
 *
 * Attributes:
 *   piece    a piece's slug: "donut", "braille-wave"
 *   fps      overrides the piece's frame rate
 *   options  JSON overriding the piece's option defaults: '{"speed":2}'
 *   label    what the picture shows, for screen readers; the piece's name otherwise
 *   mono     draws a coloured piece as text in one ink, like any other
 *
 * Text pieces draw into a <pre> in the element's colour and font size; the
 * coloured ones draw onto a <canvas> as wide as the element. Whatever the
 * element holds before it loads (a first frame rendered on the server, say)
 * stays until the piece is ready.
 */
import { mount, type Meta } from "./mount.ts";
import { Player, wasm } from "./wasm.ts";

// :where gives these no specificity, so any rule of the page's own wins. The bundled font (Phase 5 of the plan)
// covers braille, geometric shapes and legacy computing where the system face has none, one cell wide.
const STYLE =
  '@font-face{font-family:"animate-unicode mono";src:url(/fonts/animate-unicode-mono.woff2) format("woff2");unicode-range:U+00B0,U+00B7,U+2022,U+2190-21FF,U+2500-25FF,U+2800-28FF,U+1FB00-1FBFF;font-display:swap}' +
  ':where(unicode-art){display:block}:where(unicode-art>pre){margin:0;font:inherit;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,"animate-unicode mono",monospace;line-height:1.2;letter-spacing:0;white-space:pre;font-variant-ligatures:none}';

const Base = (typeof HTMLElement === "undefined" ? class {} : HTMLElement) as typeof HTMLElement;

export class UnicodeArt extends Base {
  static observedAttributes = ["piece", "fps", "options", "label", "mono"];
  #stop: (() => void) | null = null;
  #player: Player | null = null;
  #run = 0;

  connectedCallback() {
    void this.#start();
  }

  disconnectedCallback() {
    this.#run++;
    this.#teardown();
  }

  attributeChangedCallback() {
    if (this.isConnected) void this.#start();
  }

  #teardown() {
    this.#stop?.();
    this.#stop = null;
    this.#player?.free();
    this.#player = null;
  }

  async #start() {
    const run = ++this.#run;
    const slug = this.getAttribute("piece") ?? "";
    let module: Awaited<ReturnType<typeof wasm>>;
    let player: Player;
    try {
      module = await wasm();
      player = new Player(slug, this.getAttribute("options") ?? "");
    } catch (error) {
      console.warn(`<unicode-art> could not load ${slug}:`, error);
      return;
    }
    // Another attribute change, or removal, while this one loaded.
    if (run !== this.#run) {
      player.free();
      return;
    }
    const meta = JSON.parse(player.meta_json()) as Meta;
    const fps = this.getAttribute("fps");
    const options = fps !== null && fps !== "" && !Number.isNaN(+fps) ? { fps: +fps } : {};
    const el = document.createElement(meta.palette && !this.hasAttribute("mono") ? "canvas" : "pre");
    el.setAttribute("role", "img");
    el.setAttribute("aria-label", this.getAttribute("label") || meta.name);
    this.#teardown();
    this.replaceChildren(el);
    this.#player = player;
    this.#stop = mount(el, module, player, options);
  }
}

/** Defines the tag, once. Importing this module calls it for "unicode-art". */
export function define(tag = "unicode-art") {
  if (typeof customElements === "undefined" || customElements.get(tag)) return;
  const style = document.createElement("style");
  style.textContent = tag === "unicode-art" ? STYLE : STYLE.replaceAll("unicode-art", tag);
  document.head.append(style);
  customElements.define(tag, tag === "unicode-art" ? UnicodeArt : class extends UnicodeArt {});
}

define();

declare global {
  interface HTMLElementTagNameMap {
    "unicode-art": UnicodeArt;
  }
}
