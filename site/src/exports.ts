/*
 * A piece's downloads, made in the browser: a PNG of one frame, a video and a
 * GIF, and the frame as text: plain, for the clipboard or a .txt, or with
 * 24-bit colour escapes, an .ans a terminal shows in colour. After ascii.rest's (MIT, @bas3line), with one difference: a piece here
 * is deterministic in time, so every frame is drawn on demand off screen and
 * encoded as fast as it is drawn, rather than recorded in real time. Video is
 * an MP4 through the browser's VideoEncoder (WebCodecs) and mp4-muxer, or a
 * real-time WebM recording where there is no VideoEncoder.
 *
 * A scene fills its picture edge to edge, 1280 px wide; anything else sits on
 * the page's well with a margin round it, its cells up to 16 px wide.
 */
import { NO_BACK, paint, player as make, WIDE_TAIL, type Meta } from "unicel";

/** What a download is of: the piece as the page shows it. */
export interface Take {
  slug: string;
  /** JSON option overrides, or "" for the defaults. */
  options: string;
  /** In one ink, as text, rather than in colour. */
  mono: boolean;
  /** Dark ink on a light ground (the page's look in one ink, or for a piece without a palette). */
  paper: boolean;
  /** The page's colours: its ink, and the well a piece sits on. */
  ink: string;
  well: string;
  /** Seconds of play: the frame for a PNG, where a video or GIF starts. */
  time: number;
}

/** A player for the take, its module's memory, its meta, and how to draw it. */
async function setup(take: Take) {
  const { player, exports } = await make(take.slug, take.options);
  const meta = JSON.parse(player.meta_json()) as Meta;
  const colour = meta.palette !== null && !take.mono;
  player.set_mono(!colour);
  // A coloured piece shades for its own ground, as on the page.
  const paper = colour && meta.ground ? !dark(meta.ground) : take.paper;
  const look = { mono: !colour, ink: take.ink, ground: (colour && meta.ground) || take.well };
  const draw = (ctx: CanvasRenderingContext2D, t: number, scale: number) => {
    if (meta.clock) player.set_clock(Date.now(), -new Date().getTimezoneOffset());
    player.frame(t, paper);
    const l = layout(meta, colour && meta.ground !== null, scale);
    ctx.fillStyle = take.well;
    ctx.fillRect(0, 0, l.w, l.h);
    paint(ctx, exports, player, meta, l.art, look);
  };
  return { player, meta, draw, layout: (scale: number) => layout(meta, colour && meta.ground !== null, scale) };
}

/** True for a dark colour, given as #rrggbb or rgb(r, g, b). */
function dark(css: string) {
  const [r = 0, g = 0, b = 0] = css[0] === "#" ? [1, 3, 5].map((i) => parseInt(css.slice(i, i + 2), 16)) : (css.match(/[\d.]+/g) ?? []).map(Number);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b < 128;
}

/** The picture's size at `scale`, and the box the art fills in it. */
function layout(meta: Meta, scene: boolean, scale: number) {
  const { cols, rows, cell } = meta;
  const even = (v: number) => 2 * Math.ceil(v / 2);
  let w: number, h: number, art: { x: number; y: number; w: number; h: number };
  if (scene) {
    w = 1280;
    h = even((w * rows * cell) / cols);
    art = { x: 0, y: 0, w, h };
  } else {
    // cells up to 16 px wide, the art up to 1040 by 720, and 48 px of well round it
    const size = Math.min(16, 1040 / cols, 720 / (rows * cell));
    const aw = Math.round(cols * size), ah = Math.round(rows * cell * size);
    w = even(aw + 96);
    h = even(ah + 96);
    art = { x: Math.floor((w - aw) / 2), y: Math.floor((h - ah) / 2), w: aw, h: ah };
  }
  return { w: w * scale, h: h * scale, art: { x: art.x * scale, y: art.y * scale, w: art.w * scale, h: art.h * scale } };
}

function canvas(w: number, h: number) {
  const c = document.createElement("canvas");
  c.width = w;
  c.height = h;
  return { canvas: c, ctx: c.getContext("2d", { willReadFrequently: true })! };
}

/** The bundled font, loaded before the first glyph is drawn: a canvas never asks for one by itself. */
async function font(px: number) {
  await document.fonts?.load(`${Math.max(8, Math.round(px))}px "unicel mono"`).catch(() => {});
}

