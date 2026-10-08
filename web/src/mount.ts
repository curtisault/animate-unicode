/*
 * mount: plays a Player (a piece in wasm) in a <pre>, or on a <canvas> in colour.
 *
 * After mount.ts in ascii.rest by @bas3line (https://github.com/bas3line), MIT
 * licensed, with the frame read as code points out of wasm memory instead of a
 * string: an astral glyph is one cell, and a WIDE_TAIL cell is the right half
 * of the double-width glyph before it.
 *
 * In a <pre> the piece is text in the pre's own colour. On a <canvas> it is
 * drawn in its palette (or the canvas's text colour) over `meta.ground`,
 * filling the canvas's width with cells `meta.cell` widths tall. Each
 * (glyph, colour) is drawn once into an atlas and copied from there; a frame
 * redraws only the cells that changed.
 *
 * Play time only advances while the element is on screen and the tab is open,
 * and prefers-reduced-motion keeps the first frame unless `motion` is set.
 * Returns a stop function.
 */
import type { InitOutput, Player } from "../pkg/animate_unicode.js";

export interface Meta {
  name: string;
  slug: string;
  category: string;
  note: string;
  cols: number;
  rows: number;
  fps: number;
  charset: "basic" | "extended" | "wide";
  options: string | null;
  clock: boolean;
  palette: string[] | null;
  ground: string | null;
  cell: 1 | 2;
}

export interface MountOptions {
  /** Overrides the piece's frame rate. */
  fps?: number;
  /** Plays even when the reader prefers reduced motion: only for a page with its own play control. */
  motion?: boolean;
}

/** The right half of a double-width glyph, in the cells buffer. Must match animate_unicode::WIDE_TAIL. */
export const WIDE_TAIL = 0;

const FONT = 'ui-monospace, SFMono-Regular, Menlo, Consolas, "animate-unicode mono", monospace';

