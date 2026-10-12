/*
 * frame: draws the grid a player holds onto any 2D canvas, in one call, with
 * no state kept between calls: for stills and exports (a PNG, each frame of a
 * video or a GIF) rather than for playing, which mount does with its atlas.
 *
 * Back colours are filled first, block-like glyphs (octants, sextants,
 * quadrants, wedges, half and full blocks) are filled as exact shapes so they
 * tile without seams, and every other glyph is drawn from the bundled font.
 */
import type { InitOutput, Player } from "../pkg/modules.js";
import { shapeOf } from "./blocks.ts";
import { NO_BACK, WIDE_TAIL, type Meta } from "./mount.ts";

/** How to draw a frame. */
export interface Look {
  /** One ink for every glyph, and no back colours: the piece as text. */
  mono: boolean;
  /** The ink of a piece drawn in one ink (or one with no palette). */
  ink: string;
  /** What the cells sit on: the piece's ground for a coloured piece, or the page's. Undefined leaves the canvas as it is. */
  ground?: string;
}

// The bundled font first, as in element.ts's STYLE.
const FONT = '"unicel mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace';

/**
 * Draws the player's current grid into the box (`x`, `y`, `w`, `h`) of `ctx`,
 * a cell `w / cols` wide and `h / rows` tall. Call `player.frame` first.
 */
export function paint(
  ctx: CanvasRenderingContext2D,
  wasm: InitOutput,
  player: Player,
  meta: Meta,
  box: { x: number; y: number; w: number; h: number },
  look: Look,
): void {
  const { cols, rows, palette } = meta;
  const n = cols * rows;
  const cells = new Uint32Array(wasm.memory.buffer, player.cells_ptr(), n);
  const colors = new Uint8Array(wasm.memory.buffer, player.colors_ptr(), n);
  const backs = new Uint8Array(wasm.memory.buffer, player.backs_ptr(), n);
  const colour = !look.mono && palette !== null;
  const ink = (k: number) => (colour ? (palette![k] ?? palette![0]!) : look.ink);
  // Cell edges on whole pixels, so neighbours meet exactly.
  const xs = Array.from({ length: cols + 1 }, (_, i) => Math.round(box.x + (i * box.w) / cols));
  const ys = Array.from({ length: rows + 1 }, (_, i) => Math.round(box.y + (i * box.h) / rows));
  if (look.ground) {
    ctx.fillStyle = look.ground;
    ctx.fillRect(box.x, box.y, box.w, box.h);
  }
  const size = box.w / cols / 0.6;
  ctx.font = `${size}px ${FONT}`;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  for (let i = 0; i < n; i++) {
    const cp = cells[i]!;
    if (cp === WIDE_TAIL) continue;
    const c = i % cols, r = (i - c) / cols;
    const wide = c + 1 < cols && cells[i + 1] === WIDE_TAIL;
    const x0 = xs[c]!, y0 = ys[r]!, cw = xs[c + (wide ? 2 : 1)]! - x0, ch = ys[r + 1]! - y0;
    if (colour && backs[i] !== NO_BACK) {
      ctx.fillStyle = palette![backs[i]!] ?? palette![0]!;
      ctx.fillRect(x0, y0, cw, ch);
    }
    if (cp === 0x20) continue;
    ctx.fillStyle = ink(colors[i]!);
    const shape = wide ? undefined : shapeOf(cp);
    if (!shape) {
      ctx.fillText(String.fromCodePoint(cp), x0 + cw / 2, y0 + ch / 2);
    } else if ("mask" in shape) {
      for (let d = 0; d < shape.w * shape.h; d++) {
        if (!((shape.mask >> d) & 1)) continue;
        const dx = d % shape.w, dy = (d - dx) / shape.w;
        const ax = x0 + Math.round((dx * cw) / shape.w), ay = y0 + Math.round((dy * ch) / shape.h);
        ctx.fillRect(ax, ay, x0 + Math.round(((dx + 1) * cw) / shape.w) - ax, y0 + Math.round(((dy + 1) * ch) / shape.h) - ay);
      }
    } else {
      ctx.beginPath();
      for (let k = 0; k < shape.poly.length; k += 2) ctx.lineTo(x0 + shape.poly[k]! * cw, y0 + shape.poly[k + 1]! * ch);
      ctx.fill();
    }
  }
}
