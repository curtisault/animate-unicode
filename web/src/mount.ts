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
 * redraws only the cells that changed, and half and full blocks are filled as
 * rectangles so they tile with no seams. A frame with back colours (a scene,
 * two square pixels a cell) is drawn instead as one image of half cells,
 * scaled up, with any other glyphs drawn over it.
 *
 * Play time only advances while the element is on screen and the tab is open,
 * and prefers-reduced-motion keeps the first frame unless `motion` is set.
 * Returns a Playback: stop it, read its time, or hold it on any frame (pieces
 * are deterministic in time, so any frame can be drawn at once).
 */
import type { InitOutput, Player } from "../pkg/modules.js";
import { shapeOf } from "./blocks.ts";

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

/** A mounted piece: stop it, read its play time, or hold it on one frame. */
export interface Playback {
  /** Stops playing and lets go of the element's observers. The player is the caller's to free. */
  stop(): void;
  /** Seconds of play time so far: the time of the frame shown. */
  time(): number;
  /** Holds the piece on the frame at `t` seconds, paused; `null` plays on from there. */
  hold(t: number | null): void;
}

/** The right half of a double-width glyph, in the cells buffer. Must match unicel::WIDE_TAIL. */
export const WIDE_TAIL = 0;
/** No colour behind a cell, in the backs buffer. Must match unicel::NO_BACK. */
export const NO_BACK = 255;

// The bundled font first, as in element.ts's STYLE: one face, one width, for every glyph it has.
const FONT = '"unicel mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace';