export function mount(el: HTMLElement, wasm: InitOutput, player: Player, options: MountOptions = {}): () => void {
  const meta = JSON.parse(player.meta_json()) as Meta;
  const { cols, rows, palette, ground, cell } = meta;
  const fps = options.fps ?? meta.fps;
  const motion = options.motion ?? false;
  const n = cols * rows;
  const canvas = el instanceof HTMLCanvasElement ? el : null;

  // The grid, as views over wasm memory. Taken fresh each frame: the buffer moves if memory grows.
  const cells = () => new Uint32Array(wasm.memory.buffer, player.cells_ptr(), n);
  const colors = () => new Uint8Array(wasm.memory.buffer, player.colors_ptr(), n);

  const rgb = (css: string) => (css[0] === "#" ? [1, 3, 5].map((i) => parseInt(css.slice(i, i + 2), 16)) : (css.match(/[\d.]+/g) || []).map(Number));
  const dark = (css: string) => {
    const c = rgb(css);
    return 0.2126 * (c[0] ?? 0) + 0.7152 * (c[1] ?? 0) + 0.0722 * (c[2] ?? 0) < 128;
  };
  const paper = () => (canvas && ground ? !dark(ground) : dark(getComputedStyle(el).color));

  let t = 0;
  let draw = () => {
    player.frame(t, paper());
    el.textContent = player.text();
  };

  let ro: ResizeObserver | undefined;
  if (canvas) {
    const ctx = canvas.getContext("2d")!;
    const atlas = document.createElement("canvas");
    const actx = atlas.getContext("2d")!;
    // slot index by (code point, colour, wide)
    const slots = new Map<number, number>();
    let w = 0, h = 0, sw = 0, sh = 0, pw = 0, ph = 0, width = -1, ink = "";
    let xs = new Int32Array(0), ys = new Int32Array(0), gx = new Int32Array(0), gy = new Int32Array(0);
    let last = new Uint32Array(0), lastColor = new Uint8Array(0), full = true;
    canvas.style.display ||= "block";
    canvas.style.width ||= "100%";
    canvas.style.aspectRatio = `${cols} / ${rows * cell}`;
    const size = () => {
      width = canvas.clientWidth;
      w = (width * (devicePixelRatio || 1)) / cols;
      h = w * cell;
      sw = Math.ceil(w);
      sh = Math.ceil(h);
      // A slot is two cells wide so a wide glyph fits; a narrow one uses the left half.
      pw = 2 * sw + 2;
      ph = sh + 2;
      canvas.width = Math.round(w * cols);
      canvas.height = Math.round(h * rows);
      atlas.width = pw * 32;
      atlas.height = ph * 32;
      xs = Int32Array.from({ length: cols + 1 }, (_, x) => Math.round(x * w));
      ys = Int32Array.from({ length: rows + 1 }, (_, y) => Math.round(y * h));
      gx = Int32Array.from({ length: cols }, (_, x) => Math.round(x * w + (w - sw) / 2));
      gy = Int32Array.from({ length: rows }, (_, y) => Math.round(y * h + (h - sh) / 2));
      slots.clear();
      full = true;
    };
    const glyph = (cp: number, color: number, wide: boolean) => {
      const key = (cp * 256 + color) * 2 + (wide ? 1 : 0);
      let s = slots.get(key);
      if (s !== undefined) return s;
      if (slots.size === 1024) {
        actx.clearRect(0, 0, atlas.width, atlas.height);
        slots.clear();
      }
      s = slots.size;
      const x = (s % 32) * pw + 1, y = Math.floor(s / 32) * ph + 1;
      actx.font = `${w / 0.6}px ${FONT}`;
      actx.textAlign = "center";
      actx.textBaseline = "middle";
      actx.fillStyle = palette ? (palette[color] ?? palette[0]!) : ink;
      actx.save();
      actx.beginPath();
      actx.rect(x - 1, y - 1, pw, ph);
      actx.clip();
      actx.fillText(String.fromCodePoint(cp), x + (wide ? sw : sw / 2), y + sh / 2);
      actx.restore();
      slots.set(key, s);
      return s;
    };
    draw = () => {
      if (!palette && getComputedStyle(canvas).color !== ink) {
        ink = getComputedStyle(canvas).color;
        slots.clear();
        actx.clearRect(0, 0, atlas.width, atlas.height);
        full = true;
      }
      player.frame(t, paper());
      const c = cells(), k = colors();
      if (ground) ctx.fillStyle = ground;
      if (full) {
        if (ground) ctx.fillRect(0, 0, canvas.width, canvas.height);
        else ctx.clearRect(0, 0, canvas.width, canvas.height);
      }
      for (let i = 0; i < n; i++) {
        const cp = c[i]!;
        if (cp === WIDE_TAIL) continue;
        const x = i % cols, y = (i - x) / cols;
        const wide = x + 1 < cols && c[i + 1] === WIDE_TAIL;
        const changed = full || cp !== last[i] || k[i] !== lastColor[i] || (wide && c[i + 1] !== last[i + 1]);
        if (!changed) continue;
        const x0 = xs[x]!, y0 = ys[y]!, cw = xs[x + (wide ? 2 : 1)]! - x0, ch = ys[y + 1]! - y0;
        if (!full) {
          if (ground) ctx.fillRect(x0, y0, cw, ch);
          else ctx.clearRect(x0, y0, cw, ch);
        }
        if (cp !== 0x20) {
          const s = glyph(cp, k[i]!, wide);
          ctx.drawImage(atlas, (s % 32) * pw + 1 + x0 - gx[x]!, Math.floor(s / 32) * ph + 1 + y0 - gy[y]!, cw, ch, x0, y0, cw, ch);
        }
      }
      if (last.length !== n) {
        last = new Uint32Array(n);
        lastColor = new Uint8Array(n);
      }
      last.set(c);
      lastColor.set(k);
      full = false;
    };
    size();
    ro = new ResizeObserver(() => {
      if (canvas.clientWidth !== width) {
        size();
        draw();
      }
    });
    ro.observe(canvas);
  }
  try {
    draw();
  } catch (error) {
    ro?.disconnect();
    throw error;
  }
  if (!fps) return () => ro?.disconnect();

  const still = matchMedia("(prefers-reduced-motion: reduce)");
  let raf = 0;
  let lastNow = 0;
  let seen = false;
  // Set when a frame throws: a piece that panicked leaves its player unusable
  // (crates/wasm/src/lib.rs, "Panics"), so stop instead of throwing every frame.
  let broken = false;
  const tick = (now: number) => {
    raf = requestAnimationFrame(tick);
    const dt = now - lastNow;
    if (dt < 1000 / fps - 2) return;
    lastNow = now;
    t += Math.min(dt, 100) / 1000;
    try {
      draw();
    } catch (error) {
      broken = true;
      run();
      throw error;
    }
  };
  const run = () => {
    const go = !broken && seen && !document.hidden && (motion || !still.matches);
    if (go && !raf) {
      lastNow = performance.now();
      raf = requestAnimationFrame(tick);
    } else if (!go && raf) {
      cancelAnimationFrame(raf);
      raf = 0;
    }
  };
  const io = new IntersectionObserver((entries) => {
    seen = entries[entries.length - 1]!.isIntersecting;
    run();
  });
  io.observe(el);
  document.addEventListener("visibilitychange", run);
  still.addEventListener("change", run);

  return () => {
    io.disconnect();
    ro?.disconnect();
    cancelAnimationFrame(raf);
    raf = 0;
    document.removeEventListener("visibilitychange", run);
    still.removeEventListener("change", run);
  };
}
