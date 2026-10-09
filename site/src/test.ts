/*
 * The shell checks page (test.html): readouts for what can be measured, the
 * swap and removal buttons, the debug-only tags, and a Markdown record of the
 * manual pass. Plain DOM, no Elm.
 */
import "animate-unicode/element";
import { Player, wasm } from "animate-unicode";

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const frame = () => new Promise<void>((r) => requestAnimationFrame(() => r()));
const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
const say = (el: HTMLElement, ok: boolean | null, text: string) => {
  el.innerHTML = "";
  const span = document.createElement("span");
  if (ok !== null) span.className = ok ? "ok" : "bad";
  span.textContent = (ok === null ? "" : ok ? "ok: " : "check: ") + text;
  el.append(span);
};

// First frame: look before the wasm has loaded. The element only replaces its
// children once its player exists, so the placeholder must still be here now.
const first = $("first");
const placeholder = first.querySelector("pre")!;
const keptWhileLoading = first.firstElementChild === placeholder;
let replacedByPiece = false;
new MutationObserver(() => {
  if (!first.contains(placeholder) && first.firstElementChild?.getAttribute("role") === "img") replacedByPiece = true;
}).observe(first, { childList: true });

const module = await wasm();
const megabytes = () => `${(module.memory.buffer.byteLength / 2 ** 20).toFixed(2)} MB`;

let debugBuild = true;
try {
  new Player("debug-wide", "").free();
} catch {
  debugBuild = false;
}

// The environment, for the record.
const reduced = matchMedia("(prefers-reduced-motion: reduce)");
const dark = matchMedia("(prefers-color-scheme: dark)");
const env = () =>
  `device pixel ratio ${devicePixelRatio} · ${dark.matches ? "dark" : "light"} scheme · reduced motion ${reduced.matches ? "on" : "off"} · ${
    debugBuild ? "debug" : "release"
  } wasm`;
const showEnv = () => {
  $("env").textContent = env();
  say($("motion-readout"), null, `prefers-reduced-motion is ${reduced.matches ? "reduce" : "no-preference"} right now`);
};
showEnv();
reduced.addEventListener("change", showEnv);
dark.addEventListener("change", showEnv);
addEventListener("resize", showEnv); // zooming changes devicePixelRatio and fires resize

// Debug-only tags: made here so a release build logs nothing for them.
for (const spot of document.querySelectorAll<HTMLElement>(".debug-tag")) {
  if (debugBuild) {
    const tag = document.createElement("unicode-art");
    tag.setAttribute("piece", spot.dataset.piece!);
    if (spot.hasAttribute("data-mono")) tag.setAttribute("mono", "");
    spot.replaceWith(tag);
  } else {
    spot.className = "note";
    spot.textContent = "needs the debug wasm build: open this page from `mise run dev`";
    spot.closest("section")?.querySelectorAll("input").forEach((i) => (i.disabled = true));
  }
}

/** Each line's rendered width in a <pre> whose only child is its text. */
const rowWidths = (pre: Element | null | undefined) => {
  const text = pre?.firstChild;
  if (!text || text.nodeType !== Node.TEXT_NODE) return [];
  const range = document.createRange();
  const widths: number[] = [];
  let at = 0;
  for (const line of text.textContent!.split("\n")) {
    range.setStart(text, at);
    range.setEnd(text, at + line.length);
    widths.push(range.getBoundingClientRect().width);
    at += line.length + 1;
  }
  return widths;
};
const spread = (widths: number[]) => {
  const min = Math.min(...widths), max = Math.max(...widths);
  return { even: widths.length > 0 && max - min < 0.5, text: `${widths.length} rows, ${min.toFixed(1)}–${max.toFixed(1)} px wide` };
};

// Braille row widths in the <pre>: equal if the font keeps braille one cell wide.
const braille = $("braille");
setInterval(() => {
  const s = spread(rowWidths(braille.querySelector("pre")));
  say($("braille-rows"), s.even, s.text);
}, 500);