export function mount(el: HTMLElement, wasm: InitOutput, player: Player, options: MountOptions = {}): Playback {
  const meta = JSON.parse(player.meta_json()) as Meta;
  const { cols, rows, palette, ground, cell } = meta;
  const fps = options.fps ?? meta.fps;
  const motion = options.motion ?? false;
  const n = cols * rows;
  const canvas = el instanceof HTMLCanvasElement ? el : null;

  // The grid, as views over wasm memory. Taken fresh each frame: the buffer moves if memory grows.
  const cells = () => new Uint32Array(wasm.memory.buffer, player.cells_ptr(), n);
  const colors = () => new Uint8Array(wasm.memory.buffer, player.colors_ptr(), n);
  const backs = () => new Uint8Array(wasm.memory.buffer, player.backs_ptr(), n);

  const rgb = (css: string) => (css[0] === "#" ? [1, 3, 5].map((i) => parseInt(css.slice(i, i + 2), 16)) : (css.match(/[\d.]+/g) || []).map(Number));
  const dark = (css: string) => {
    const c = rgb(css);
    return 0.2126 * (c[0] ?? 0) + 0.7152 * (c[1] ?? 0) + 0.0722 * (c[2] ?? 0) < 128;
  };
  // In a <pre>, or on a canvas without a palette, the piece is one ink.
  player.set_mono(!(canvas && palette));
  const paper = () => (canvas && ground ? !dark(ground) : dark(getComputedStyle(el).color));

  // A piece that shows the time is told the wall clock before each frame.
  const frame = () => {
    if (meta.clock) player.set_clock(Date.now(), -new Date().getTimezoneOffset());
    player.frame(t, paper());
  };

  let t = 0;
  let held: number | null = null;
  let draw = () => {
    frame();
    el.textContent = player.text();
  };

  let ro: ResizeObserver | undefined;
  let unfont = () => {};
  if (canvas) {
    const ctx = canvas.getContext("2d")!;
    const atlas = document.createElement("canvas");
    const actx = atlas.getContext("2d")!;
    // slot index by (code point, colour, wide)
    const slots = new Map<number, number>();
    let w = 0, h = 0, sw = 0, sh = 0, pw = 0, ph = 0, width = -1, ink = "";
    let xs = new Int32Array(0), ys = new Int32Array(0), gx = new Int32Array(0), gy = new Int32Array(0);
    let last = new Uint32Array(0), lastColor = new Uint8Array(0), lastBack = new Uint8Array(0), full = true;
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
      const shape = wide ? undefined : shapeOf(cp);
      if (!shape) {
        actx.fillText(String.fromCodePoint(cp), x + (wide ? sw : sw / 2), y + sh / 2);
      } else {
        // Filled a pixel past the cell each way, so the copy out of the slot, rounded, never shows a gap.
        const bx = x - 1, by = y - 1, bw = sw + 2, bh = sh + 2;
        if ("mask" in shape) {
          for (let d = 0; d < shape.w * shape.h; d++) {
            if (!((shape.mask >> d) & 1)) continue;
            const cx = d % shape.w, cy = (d - cx) / shape.w;
            const x0 = bx + Math.round((cx * bw) / shape.w), y0 = by + Math.round((cy * bh) / shape.h);
            actx.fillRect(x0, y0, bx + Math.round(((cx + 1) * bw) / shape.w) - x0, by + Math.round(((cy + 1) * bh) / shape.h) - y0);
          }
        } else {
          actx.beginPath();
          for (let k = 0; k < shape.poly.length; k += 2) actx.lineTo(bx + shape.poly[k]! * bw, by + shape.poly[k + 1]! * bh);
          actx.fill();
        }
      }
      actx.restore();
      slots.set(key, s);
      return s;
    };
    // A frame with back colours (a scene: two square pixels a cell) is drawn as one image, a pixel for each half
    // cell, scaled up in one call; only the cells holding some other glyph are then drawn over it. Cell by cell,
    // its tens of thousands of rectangles would cost far more.
    const image = document.createElement("canvas");
    const ictx = image.getContext("2d")!;
    const prgb = (palette ?? []).map((c) => rgb(c));
    const grgb = ground ? rgb(ground) : undefined;
    let pixels: ImageData | undefined;
    const drawPixels = (c: Uint32Array, k: Uint8Array, b: Uint8Array) => {
      if (!pixels) {
        image.width = cols;
        image.height = rows * 2;
        pixels = ictx.createImageData(cols, rows * 2);
      }
      const d = pixels.data;
      const put = (o: number, p: number[] | undefined) => {
        d[o] = p?.[0] ?? 0;
        d[o + 1] = p?.[1] ?? 0;
        d[o + 2] = p?.[2] ?? 0;
        d[o + 3] = p ? 255 : 0;
      };
      const glyphs: number[] = [];
      for (let i = 0; i < n; i++) {
        const cp = c[i]!;
        const x = i % cols, y = (i - x) / cols;
        const behind = b[i] !== NO_BACK ? prgb[b[i]!] : cp === WIDE_TAIL ? (b[i - 1] !== NO_BACK ? prgb[b[i - 1]!] : grgb) : grgb;
        const ink = prgb[k[i]!] ?? prgb[0];
        const top = cp === 0x2580 || cp === 0x2588 ? ink : behind;
        const bottom = cp === 0x2584 || cp === 0x2588 ? ink : behind;
        put((2 * y * cols + x) * 4, top);
        put(((2 * y + 1) * cols + x) * 4, bottom);
        if (cp !== 0x20 && cp !== WIDE_TAIL && cp !== 0x2580 && cp !== 0x2584 && cp !== 0x2588) glyphs.push(i);
      }
      ictx.putImageData(pixels, 0, 0);
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(image, 0, 0, canvas.width, canvas.height);
      for (const i of glyphs) {
        const x = i % cols, y = (i - x) / cols;
        const wide = x + 1 < cols && c[i + 1] === WIDE_TAIL;
        const x0 = xs[x]!, y0 = ys[y]!, cw = xs[x + (wide ? 2 : 1)]! - x0, ch = ys[y + 1]! - y0;
        const s = glyph(c[i]!, k[i]!, wide);
        ctx.drawImage(atlas, (s % 32) * pw + 1 + x0 - gx[x]!, Math.floor(s / 32) * ph + 1 + y0 - gy[y]!, cw, ch, x0, y0, cw, ch);
      }
    };
    draw = () => {
      if (!palette && getComputedStyle(canvas).color !== ink) {
        ink = getComputedStyle(canvas).color;
        slots.clear();
        actx.clearRect(0, 0, atlas.width, atlas.height);
        full = true;
      }
      frame();
      const c = cells(), k = colors(), b = backs();
      if (palette && b.some((v) => v !== NO_BACK)) {
        drawPixels(c, k, b);
        full = true; // a frame without back colours starts over
        return;
      }
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
        const back = palette && b[i] !== NO_BACK ? palette[b[i]!] : undefined;
        const changed = full || cp !== last[i] || k[i] !== lastColor[i] || b[i] !== lastBack[i] || (wide && c[i + 1] !== last[i + 1]);
        if (!changed) continue;
        const x0 = xs[x]!, y0 = ys[y]!, cw = xs[x + (wide ? 2 : 1)]! - x0, ch = ys[y + 1]! - y0;
        if (back) {
          ctx.fillStyle = back;
          ctx.fillRect(x0, y0, cw, ch);
          if (ground) ctx.fillStyle = ground;
        } else if (!full) {
          if (ground) ctx.fillRect(x0, y0, cw, ch);
          else ctx.clearRect(x0, y0, cw, ch);
        }
        // Half and full blocks are filled exactly, so cells of them tile with no seams.
        const half = cp === 0x2580 ? 1 : cp === 0x2584 ? 2 : cp === 0x2588 ? 3 : 0;
        if (half && palette) {
          ctx.fillStyle = palette[k[i]!] ?? palette[0]!;
          const mid = Math.round(ch / 2);
          ctx.fillRect(x0, half === 2 ? y0 + mid : y0, cw, half === 1 ? mid : half === 2 ? ch - mid : ch);
          if (ground) ctx.fillStyle = ground;
        } else if (cp !== 0x20) {
          const s = glyph(cp, k[i]!, wide);
          ctx.drawImage(atlas, (s % 32) * pw + 1 + x0 - gx[x]!, Math.floor(s / 32) * ph + 1 + y0 - gy[y]!, cw, ch, x0, y0, cw, ch);
        }
      }
      if (last.length !== n) {
        last = new Uint32Array(n);
        lastColor = new Uint8Array(n);
        lastBack = new Uint8Array(n);
      }
      last.set(c);
      lastColor.set(k);
      lastBack.set(b);
      full = false;
    };
    size();
    // A canvas does not wait for a web font: glyphs drawn before the bundled font arrives come from a fallback face
    // and stay in the atlas. Ask for it (a canvas alone never would), and start the atlas over when any font loads.
    const fonts = typeof document !== "undefined" ? document.fonts : undefined;
    const refont = () => {
      slots.clear();
      actx.clearRect(0, 0, atlas.width, atlas.height);
      full = true;
      try {
        draw(); // now, for a still; an animated piece would anyway on its next frame
      } catch {
        // a piece that panics is stopped and reported by the frame loop
      }
    };
    fonts?.addEventListener("loadingdone", refont);
    fonts?.load(`${Math.max(8, Math.round(w / 0.6))}px "unicel mono"`).catch(() => {});
    unfont = () => fonts?.removeEventListener("loadingdone", refont);
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
    unfont();
    throw error;
  }
  if (!fps)
    return {
      stop: () => {
        ro?.disconnect();
        unfont();
      },
      time: () => t,
      hold: (at) => {
        if (at === null) return;
        t = at;
        draw();
      },
    };

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
    const go = held === null && !broken && seen && !document.hidden && (motion || !still.matches);
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

  return {
    stop: () => {
      io.disconnect();
      ro?.disconnect();
      unfont();
      cancelAnimationFrame(raf);
      raf = 0;
      document.removeEventListener("visibilitychange", run);
      still.removeEventListener("change", run);
    },
    time: () => t,
    hold: (at) => {
      held = at;
      if (at !== null && !broken) {
        t = at;
        try {
          draw();
        } catch (error) {
          broken = true;
          throw error;
        }
      }
      run();
    },
  };
}