/** How long a loop is: a logo's sheen comes round every `shine` seconds (5 by default); anything else, `fallback`. */
export function loop(meta: Meta, options: string, fallback: number): number {
  if (!["logos", "companies", "distros"].includes(meta.category)) return fallback;
  let shine = 5;
  try {
    const set = JSON.parse(options || meta.options || "{}") as { shine?: number };
    if (typeof set.shine === "number" && set.shine > 0) shine = Math.max(set.shine, 2.5);
  } catch {
    // the defaults, then
  }
  return shine;
}

/** One frame at twice the size, as a PNG. */
export async function png(take: Take): Promise<Blob> {
  const s = await setup(take);
  try {
    const l = s.layout(2);
    await font(l.art.w / s.meta.cols / 0.6);
    const { canvas: c, ctx } = canvas(l.w, l.h);
    s.draw(ctx, take.time, 2);
    return await new Promise<Blob>((resolve, reject) => c.toBlob((b) => (b ? resolve(b) : reject(new Error("no PNG"))), "image/png"));
  } finally {
    s.player.free();
  }
}

/** A loop as a GIF at up to 15 fps and 640 px wide, one palette for every frame. `part` hears progress, 0 to 1. */
export async function gif(take: Take, part: (done: number) => void): Promise<Blob> {
  const { GIFEncoder, quantize, applyPalette } = await import("gifenc");
  const s = await setup(take);
  try {
    const full = s.layout(1);
    const scale = Math.min(1, 640 / full.w);
    const l = s.layout(scale);
    const w = Math.round(l.w), h = Math.round(l.h);
    await font(l.art.w / s.meta.cols / 0.6);
    const fps = Math.min(15, s.meta.fps || 1);
    const n = s.meta.fps ? Math.round(loop(s.meta, take.options, 4) * fps) : 1;
    const { ctx } = canvas(w, h);
    const grab = (i: number) => {
      s.draw(ctx, take.time + i / fps, scale);
      return ctx.getImageData(0, 0, w, h).data;
    };
    // One palette, from a few frames across the loop.
    const samples = [0, 0.25, 0.5, 0.75].map((f) => grab(Math.floor(f * n)));
    const pool = new Uint8ClampedArray(samples.reduce((a, d) => a + d.length, 0));
    samples.reduce((at, d) => (pool.set(d, at), at + d.length), 0);
    const palette = quantize(pool, 256);
    const gif = GIFEncoder();
    for (let i = 0; i < n; i++) {
      const data = grab(i);
      gif.writeFrame(applyPalette(data, palette), w, h, { palette: i === 0 ? palette : undefined, delay: Math.round(1000 / fps), repeat: 0 });
      part((i + 1) / n);
      if (i % 4 === 3) await new Promise((r) => setTimeout(r)); // let the page breathe
    }
    gif.finish();
    return new Blob([gif.bytesView() as Uint8Array<ArrayBuffer>], { type: "image/gif" });
  } finally {
    s.player.free();
  }
}

/** Seconds of video: a logo's loop, or ten seconds of anything else. */
const VIDEO = 10;