// Every range the bundled font covers, 32 glyphs a row, in it and in system fonts only.
const run = (first: number, step = 1) => String.fromCodePoint(...Array.from({ length: 32 }, (_, i) => first + i * step));
const RANGES: [string, string][] = [
  ["ascii", run(0x30)],
  ["latin", "°·•●".repeat(8)],
  ["arrows", run(0x2190)],
  ["box", run(0x2500)],
  ["box", run(0x2550)],
  ["blocks", run(0x2580)],
  ["shapes", run(0x25a0)],
  ["braille", run(0x2800, 8)],
  ["sextants", run(0x1fb00)],
  ["wedges", run(0x1fb3c)],
  ["octants", run(0x1cd00)],
];
for (const suffix of ["", "-system"]) {
  $(`ranges-labels${suffix}`).textContent = RANGES.map(([label]) => label).join("\n");
}
$("ranges-font").textContent = $("ranges-system").textContent = RANGES.map(([, glyphs]) => glyphs).join("\n");
const FONT = '15px "animate-unicode mono"';
const measureRanges = () => {
  const loaded = document.fonts.check(FONT, "⠿🬀");
  const ours = spread(rowWidths($("ranges-font")));
  say($("ranges-font-readout"), loaded && ours.even, `bundled font ${loaded ? "loaded" : "not loaded"}; ${ours.text}`);
  const system = spread(rowWidths($("ranges-system")));
  say($("ranges-system-readout"), null, `system fonts only: ${system.text}${system.even ? "" : " (uneven, as expected without the font)"}`);
};
void document.fonts.load(FONT, "⠿🬀").then(measureRanges, measureRanges);
setInterval(measureRanges, 1000);

// Changing the piece, at a click and in bulk.
const swap = $("swap");
const swapReadout = $("swap-readout");
say(swapReadout, null, `wasm memory ${megabytes()}`);
for (const b of document.querySelectorAll<HTMLButtonElement>("button[data-piece]")) {
  b.addEventListener("click", async () => {
    swap.setAttribute("piece", b.dataset.piece!);
    await sleep(100);
    say(swapReadout, null, `showing ${b.dataset.piece} · wasm memory ${megabytes()}`);
  });
}
const PIECES = ["donut", "braille-wave", "quadrant-fire"];
$("churn").addEventListener("click", async () => {
  const round = async () => {
    for (let i = 0; i < 120; i++) {
      swap.setAttribute("piece", PIECES[i % PIECES.length]!);
      await frame();
    }
  };
  say(swapReadout, null, "swapping…");
  const start = megabytes();
  await round();
  const warm = module.memory.buffer.byteLength;
  const mid = megabytes();
  await round();
  const grew = module.memory.buffer.byteLength - warm;
  say(swapReadout, grew === 0, `round 1: ${start} → ${mid}; round 2: ${mid} → ${megabytes()}${grew ? ` (grew ${grew} bytes)` : ""}`);
});
$("remove").addEventListener("click", async () => {
  const parent = swap.parentElement!;
  swap.remove();
  await sleep(400);
  parent.append(swap);
});

// First-frame readout, once the tags have had a moment.
await sleep(300);
say(
  $("first-readout"),
  keptWhileLoading && replacedByPiece,
  `placeholder ${keptWhileLoading ? "was still there while the wasm loaded" : "was gone before the wasm loaded"}, and ${
    replacedByPiece ? "was replaced by the piece" : "was not replaced by the piece"
  }`,
);

// The record of the manual pass.
$("copy").addEventListener("click", async () => {
  const lines = ["### Shell checks (site/test.html)", "", `- ${navigator.userAgent}`, `- ${env()}`, ""];
  for (const section of document.querySelectorAll("section")) {
    lines.push(`**${section.querySelector("h2")!.textContent!.trim()}**`);
    const readout = section.querySelector(".readout")?.textContent;
    if (readout) lines.push(`- readout: ${readout}`);
    for (const box of section.querySelectorAll<HTMLInputElement>("input[type=checkbox]")) {
      const label = box.parentElement!.textContent!.trim();
      lines.push(`- [${box.checked ? "x" : " "}] ${label}${box.disabled ? " (not run: release build)" : ""}`);
    }
    lines.push("");
  }
  try {
    await navigator.clipboard.writeText(lines.join("\n"));
    $("copied").textContent = " copied";
  } catch (error) {
    $("copied").textContent = ` could not copy: ${error}`;
  }
});
