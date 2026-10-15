/*
 * Share cards, after ascii.rest's (MIT, @bas3line): one 1200×630 PNG a piece
 * at og/<slug>.png, and a copy of index.html for each piece, <slug>.html, that
 * carries its card in Open Graph and Twitter tags. Link previewers read those
 * tags without running the page's script, so the Elm app alone could never
 * show them; a reader who opens the link gets the same app as from /.
 *
 * Run after `vite build` (`npm run build` does), with web/pkg built: each card
 * is the piece's first frame, drawn in Node by the wasm player and the shell's
 * own paint(), onto @napi-rs/canvas, in the bundled font. A scene fills the
 * card edge to edge with its name along the bottom; anything else sits on the
 * dark ground, as large as it fits.
 *
 * SITE_URL, the site's address ("https://unicel.example"), makes the card's
 * links absolute, as previewers expect, and adds each page's canonical link
 * and a sitemap.xml of every page, named in robots.txt; without it the links
 * are relative, and there is neither.
 */
import { readFileSync, writeFileSync, appendFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createCanvas, GlobalFonts } from "@napi-rs/canvas";
import { paint } from "../../web/src/frame.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const pkg = resolve(root, "web/pkg");
const dist = resolve(root, "site/dist");
const site = (process.env.SITE_URL ?? "").replace(/\/$/, "");
const W = 1200, H = 630, BAND = 84, SIDE = 48;
const GROUND = "#131518", INK = "#e8ebef", MUTED = "#8c95a1";

GlobalFonts.registerFromPath(resolve(root, "web/fonts/unicel-mono.woff2"), "unicel mono");
const { pieces, modules, fallback } = await import(pathToFileURL(resolve(pkg, "modules.js")).href);

const loaded = new Map();
async function module(name) {
  if (!loaded.has(name)) {
    const mod = await import(pathToFileURL(resolve(pkg, name, "unicel.js")).href);
    const wasm = await mod.default({ module_or_path: readFileSync(resolve(pkg, name, "unicel_bg.wasm")) });
    loaded.set(name, { mod, wasm });
  }
  return loaded.get(name);
}

/** The card for one piece: its first frame, its name, and the site's (or `tag` in its place). */
async function card(meta) {
  const { mod, wasm } = await module(modules[meta.category] ?? fallback);
  const player = new mod.Player(meta.slug, "");
  try {
    const colour = meta.palette !== null;
    player.set_mono(!colour);
    player.frame(0, false);
    const canvas = createCanvas(W, H);
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = GROUND;
    ctx.fillRect(0, 0, W, H);
    const scene = colour && meta.ground !== null && meta.cols * 2 >= meta.rows * meta.cell * 3;
    const aspect = (meta.rows * meta.cell) / meta.cols;
    let box;
    if (scene) {
      const h = Math.round(W * aspect);
      box = { x: 0, y: Math.min(0, Math.round((H - BAND - h) / 2)), w: W, h };
    } else {
      const room = { w: W - 2 * SIDE, h: H - BAND - SIDE };
      const w = Math.min(room.w, room.h / aspect);
      box = { x: Math.round((W - w) / 2), y: Math.round(SIDE / 2 + (room.h - w * aspect) / 2), w: Math.round(w), h: Math.round(w * aspect) };
    }
    paint(ctx, wasm, player, meta, box, { mono: !colour, ink: INK, ground: colour ? (meta.ground ?? undefined) : undefined });
    // The name along the bottom, on a band over a scene.
    if (scene) {
      ctx.fillStyle = "rgba(19, 21, 24, 0.82)";
      ctx.fillRect(0, H - BAND, W, BAND);
    }
    ctx.textBaseline = "middle";
    ctx.font = '34px "unicel mono"';
    ctx.fillStyle = INK;
    ctx.textAlign = "left";
    ctx.fillText(meta.name, SIDE, H - BAND / 2);
    ctx.font = '24px "unicel mono"';
    ctx.fillStyle = MUTED;
    ctx.textAlign = "right";
    ctx.fillText(meta.tag ?? `unicel · ${meta.category}`, W - SIDE, H - BAND / 2);
    return canvas.toBuffer("image/png");
  } finally {
    player.free();
  }
}

const attr = (s) => s.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");

/** index.html with a page's title and its card's tags. */
function page(html, { title, description, image, url }) {
  const tags = [
    `<meta name="description" content="${attr(description)}" />`,
    `<meta property="og:site_name" content="unicel" />`,
    `<meta property="og:type" content="website" />`,
    `<meta property="og:title" content="${attr(title)}" />`,
    `<meta property="og:description" content="${attr(description)}" />`,
    `<meta property="og:image" content="${attr(image)}" />`,
    `<meta property="og:image:width" content="${W}" />`,
    `<meta property="og:image:height" content="${H}" />`,
    `<meta property="og:url" content="${attr(url)}" />`,
    `<meta name="twitter:card" content="summary_large_image" />`,
    ...(site ? [`<link rel="canonical" href="${attr(url)}" />`] : []),
  ].join("\n    ");
  return html.replace(/<title>[^<]*<\/title>/, `<title>${attr(title)}</title>\n    ${tags}`);
}

const index = readFileSync(resolve(dist, "index.html"), "utf8");
mkdirSync(resolve(dist, "og"), { recursive: true });
let made = 0;
for (const meta of pieces) {
  writeFileSync(resolve(dist, "og", `${meta.slug}.png`), await card(meta));
  writeFileSync(
    resolve(dist, `${meta.slug}.html`),
    page(index, { title: `${meta.name} · unicel`, description: meta.note, image: `${site}/og/${meta.slug}.png`, url: `${site}/${meta.slug}` }),
  );
  made++;
}
// The front page's card is a scene's, with the site's own words.
const front = pieces.find((m) => m.slug === "night-coast") ?? pieces[0];
writeFileSync(resolve(dist, "og", "index.png"), await card({ ...front, name: "unicel: animated unicode art", tag: `${pieces.length} pieces` }));
writeFileSync(
  resolve(dist, "index.html"),
  page(index, {
    title: "unicel: animated unicode art for web pages",
    description: `${pieces.length} pieces drawn in braille, blocks, box drawing and other unicode, each one HTML tag.`,
    image: `${site}/og/index.png`,
    url: `${site}/`,
  }),
);
// A sitemap needs whole addresses, so only with SITE_URL.
if (site) {
  const urls = ["/", ...pieces.map((m) => `/${m.slug}`)].map((path) => `  <url><loc>${attr(site + path)}</loc></url>`);
  writeFileSync(
    resolve(dist, "sitemap.xml"),
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.join("\n")}\n</urlset>\n`,
  );
  appendFileSync(resolve(dist, "robots.txt"), `\nSitemap: ${site}/sitemap.xml\n`);
}
console.log(`${made} share cards and pages in site/dist${site ? ", and a sitemap" : " (SITE_URL unset: their links are relative, and there is no sitemap)"}`);