/** A loop as video at the piece's own frame rate: MP4 frame by frame, or a real-time WebM where WebCodecs is missing. */
export async function video(take: Take, part: (done: number) => void): Promise<{ blob: Blob; ext: string }> {
  const s = await setup(take);
  try {
    const l = s.layout(1);
    await font(l.art.w / s.meta.cols / 0.6);
    const fps = s.meta.fps || 1;
    const n = Math.round(loop(s.meta, take.options, VIDEO) * fps);
    const { canvas: c, ctx } = canvas(l.w, l.h);
    const config: VideoEncoderConfig = { codec: "avc1.640028", width: l.w, height: l.h, bitrate: 8_000_000, framerate: fps };
    if (typeof VideoEncoder !== "undefined" && (await VideoEncoder.isConfigSupported(config)).supported) {
      const { Muxer, ArrayBufferTarget } = await import("mp4-muxer");
      const muxer = new Muxer({ target: new ArrayBufferTarget(), video: { codec: "avc", width: l.w, height: l.h, frameRate: fps }, fastStart: "in-memory" });
      let failed: unknown = null;
      const encoder = new VideoEncoder({ output: (chunk, meta) => muxer.addVideoChunk(chunk, meta), error: (e) => (failed = e) });
      encoder.configure(config);
      for (let i = 0; i < n; i++) {
        s.draw(ctx, take.time + i / fps, 1);
        const frame = new VideoFrame(c, { timestamp: Math.round((i * 1e6) / fps), duration: Math.round(1e6 / fps) });
        encoder.encode(frame, { keyFrame: i % (fps * 2) === 0 });
        frame.close();
        part((i + 1) / n);
        if (encoder.encodeQueueSize > 8) await new Promise((r) => setTimeout(r, 5));
        if (failed) throw failed;
      }
      await encoder.flush();
      if (failed) throw failed;
      muxer.finalize();
      return { blob: new Blob([muxer.target.buffer], { type: "video/mp4" }), ext: "mp4" };
    }
    // No WebCodecs: record the canvas as the frames are drawn, in real time.
    const type = ["video/webm;codecs=vp9", "video/webm", "video/mp4"].find((t) => MediaRecorder.isTypeSupported(t));
    if (!type) throw new Error("this browser can make neither MP4 nor WebM");
    const rec = new MediaRecorder(c.captureStream(fps), { mimeType: type, videoBitsPerSecond: 8_000_000 });
    const chunks: Blob[] = [];
    rec.ondataavailable = (e) => chunks.push(e.data);
    const stopped = new Promise((r) => (rec.onstop = r));
    rec.start();
    for (let i = 0; i < n; i++) {
      s.draw(ctx, take.time + i / fps, 1);
      part((i + 1) / n);
      await new Promise((r) => setTimeout(r, 1000 / fps));
    }
    rec.stop();
    await stopped;
    return { blob: new Blob(chunks, { type }), ext: type.startsWith("video/mp4") ? "mp4" : "webm" };
  } finally {
    s.player.free();
  }
}

/** The frame shown, as text: the piece in one ink, as a <pre> shows it. */
export async function text(take: Take): Promise<string> {
  const s = await setup({ ...take, mono: true });
  try {
    if (s.meta.clock) s.player.set_clock(Date.now(), -new Date().getTimezoneOffset());
    s.player.frame(take.time, take.paper);
    return s.player.text() + "\n";
  } finally {
    s.player.free();
  }
}

/**
 * The frame shown as text with 24-bit colour escapes (ESC [38;2;r;g;b m for a
 * cell's ink, 48;2 for what is behind it): `cat piece.ans` in a terminal shows
 * it in colour. A piece without a palette, or in one ink, is plain text.
 */
export async function ansi(take: Take): Promise<string> {
  const { player, exports } = await make(take.slug, take.options);
  try {
    const meta = JSON.parse(player.meta_json()) as Meta;
    const { cols, rows, palette } = meta;
    const colour = palette !== null && !take.mono;
    player.set_mono(!colour);
    if (meta.clock) player.set_clock(Date.now(), -new Date().getTimezoneOffset());
    player.frame(take.time, colour && meta.ground ? !dark(meta.ground) : take.paper);
    if (!colour) return player.text() + "\n";
    const n = cols * rows;
    const cells = new Uint32Array(exports.memory.buffer, player.cells_ptr(), n);
    const colors = new Uint8Array(exports.memory.buffer, player.colors_ptr(), n);
    const backs = new Uint8Array(exports.memory.buffer, player.backs_ptr(), n);
    const sgr = (layer: 38 | 48, hex: string) => `\x1b[${layer};2;${[1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16)).join(";")}m`;
    let out = "";
    for (let r = 0; r < rows; r++) {
      let fg = "", bg = "";
      for (let c = 0; c < cols; c++) {
        const i = r * cols + c;
        if (cells[i] === WIDE_TAIL) continue;
        const back = backs[i] !== NO_BACK ? palette![backs[i]!]! : (meta.ground ?? "");
        const ink = palette![colors[i]!] ?? palette![0]!;
        if (back && back !== bg) (out += sgr(48, back)), (bg = back);
        if (cells[i] !== 0x20 && ink !== fg) (out += sgr(38, ink)), (fg = ink);
        out += String.fromCodePoint(cells[i]!);
      }
      out += "\x1b[0m\n";
    }
    return out;
  } finally {
    player.free();
  }
}

/** Saves a blob as a file. */
export function save(blob: Blob, name: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}
