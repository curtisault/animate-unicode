# animate-unicode: implementation plan

For the session that builds this out. Read it top to bottom once; then work
phase by phase. Every phase ends with acceptance criteria that `mise run check`
or a named command verifies. Nothing in this document is a guess unless marked
**(unverified)**.

- [Goal](#goal)
- [Decisions already made](#decisions-already-made)
- [What the scaffold contains and what state it is in](#what-the-scaffold-contains)
- [Architecture](#architecture)
- [The piece contract](#the-piece-contract)
- [Unicode: the whole point](#unicode)
- [Phases](#phases)
  - [0 · Toolchain green](#phase-0)
  - [1 · Core crate](#phase-1)
  - [2 · Wasm boundary](#phase-2)
  - [3 · Web shell](#phase-3)
  - [4 · Elm site](#phase-4)
  - [5 · Fonts](#phase-5)
  - [6 · Pieces](#phase-6)
  - [7 · CLI](#phase-7)
  - [8 · Packaging, deploy, CI](#phase-8)
- [Testing strategy](#testing-strategy)
- [Performance budgets](#performance-budgets)
- [Risks and open questions](#risks-and-open-questions)
- [Appendix A: what to borrow from ../ascii](#appendix-a)
- [Appendix B: size measurements](#appendix-b)
- [Appendix C: glyph encodings](#appendix-c)

---

## Goal

A library and site in the spirit of [ascii.rest](https://ascii.rest)
(`../ascii` on this machine: TypeScript, 210 pieces, `<ascii-art>` tag, React
and Astro wrappers, CLI), but:

1. **Pieces written in Rust**, compiled to WebAssembly, drawing into a grid of
   **code points** rather than a string. That removes the UTF-16 problems that
   make astral glyphs (sextants, octants) and double-width glyphs impossible in
   the TS design, and makes the heavy scenes cheap.
2. **Unicode as the medium**: braille (8 dots a cell), sextants and octants,
   geometric shapes, arrows, box drawing and blocks, and, as a later tier,
   double-width CJK and emoji.
3. **An Elm site** to show every piece, with one page per piece.
4. One HTML tag, `<unicode-art piece="…">`, that works with no build step.

Non-goals for now: React/Astro wrappers (add after the tag is solid), third
party pieces loaded by URL (`src=` in ascii.rest), per-piece lazy loading.

## Decisions already made

Do not relitigate these; the owner chose them with the trade-offs in view.

| Decision | Choice | Why |
|---|---|---|
| Piece language | **Rust, `std`** | Measured: a `std` donut is 26.6 KB raw / 9.8 KB gz vs 1.5 KB gz for a `no_std` build. The owner judged 26 KB fine. The whole scaffold module (2 pieces, serde_json, wasm-bindgen) is 77.6 KB raw / 35.8 KB gz. See Appendix B. |
| Wasm glue | `wasm-bindgen` + `wasm-pack --target web` | Standard, typed `.d.ts`, works with `std`. Installed via `cargo:wasm-pack` in mise. |
| Frame buffer | `u32` code point + `u8` colour per cell, read by JS as typed-array views over wasm memory | No string building or decoding per frame; astral glyphs are one cell. |
| Wide glyphs | A `WIDE_TAIL` (0) sentinel in the cell after a double-width glyph | Keeps "every frame is `cols × rows` cells" true; renderers skip tails. |
| Browser renderer | TypeScript shell (`web/`) porting ascii.rest's atlas renderer, not `web-sys` canvas calls from Rust | Each `drawImage` from Rust would cross the JS boundary anyway; the TS is 200 lines and already proven. |
| Site | Elm 0.19.1, Vite + `vite-plugin-elm` | Owner's choice. Elm renders `<unicode-art>` as an ordinary tag; it never touches wasm memory. |
| Tooling | `mise.toml` with `[tasks]` as the single entry point | `mise run check` is what CI runs. |
| One wasm module for all pieces | Yes, for now | Simplest; revisit (per-category bundles) only if the module passes ~500 KB gz. |
| Licence | MIT | `Cargo.toml` says so; `LICENSE` file not yet written (owner's name). Code ported from ascii.rest keeps its MIT attribution comment. |

## What the scaffold contains

Everything below exists and was verified on 2026-10-08 unless noted.

```
mise.toml                     tools + tasks (build:wasm, build:site, dev, test, lint, fmt, size, check)
rust-toolchain.toml           1.97, wasm32 target, rustfmt, clippy
Cargo.toml                    workspace; release profile opt-level=s, lto, panic=abort, strip
package.json                  npm workspaces: web, site
crates/animate-unicode/       the core crate                                  ✅ cargo test --release: 12 pass
  src/grid.rs                 Grid, BLANK, WIDE_TAIL, text()
  src/meta.rs                 Meta, Category, Charset (Basic/Extended/Wide)
  src/options.rs              Options: JSON object with typed getters + defaults
  src/piece.rs                Piece trait, Env { paper }
  src/registry.rs             REGISTRY lookup, make(slug, options_json)
  src/util.rs                 Rng (xorshift64*), ramp(), braille(), smoothstep()
  src/pieces/mod.rs           REGISTRY list
  src/pieces/donut.rs         port of ascii.rest donut.ts                     ✅ byte-identical to TS at t=0, 1.3, 7.7
  src/pieces/braille_wave.rs  first unicode-only piece                        ✅ passes contract
  tests/contract.rs           port of ascii.rest scripts/check.ts
  tests/fixtures/donut.txt    TS donut frames (captured with node from ../ascii)
crates/wasm/                  Player (new/frame/cells_ptr/colors_ptr/text/meta_json), slugs_json, metas_json
                                                                              ✅ wasm-pack build ok; Node smoke test: views == text()
crates/cli/src/main.rs        stub: ANSI loop, `list`, --seconds/--paper/--options    ⚠️ untested
web/src/mount.ts              atlas renderer reading Uint32Array/Uint8Array views; wide cells; rAF/IO/visibility/reduced-motion
web/src/element.ts            <unicode-art> custom element                     ⚠️ tsc passes; NOT yet run in a browser
web/src/wasm.ts               shared init()                                    
web/src/index.ts              exports
site/                         Vite + vite-plugin-elm + Browser.application     ✅ elm make ok; vite build ok (dist/ 36 KB wasm + 16 KB js gz)
site/src/Main.elm             sidebar by category, / and /:slug routes, renders <unicode-art>
site/src/main.ts              awaits wasm(), reads metas_json(), starts Elm with flags
.github/workflows/ci.yml      jdx/mise-action → npm ci → mise run lint/test/build/size
README.md
```

Known loose ends, in order of urgency (they are Phase 0's task list):

1. `site` `tsc -p tsconfig.json` reports two errors: `Cannot find module 'animate-unicode'`
   in `src/main.ts` (the Vite alias resolves it at build time but tsc does not
   know the alias; add `paths` to `site/tsconfig.json` or point at
   `../web/src/*`) and `Cannot find module 'node:url'` in `vite.config.ts`
   (add `@types/node` to site devDependencies). `vite build` itself succeeds.
2. `mise run lint` has never passed end to end (clippy on the donut needed an
   `allow(approx_constant)`, which is in; the rest is unverified as one run).
3. `mise run dev` has never been opened in a browser. The renderer and element
   are ports, not yet seen drawing.
4. `wasm-pack` 0.13.1 is pinned; 0.15.0 exists. Bump when convenient.
5. No `LICENSE` file.
6. `.github/workflows/ci.yml` has never run.

## Architecture

```
                         ┌──────────────────────────────────────────────┐
                         │ crates/animate-unicode  (pure Rust, no DOM)  │
                         │  Piece::frame(t, &Env, &mut Grid)            │
                         │  Grid: Vec<u32> cells, Vec<u8> colors        │
                         │  registry: slug → (&Meta, make)              │
                         └───────┬──────────────────────────┬───────────┘
                                 │                          │
                  ┌──────────────▼───────────┐    ┌─────────▼──────────┐
                  │ crates/wasm (wasm-bindgen)│    │ crates/cli         │
                  │  Player { meta, piece,    │    │  ANSI / crossterm  │
                  │           grid }          │    └────────────────────┘
                  │  frame(t, paper)          │
                  │  cells_ptr() colors_ptr() │  ← JS makes Uint32Array / Uint8Array
                  │  text() meta_json()       │    views over wasm.memory.buffer
                  └──────────────┬────────────┘
                                 │ web/pkg (generated by wasm-pack, gitignored)
                  ┌──────────────▼────────────────────────────────────┐
                  │ web/src  (TypeScript shell, the npm package)      │
                  │  wasm.ts     one shared init()                    │
                  │  mount.ts    <pre> text, or <canvas> glyph atlas  │
                  │  element.ts  <unicode-art piece options fps mono> │
                  └──────────────┬────────────────────────────────────┘
                                 │ Vite alias → ../web/src (no web build needed in dev)
                  ┌──────────────▼────────────────────────────────────┐
                  │ site/  (Elm)                                      │
                  │  main.ts: await wasm(); flags = metas_json()      │
                  │  Main.elm: routes, sidebar, node "unicode-art"    │
                  └───────────────────────────────────────────────────┘
```

### Per-frame data flow (canvas)

1. `requestAnimationFrame` → `mount.ts` `tick` → `t += dt` (capped at 100 ms) → `draw()`.
2. `player.frame(t, paper)` runs the Rust piece into its `Grid`.
3. `new Uint32Array(wasm.memory.buffer, player.cells_ptr(), n)` and the `Uint8Array` for colours. **Re-created every frame**: the buffer detaches when wasm memory grows.
4. Diff against `last`/`lastColor`; for each changed cell: clear its rectangle, look up or rasterise the `(codepoint, colour, wide)` glyph in the atlas canvas, `drawImage` it.
5. Copy `cells`→`last`, `colors`→`lastColor`.

For a `<pre>`: `player.text()` → `el.textContent`. One string allocation per frame, fine at these sizes.

### Memory and lifetime

- A `Player` is a Rust struct owned by JS via wasm-bindgen; **`player.free()` must be called** when the element disconnects or the piece changes (element.ts does this in `#teardown`). Leaking Players leaks their grids and z-buffers.
- One wasm instance per page (`wasm.ts` memoises `init()`), shared by every tag.
- `WIDE_TAIL` is exported from both Rust (`animate_unicode::WIDE_TAIL`) and TS (`mount.ts`); they must stay equal (0). Consider reading `wide_tail()` from wasm once at init instead of duplicating (minor).

## The piece contract

A piece is a module in `crates/animate-unicode/src/pieces/` exporting:

```rust
pub static META: Meta = Meta { name, slug, category, note, cols, rows, fps, charset, options, clock, palette, ground, cell, ..Meta::DEFAULT };
pub fn make(options: &Options) -> Box<dyn Piece>;
impl Piece for X { fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid); }
```

and one line in `pieces/mod.rs` `REGISTRY`. `tests/contract.rs` enforces, for every registered piece:

| Rule | Detail |
|---|---|
| slug | kebab-case, unique |
| name, note | non-empty; note ≤ 72 chars, lowercase, says what you see |
| size | 1–80 × 1–32, or 1–320 × 1–120 for `Category::Scenes` |
| fps | 0–60; 0 means a still |
| palette | `None`, or 2–64 `#rrggbb`; scenes must have palette **and** ground |
| cell | 1 (square) or 2 (character-shaped) |
| options | `None` or a non-empty JSON object of defaults |
| frame shape | `grid.text()` has exactly `rows` lines (the grid guarantees width) |
| charset | every cell's code point is allowed by `meta.charset` (see Unicode); `WIDE_TAIL` only in `Charset::Wide` pieces and only directly after a non-tail cell |
| colours | every colour index `< palette.len()` |
| first frame | not nearly empty (frame 0 is used as the still) |
| motion | fps > 0 ⇒ ≥ 3 distinct frames over 6 s; fps = 0 ⇒ frames identical (unless `clock`) |
| determinism | two fresh instances produce identical frames and colours at the same `t` (unless `clock`) |
| paper | a frame with `Env { paper: true }` is the right shape |
| scenes | ≥ 4 palette colours on inked cells in frame 0 |
| time budget | release only: avg ≤ 4 ms/frame (scenes 10 ms), worst ≤ 30 ms (scenes 40 ms) |

Rules of thumb that are not tested but are the house style (from ascii.rest's
CONTRIBUTING.md, which is worth reading in full):

- Frame 0 should be a good still: the site, the README and `fps=0` show it.
- Use `util::Rng` seeded from a constant or from `t`; never a clock.
- Keep state between frames (z-buffer, particles) but **derive the picture from `t`**, not from "the previous frame", because the server, the contract test and a scrolled-back-into-view element all call `frame` at arbitrary `t`.
- Flip shading ramps for `env.paper` (`util::ramp` does it).
- Draw with `grid.put/set/text_at`; they clip, so shapes can overhang edges.

## Unicode

The reason this project exists. Three tiers, declared per piece in `Meta.charset`
and enforced by the contract test's `allowed()`:

| Charset | Ranges | Font need | Cells |
|---|---|---|---|
| `Basic` | U+0020–007E, `·°•●` (U+00B7, U+00B0, U+2022, U+25CF), box drawing U+2500–257F, blocks U+2580–259F | system monospace; a 3 KB fallback cut covers Android | 1 |
| `Extended` | Basic + arrows U+2190–21FF, geometric shapes U+25A0–25FF, braille U+2800–28FF, Symbols for Legacy Computing U+1FB00–1FBFF (sextants, wedges), octants U+1CD00–1CDE5 and the quarter blocks U+1CEA0–1CEAF they borrow | the bundled font (Phase 5) | 1 |
| `Wide` | any valid scalar; double-width glyphs occupy two cells via `set_wide` | system CJK/emoji fonts; colour emoji ignore `fillStyle` | 1 or 2 |

Not in `allowed()` yet, add when a piece needs them: the rest of U+1CC00–1CEBF (separated quadrants and sextants, sixteenths, and more).

Why code points matter, concretely: `../ascii/src/mount.ts:128` does
`text.charCodeAt(k)` per cell, so a sextant (U+1FB00, two UTF-16 units) would
occupy two cells and desync every row after it. Here a cell is a `u32`, and the
shell calls `String.fromCodePoint`. `grid.rs` has a unit test with `'🬀'` to
pin this.

What "one cell wide" means on each surface:

- **Canvas**: each glyph is clipped into its own cell rectangle in the atlas, so a font whose braille is slightly wide does no harm. This is the robust path and the default for coloured pieces.
- **`<pre>`**: relies entirely on the font. If the system monospace lacks braille and falls back to a proportional face, rows drift. Phase 5 solves this with a subset font declared via `unicode-range`, as ascii.rest does for box drawing (`../ascii/src/ascii.ts` `STYLE`).
- **Terminal**: braille and sextants work in modern terminals (iTerm2, Kitty, WezTerm, Ghostty, Windows Terminal); legacy computing needs a Nerd Font or a recent monospace. The CLI should not assume.

Encodings for braille and sextants are in Appendix C; `util::braille` is implemented and tested.

## Phases

Each phase lists tasks, then acceptance. Work in order; 3 depends on 2, 4 on 3,
6 can start any time after 1 and should be interleaved with everything else
because pieces are the product.

<a id="phase-0"></a>
### Phase 0 · Toolchain green (half a day)

Tasks:
1. `mise install` then `mise run install`. Confirm `mise ls --current` shows every tool installed (elm-test was the one that failed before pinning).
2. Fix the two `site` tsc errors (see loose ends). Add a `typecheck` script to `site/package.json` and include it in the `lint` task.
3. Run `mise run lint`, `mise run test`, `mise run build`, `mise run size` and make each pass.
4. `mise run dev`, open http://localhost:5173, confirm the donut turns in a `<pre>` and braille-wave draws. Check the console for warnings. Resize the window; navigate to `/donut` and back.
5. Write `LICENSE` (MIT, owner's name). Initialise git (owner said they will; coordinate).
6. Push; confirm CI passes on GitHub.

Acceptance: `mise run check` exits 0 locally and in CI; both pieces animate in a browser.

<a id="phase-1"></a>
### Phase 1 · Core crate (1–2 days)

**Done 2026-10-08** on branch `phase-1-core`. Where it landed, where it differs
from the tasks below: the sub-cell canvases are in `src/subcell.rs` (one
generic `Canvas<E>` with `Dots`, `Sextants`, `Quadrants` aliases), not `util`;
`into_grid` is `Canvas::draw(grid, x, y, color)`, which leaves empty cells
alone so canvases layer; `Ramp` is in `src/palette.rs`; reading an option with
no default in `Meta.options` is a contract **error**, not a warning; `Grid::line`
and `blit` take a colour, and `blit` treats spaces as transparent. Lint now
also runs `cargo fmt --check` (style in `rustfmt.toml`) and rustdoc with
`-D warnings`. `mise run show <slug>` and `mise run bench` wrap the examples.

The scaffold has the shape; this phase makes it solid.

Tasks:
1. **Grid**: add `blit(&Grid, x, y)` (copy one grid onto another, for sprites), `row(y) -> &[u32]`, `fill_rect`, `line(x0,y0,x1,y1,ch)` (Bresenham). Keep everything clipping.
2. **Sub-cell canvases** in `util`: a `Dots` type (bitmap 2w × 4h) with `set(x,y)`, `line`, `circle`, and `into_grid(&mut Grid)` that encodes braille; the same for sextants (`Sextants`, 2w × 3h) and quadrants (2w × 2h, using U+2596–259F plus `▌▐█`). braille_wave.rs hand-rolls this; refactor it onto `Dots`.
3. **Palette helpers**: `Meta.palette` is `&'static [&'static str]`. Add a `Ramp` for shaded colour (index into palette by brightness) and a convention: for coloured pieces, palette[0] is the dimmest.
4. **Options**: add `list_f64`, `string` (owned). Decide whether options are validated against `Meta.options` keys (recommend: warn in the contract test if a piece reads a key its defaults do not declare — needs a recording `Options`; optional).
5. **Registry**: keep the hand-written `REGISTRY`. Do not add a build script or proc macro; the list is the one place to see every piece.
6. **Contract test**: it is one `#[test]` that loops; failures report every piece. Keep that, but also add a `cargo test -- --nocapture` friendly summary and a `PIECE=slug` env filter for iterating on one piece. Add a `--show` equivalent: a small `examples/show.rs` that prints frames at t = 0, 1, 2.5, 5 inside a ruled box (port of check.ts `--show`).
7. **Benchmarks**: `benches/` with criterion or a plain `examples/bench.rs` that times 60 frames of every piece in release; print a table. The contract budgets are the hard limit; the bench is for seeing regressions.
8. **Docs**: `cargo doc` clean; every public item has a line.

Acceptance: `cargo test --release` green; `cargo clippy --all-targets -D warnings` green; `examples/show.rs donut` prints four boxed frames; the sub-cell helpers have unit tests pinning specific code points (like `braille_dots_map_to_the_right_bits`).

<a id="phase-2"></a>
### Phase 2 · Wasm boundary (1 day)

**Done 2026-10-08** on branch `phase-2-wasm`. What changed from the tasks below:

- **A panic does not kill the page.** Measured: with `panic = "abort"` the
  panicking call traps (`RuntimeError: unreachable`) and the instance carries
  on; other players keep drawing and new ones can be made. The player whose
  piece panicked is left borrowed, so every later call on it, `free()`
  included, throws. `mount.ts` now stops a tag whose frame throws, and
  `element.ts` frees inside a `try`. Details under "Panics" in
  `crates/wasm/src/lib.rs`. Decision unchanged: keep abort, the contract test
  is the guard.
- **The test piece** is the slug `debug-panic`, which `Player::new` accepts
  only with the `debug` feature (never registered). It draws for a second,
  then indexes past its grid. In the dev site, `debugPanic()` in the console
  adds one.
- **Builds:** `build:wasm` is release; `build:wasm:debug` (what `dev` serves)
  adds the `debug` feature with `console_error_panic_hook`, in its own cargo
  target dir. Both write `web/pkg`, so neither is cached by sources/outputs.
- **Tests:** `web/test/*.test.mjs` by `node --test`, run by `mise run test`
  (now `test:rust` + `test:wasm`) twice: on the release build, then on a debug
  build in `target/wasm-debug-pkg`. They cover every piece's views against
  `text()`, the donut fixture through wasm, errors, `free()` with a leak
  check, and the panic behaviour above.
- **Size gate:** `mise run size` fails past 500 KB gzipped (`WASM_GZ_LIMIT`
  overrides) and writes the line to the GitHub step summary.
- **wasm-pack 0.15.0** from aqua (`drager/wasm-pack`, prebuilt), not cargo.
- `Player::set_options` was not needed: the element re-creates the player.

Tasks:
1. Keep the API as is (`Player`, `slugs_json`, `metas_json`, `wide_tail`). Add `Player::set_options(json)` only if hot-changing options without re-creating the player proves necessary for the site; otherwise re-create.
2. **Panic safety**: with `panic = "abort"`, a piece that indexes out of bounds kills the whole wasm instance for every tag on the page. Add `console_error_panic_hook` behind a `debug` feature so dev builds say where; in release, document that the contract test is the guard. Consider `Player::frame` wrapping in `std::panic::catch_unwind` — **not possible with `panic=abort`**; the trade-off is size (unwinding adds ~10–20 KB (unverified)). Decision: keep abort, rely on the contract test.
3. **wasm-bindgen-test**: `crates/wasm/tests/web.rs` with `wasm_bindgen_test_configure!(run_in_browser)` is optional; the Node smoke test is cheaper. Add `web/test/smoke.test.mjs` (the script from the scaffold session, see Appendix A) run by `node --test`, included in `mise run test`.
4. **Size tracking**: `mise run size` prints raw and gz. Add a CI step that fails if gz > 500 KB (generous; revisit).
5. Bump `wasm-pack` to 0.15 in `mise.toml` and re-verify.

Acceptance: `node --test web/test` green; `mise run size` printed in CI; a deliberately panicking test piece (not registered) shows a readable message in dev.

<a id="phase-3"></a>
### Phase 3 · Web shell (2–3 days)

**Built 2026-10-08** on branch `phase-3-shell`; the manual pass in Chrome,
Safari and on Android is the owner's (record it with the page's copy button).

- **`quadrant-fire`** (Phase 6 #3): doom-fire.ts's simulation ported line for
  line and held to its TS frames by a fixture (`tests/fixtures/doom-fire.txt`),
  drawn in `Quadrants` and the 35-step Doom palette over a `#070707` ground.
  Faint quadrants need two lit neighbours, as doom-fire's faint glyphs do.
- **`debug-wide`** joins `debug-panic` in `crates/wasm/src/debug.rs` (debug
  builds only): 漢 字 中 and a moving 🔥 between narrow glyphs, colours
  swapping each second, a digit ruler underneath.
- **`site/test.html`** (not `site/public/`: it needs Vite to resolve the
  shell) is the checklist page, built beside the Elm site. One tag per case,
  tick boxes, live readouts (braille row widths, wasm memory over 240 swaps,
  whether a server-rendered placeholder survives loading), and a button that
  copies the pass as Markdown. Debug-only cases say so in a release build.
- **Element:** `motion` attribute. A swap to a piece that fails to load now
  stops and clears the old one (ascii.rest keeps showing it); a first load
  that fails still keeps the element's server-rendered children.
- **Font:** the `@font-face` is out of `STYLE` until Phase 5 ships the file.
  Declared without it, every braille page logged a failed font load (the dev
  server answers the missing URL with index.html, a host with a 404). The rule
  is kept in a comment in `element.ts`.
- **npm package:** `npm pack` was leaving out all of `pkg/`, because
  wasm-pack writes a `*` .gitignore there. The wasm tasks now build with
  `--no-pack` and delete it; `web/test/package.test.mjs` dry-runs `npm pack`
  and checks every export and the wasm are in the tarball. `prepublishOnly`
  runs `mise run build:web`. The name is still the placeholder: on 2026-10-08
  `animate-unicode`, `unicode-art` and `unicode-art-element` were all free on npm.
- Checked headless (Lightpanda): every case mounts the right element, swaps
  and re-adds restart cleanly, memory stays flat, the console is clean apart
  from the expected bad-slug warning. Pixels, DPR seams and fonts need the
  real browsers.

`web/src/mount.ts` and `element.ts` are ports; this phase proves them.

Tasks:
1. **Visual check of the atlas renderer** with a coloured piece. Write one (Phase 6 #3, `quadrant-fire` is a good first) so the canvas path is exercised. Check: no seams between cells at fractional DPR (Safari at 1.5×, Chrome at 1.25×); glyphs centred; colour changes redraw the cell; `ground` fills; resize re-rasterises.
2. **Wide glyphs**: a throwaway `Charset::Wide` test piece drawing `漢字` and `🔥` with `set_wide`. Verify on canvas (slot is two cells wide, glyph centred over both) and in `<pre>` (text() drops the tail, so the browser lays the wide glyph over two columns, which only lines up if the font is truly duospaced; document the limitation).
3. **`mono` attribute** on a coloured piece: `<pre>` path, text in one ink.
4. **Attribute changes**: change `piece` at runtime (the Elm site does this when navigating); ensure the old player is freed (`#teardown`), no leaked rAF (`stop` called), no stale IntersectionObserver.
5. **Reduced motion**: with `prefers-reduced-motion: reduce`, first frame only unless `motion` is set. Expose `motion` as an attribute (`element.ts` currently only passes `fps`; add `motion`).
6. **Font loading**: `STYLE` in element.ts declares `@font-face "animate-unicode mono"` at `/fonts/animate-unicode-mono.woff2`, which does not exist yet (Phase 5). Until then the browser silently falls back; fine.
7. **First-frame flash**: a `<unicode-art>` with server-rendered children keeps them until the player is ready (`replaceChildren` happens after `await wasm()`). Verify; this is what the site's stills rely on (Phase 4 #5).
8. **npm package**: `web/package.json` exports `.`, `./element`, `./wasm`. `files` includes `pkg`, so `mise run build:wasm` must run before `npm publish`. Add a `prepublishOnly` that runs both. Decide the public name (`animate-unicode` is the placeholder; check npm availability).
9. **Keep `mount.ts` and `../ascii/src/mount.ts` diffable**: same structure, same names, so fixes upstream can be carried over. The attribution comment stays.

Acceptance: a checklist page `site/public/test.html` (plain HTML, no Elm) with one tag per case above, and a manual pass recorded in the PR; no console warnings; Chrome and Safari; one Android device or emulator for the font fallback.

<a id="phase-4"></a>
### Phase 4 · Elm site (2–3 days)

**Built 2026-10-08** on branch `phase-4-site`.

- **Modules:** `Meta.elm` (all 13 fields, the three sidebar groups), `Route.elm`
  (`/`, `/<slug>`, everything else not found; `canonical` drops trailing and
  doubled slashes), `Options.elm` (defaults in the piece's order, typed edits,
  the JSON of changed values), `Main.elm` (a `port module`: `copy` for the
  clipboard). A second module also ended the vite-plugin-elm empty-dependency
  warning from Phase 0.
- **Layout** after ascii.rest: sidebar in groups art / components / more,
  then categories; the index is a grid of live cards; a piece page has its
  facts, `mono` (pieces with a palette) and `paper` (pieces without a ground,
  or in mono) toggles, the live piece in a well keyed by slug, options, usage
  snippets with copy buttons, and previous / next.
- **Options UI:** number → number input (a piece's options carry no bounds, so
  no slider), bool → checkbox, string → text, anything else shown read-only.
  Only valid values that differ from the default reach the `options`
  attribute; a half-typed number is marked invalid and left out.
- **Decoder failures** show a page with the decoder's message instead of an
  empty site.
- **Stills without script (4.5): (a), client-only.** (b) stays in the
  backlog: a Node prerender of each slug's first frame into a static page,
  with Elm on a child node.
- **Routing:** `/donut/` is replaced by `/donut`; `site/public/_redirects`
  (`/* /index.html 200`) is the host's SPA fallback; files such as test.html
  are served as they are.
- **Tests:** `site/tests` (routes, the decoder on the wasm module's real JSON,
  options), `mise run test:elm`, part of `mise run test`; lint formats-checks
  `site/tests` too. elm-review (optional) not added.
- Checked headless on the production build under `vite preview`: `/`,
  `/donut/`, `/braille-wave`, `/quadrant-fire` and `/nope` render the right
  page with a clean console; option edits restart the piece live.

`site/src/Main.elm` is a `Browser.application` with `/` and `/:slug`. It works but is bare.

Tasks:
1. **Layout** after ascii.rest: sidebar grouped (`art` = scenes; `components` = ui, data, type, logos; `more` = the rest), piece page with name, note, size, fps, charset, the live piece, and copy-able snippets (HTML tag; later React/Astro). Keep one monospace face and `prefers-color-scheme`. `site/src/site.css` has the tokens.
2. **Options UI**: for a piece with `Meta.options`, render inputs from the JSON defaults (number → range or number input, bool → checkbox, string → text) and pass the merged JSON as the `options` attribute. Elm owns the state; the element restarts the piece on attribute change. Use `Html.Keyed` around `<unicode-art>` keyed by slug so navigating between pieces replaces the node rather than mutating it (either works; keyed is cleaner for freeing).
3. **Flags**: `main.ts` passes every `Meta` as JSON. Elm's decoder (`metaDecoder`) currently reads 9 fields; add `options`, `ground`, `cell`, `clock` as they become needed. Decoder failures currently fall back to an empty list silently: surface them (show the error) during development.
4. **mono / paper toggles** on the piece page, like ascii.rest's "one ink" switch.
5. **Stills without script**: Elm's `Browser.application` takes over `<body>`, so server-rendered content is replaced on init. Options: (a) accept a client-only site (simplest; the wasm is 36 KB gz and loads fast); (b) a prerender script in Node that runs the wasm, writes `site/public/still/<slug>.txt`, and a static `index.html` per slug containing the still inside `<unicode-art>` with Elm initialising on a child node rather than `<body>` (`Browser.element` on `#app` with the sidebar outside Elm). Recommend (a) for now; note (b) in the backlog. For OG images (ascii.rest renders PNGs with `sharp`): later.
6. **Routing details**: trailing slash tolerance, 404 page, `<title>` per piece (done), `_redirects` for SPA fallback on the host.
7. **elm-test**: at least `route` and `metaDecoder` tests in `site/tests/`. `mise run test` should include `elm-test` (add to the task).
8. **elm-review** (optional): add `npm:elm-review` to mise and a default config.

Acceptance: `mise run build:site` output served statically works for `/`, `/donut`, `/braille-wave`, `/nope`; options change the piece live; elm-test green; elm-format --validate green.

<a id="phase-5"></a>
### Phase 5 · Fonts (1 day, plus testing on devices)

**Built 2026-10-08** on branch `phase-5-fonts`; the device pass (macOS,
Windows, Android) is the owner's, on test.html's new "every range" section.

- **Font: Iosevka Fixed Extended 34.9.0** (OFL 1.1, no reserved name).
  Coverage measured with fonttools: every Basic and Extended range, octants
  too, every glyph 600/1000 em. Rejected: JetBrains Mono (no braille or legacy
  computing), Cascadia Mono 2407.24 (arrows 10 of 112, and the static TTF's
  name table carries a "Microsoft supplied font" licence string, not the OFL),
  Iosevka Fixed regular width (0.5 em: pieces would draw 20% too tall).
- **The font leads the stack, ASCII included** (a change from the
  `unicode-range`-fallback design above). As a fallback after the system
  monospace, alignment would depend on each platform's ASCII width (Consolas
  is 0.55 em, the font 0.6), so braille rows would drift on Windows. First in
  the stack, a piece's every glyph has one width everywhere; wide glyphs fall
  through to the system. Canvas uses the same stack, asks for the font, and
  rebuilds its glyph atlas when fonts finish loading.
- **Build:** `scripts/font.py`, run by `mise run build:font` through `uvx` with
  fonttools 4.60.1 pinned (uv is in mise): downloads the unhinted release once
  into target/font, checks its SHA-256, subsets, renames to "animate-unicode
  mono", checks widths, coverage and that element.ts declares the same
  unicode-range. Output checked in: `web/fonts/animate-unicode-mono.woff2`,
  17.9 KB (target was < 20 KB), with `web/fonts/OFL.txt`.
- **Location:** next to the shell, found with
  `new URL("../fonts/…", import.meta.url)`, so it ships in the npm package
  (`files` and the tarball test include it) and bundlers copy it; Vite emits
  it as a hashed asset. The site needs no `site.css` declaration: the
  element's style is document-wide and the site has no `<pre>` stills.
- **Caching:** `site/public/_headers` gives `/assets/*` a year, immutable
  (hashed names). Vite preview ignores the file; it takes effect on the host.
- **test.html:** every range as 32-glyph rows, in the font and in system
  fonts only, each with a row-width readout and whether the font loaded.

Tasks:
1. Pick an OFL monospace with the `Extended` ranges. Candidates **(coverage unverified, check with `pyftsubset --unicodes=… --verbose` or `fc-query`)**: Cascadia Mono (added Symbols for Legacy Computing in the 2404 release), Iosevka (has U+1FB00 block), JetBrains Mono (braille yes, legacy computing no), GNU Unifont (everything, bitmap look). Recommend Cascadia Mono or Iosevka Fixed.
2. Subset with fonttools: `pyftsubset Font.ttf --unicodes="U+00B0,U+00B7,U+2022,U+2190-21FF,U+2500-25FF,U+2800-28FF,U+1FB00-1FBFF" --flavor=woff2 --output-file=site/public/fonts/animate-unicode-mono.woff2`. Target < 20 KB. Add `python` + `pip:fonttools[woff]` to mise if the subsetting is to be reproducible (or vendor the output and a script).
3. Put the `@font-face` back at the front of `STYLE` in `element.ts`: the rule is in the comment above it (taken out in Phase 3 so the missing file logged nothing). The `unicode-range` must match the subset. On the site, also declare it in `site.css` so `<pre>` stills (if any) use it.
4. Verify in `<pre>`: a braille piece keeps every row the same pixel width (measure with `getBoundingClientRect` on each row in a test page); same for sextants. Verify the canvas path is unaffected (it clips per cell).
5. Android: system monospace there has no box drawing at all; this font must carry U+2500–259F too (it does in the ranges above).

Acceptance: a test page with every `Extended` range drawn in a `<pre>` shows aligned rows on macOS, Windows, Android; the font is served with long cache headers and `font-display: swap`.

<a id="phase-6"></a>
### Phase 6 · Pieces (ongoing; the product)

Start with pieces that show what unicode buys. Each one: a file, a `REGISTRY` line, `mise run test`, a look in the browser. Suggested first ten, in order:

| # | slug | charset | what | notes |
|---|---|---|---|---|
| 1 | `braille-wave` | Extended | done | on `Dots` since Phase 1 |
| 2 | `braille-lissajous` | Extended | done: a 3:2 figure whose phase drifts; head a solid `Dots::line`, tail thinning by a fixed per-sample hash | options `a`, `b`, `speed`; pure in `t` |
| 3 | `quadrant-fire` | Basic, palette | done in Phase 3: doom-fire's spread in quadrants, Doom palette | simulation fixture-tested against doom-fire.ts |
| 4 | `sextant-plasma` | Extended, palette | done: the plasma drawn as contour lines at sextant resolution, one colour per band | a filled plasma needs two colours a cell; contours need one |
| 5 | `braille-donut` | Extended | done: the torus at 80×88 dots, z-buffered per dot, shaded by 4×4 Bayer dithering, flipped on paper | 0.47 ms a frame (sine tables) |
| 6 | `box-frames` | Basic | done: port of ascii.rest's, fixture-tested (`tests/fixtures/box-frames.txt`) | still |
| 7 | `geometric-tiles` | Extended | done: ◤◥◢◣ rosettes turning in rings, ◆ on crests, ● at the centre | Iosevka's geometric shapes are 0.49 em icons, not cell-filling, so tiles stand apart |
| 8 | `arrow-field` | Extended | done: three wandering vortices in a current, arrows snapped to eight ways, `·` where slack | |
| 9 | `octant-sphere` | Extended, palette | done: a lit beach ball; a cell shows its majority gore, so seams are a dot wide | octant table from UnicodeData (below) |
| 10 | `kanji-rain` | Wide, palette | done: 24 streams of full-width katakana and kanji | the first `Wide` piece |

**Phase 6, 2026-10-08,** on branch `phase-6-pieces`, with what the pieces needed:

- **Octants:** `subcell::Octant` / `Octants`. `scripts/octants.py` builds the
  256-entry table from Unicode 16's UnicodeData.txt by name: 230 BLOCK
  OCTANT characters, and 26 masks that reuse older glyphs (space, halves,
  quadrants, quarter rows, the four single corners U+1CEA0–1CEAF, the middle
  quarters U+1FBE6–7, full). `allowed()` and `Charset::Extended` now include
  U+1CD00–1CDE5 and U+1CEA0–1CEAF; the font gained U+1CEA0–1CEAF (18.0 KB).
- **Wide glyphs:** `unicode-width` (in debug assertions and the contract only).
  `Grid::set_wide` debug-asserts width 2; the contract fails a double-width
  glyph drawn without its tail, and a tail behind a narrow glyph (checked by
  breaking kanji-rain on purpose).
- Size after 11 pieces: see Appendix B.

Porting from TS (`../ascii/src/pieces/*.ts`, 210 files): mechanical. Pattern:
`const out = new Array(cols*rows).fill(" ")` → `grid.clear()`; `out[k] = ch` →
`grid.cells_mut()[k] = ch as u32`; `env.color[i] = n` → `grid.colors_mut()[i] = n`;
`Math.random` never appears (they use seeded PRNGs; port the same PRNG to keep
frames identical, then fixture-test like the donut). Keep the TS file's header
comment and add "port of ascii.rest's X (MIT, @bas3line)".

Fixture tests: for any port, capture frames with the one-liner in Appendix A
and assert equality as `donut_matches_the_typescript_original` does. Do it for
every port; it makes the port provably right and catches float-order drift.

Acceptance per piece: contract green, looks right in the browser at 1× and 2× DPR, in dark and light, and (if Basic) in the CLI.

<a id="phase-7"></a>
### Phase 7 · CLI (1 day)

`crates/cli/src/main.rs` is a stub (raw ANSI, sleeps, no input).

Tasks:
1. Add `crossterm`: alternate screen, hidden cursor, raw mode, stop on any key or Ctrl-C, restore terminal on every exit path (including panic: use a guard struct with `Drop`).
2. Colour: truecolor from `Meta.palette` (`\x1b[38;2;r;g;bm`), `--mono` for the terminal's own colour, `--light` (= `Env.paper` + a lighter palette half, if pieces provide one; ascii.rest splits palettes into dark/light halves — decide whether to adopt that).
3. Centre the piece; crop to the terminal when smaller; redraw on resize.
4. Wide glyphs: emit the text as `Grid::text()` does (tails omitted) and trust the terminal's wcwidth. Note in `--help` that `Wide` pieces need a CJK-capable terminal font.
5. `list` output grouped by category, like ascii.rest's.
6. Frame pacing: sleep until the next frame boundary rather than a fixed sleep (drift otherwise).

Acceptance: `cargo run -p animate-unicode-cli --release -- donut` plays and exits cleanly on `q`; `--seconds 2` exits on time; terminal is restored after Ctrl-C; `list` prints every piece.

<a id="phase-8"></a>
### Phase 8 · Packaging, deploy, CI (1 day)

1. **Site hosting**: Cloudflare Workers static assets as ascii.rest does (`../ascii/site/wrangler.jsonc`, `_headers`, `_redirects`). Needs: SPA fallback to `index.html`, `Content-Type: application/wasm` (Workers sets it), CORS open on `/element.js` and the wasm if the tag is to be used cross-origin from other sites, long cache on hashed assets.
2. **Serving the library from the site root** (`<script type="module" src="https://<site>/element.js">`): a Vite `lib` build or a second esbuild step that emits `element.js` + the wasm at stable paths (not hashed). ascii.rest does this by copying `dist/` into the site.
3. **npm publish** of `web/` (name TBD) with `pkg/` included; a `release.yml` on tags.
4. **CI**: `ci.yml` exists; add the size gate (Phase 2), Node smoke test, elm-test. Cache: `jdx/mise-action` caches tools; `Swatinem/rust-cache` the target dir; npm cache via `actions/setup-node` is unnecessary (mise provides node) but `npm ci` needs a lockfile committed.
5. **README**: install, HTML usage, pieces table (generate it from `metas_json()` with a script, as `../ascii/scripts/readme.ts` does), contributing (a short CONTRIBUTING.md adapted from ascii.rest's).

Acceptance: a tagged release publishes to npm and deploys the site; a plain HTML file on another origin shows a piece with two lines.

## Testing strategy

| Layer | Tool | What |
|---|---|---|
| Core | `cargo test --release` | unit tests per module; `tests/contract.rs` for every piece; fixture tests for ports |
| Wasm | `node --test 'web/test/*.test.mjs'`, release and debug builds | Player API, memory views equal `text()`, errors throw, `free()`, panics |
| Shell | manual checklist page + (later) Playwright screenshots in CI **(optional)** | atlas rendering, resize, DPR, reduced motion, teardown |
| Site | `elm-test`, `elm-format --validate`, `tsc` | routing, decoders |
| Size | `mise run size` + CI gate | wasm gz |
| Perf | contract budgets (hard) + `examples/bench.rs` (table) | per-frame ms |

Debug builds skip the timing budgets (`cfg!(debug_assertions)`); always run `mise run test`, which is `--release`.

## Performance budgets

- Per frame (release, native): ≤ 4 ms avg / 30 ms worst; scenes ≤ 10 / 40. Wasm is typically 1.2–2× native **(unverified for this code)**; if a piece sits near the native budget, check it in the browser's profiler.
- Draw: the atlas renderer redraws only changed cells; a full-screen scene at 320×120 = 38 400 cells is the worst case, which ascii.rest handles on phones. Unicode changes nothing here.
- Memory: a `Grid` is 5 bytes/cell; a 320×120 scene is 192 KB plus the piece's own buffers. Fine.
- Wasm size: 35.8 KB gz now; expect roughly +1–3 KB gz per ported piece (Appendix B). Gate at 500 KB.
- Load: `init()` once per page; `metas_json()` for the site is one string; fine.

## Risks and open questions

1. **A panicking piece stops its own tag** (measured in Phase 2: the instance survives, the player is left unusable and leaks what it held). Mitigated by the contract test and by pieces using clipping `put/set` instead of raw indexing. Keep raw `cells_mut()[k]` for hot loops only, with the bounds proven.
2. **Font coverage for `Extended` in `<pre>`** is the biggest visual risk; the canvas path does not have it. Phase 5 must be tested on real Android and Windows.
3. **Wide glyphs in `<pre>`** depend on the font being exactly 2:1 for CJK; many "monospace" fonts are not. Treat `Wide` as canvas-first.
4. **Elm and SSR** do not mix (Phase 4 #5). Decided in Phase 4: client-only for now; a prerender with Elm mounted on a child node is in the backlog.
5. **One module for all pieces**: per-piece lazy loading is lost vs ascii.rest. Revisit if the module grows large; wasm-bindgen does not split modules, so this would mean one crate per bundle.
6. **mise `npm:` backend** resolved `elm-test@latest` to 0.19.0 and then refused its `fsevents`. Pinned to `0.19.1-revision17`. Watch for the same with `elm-format` (pinned 0.8.8) and `elm-review`.
7. **`rust = { version = "1.97", targets = "wasm32-unknown-unknown" }`**: the `targets` option worked on this machine (`cargo check --target wasm32-unknown-unknown` passed) but the target had also been added by hand earlier; confirm on a clean CI runner.
8. **Name**: `animate-unicode` is a placeholder for the crate, npm package and tag (`<unicode-art>`). Decide before publishing anything.
9. **Timing budgets in CI** can flake on slow runners; if so, raise the worst-frame budget in CI via an env var rather than loosening locally.

<a id="appendix-a"></a>
## Appendix A: what to borrow from ../ascii

`/Users/curtisault/projects/ascii` (MIT, @bas3line). Keep attribution comments on anything ported.

| Need | Source | Notes |
|---|---|---|
| Renderer | `src/mount.ts` | Already ported to `web/src/mount.ts`; diff them when fixing bugs |
| Custom element | `src/ascii.ts` | Ported to `web/src/element.ts` |
| Contract checker | `scripts/check.ts` | Ported to `tests/contract.rs`; the `--show` printer is not yet |
| Piece style guide | `CONTRIBUTING.md` | Read it; the "what makes a piece good" guidance applies verbatim |
| Pieces to port | `src/pieces/*.ts` (210) | `donut` done; `doom-fire`, `box-frames`, `dividers`, `cube`, `black-hole` are small and self-contained; scenes (`tokyo-rain`, 1 122 lines) last |
| Site layout and CSS | `site/src/layouts/Layout.astro`, `site/src/styles/site.css`, `site/src/lib/library.ts` (GROUPS) | Sidebar grouping and visual tone |
| README generator | `scripts/readme.ts` | Pieces table from meta |
| Terminal player | `src/terminal.ts`, `src/cli.ts` | Behaviour to match in Phase 7 (alt screen, centre, crop, resize, restore) |
| Fonts | `site/public/fonts/ascii-rest-mono.woff2` + `STYLE` in `src/ascii.ts` | The `unicode-range` trick for a tiny fallback cut |
| Hosting | `site/wrangler.jsonc`, `site/public/_headers`, `_redirects` | Cloudflare static assets, CORS for the library files |

Capturing TS frames for a fixture (run in `../ascii`):

```sh
node --input-type=module -e '
import { writeFileSync } from "node:fs";
const m = await import("./src/pieces/donut.ts"); const f = m.default();
const out = [0, 1.3, 7.7].map(t => `t=${t}\n${f(t)}`).join("\n\n");
writeFileSync("../animate-unicode/crates/animate-unicode/tests/fixtures/donut.txt", out + "\n");'
```

For a coloured piece, also capture `env.color` (pass `{ color: new Uint8Array(cols*rows) }` and dump it) and compare `grid.colors()`.

Node smoke test of the wasm boundary (what was run during scaffolding; put it under `web/test/`):

```js
import { readFileSync } from "node:fs";
const pkg = new URL("../pkg/", import.meta.url);
const mod = await import(new URL("animate_unicode.js", pkg));
const wasm = await mod.default({ module_or_path: readFileSync(new URL("animate_unicode_bg.wasm", pkg)) });
const p = new mod.Player("donut", "");
p.frame(1.3, false);
const n = p.cols() * p.rows();
const cells = new Uint32Array(wasm.memory.buffer, p.cells_ptr(), n);
const rows = Array.from({ length: p.rows() }, (_, r) => String.fromCodePoint(...cells.subarray(r * p.cols(), (r + 1) * p.cols())));
if (rows.join("\n") !== p.text()) throw new Error("views and text() disagree");
p.free();
```

<a id="appendix-b"></a>
## Appendix B: size measurements

Measured 2026-10-08 on this machine. TS minified with esbuild 0.25; Rust 1.97, `opt-level="z"`, lto, `panic=abort`, strip, for the donut-only experiments; the scaffold uses `opt-level="s"` plus wasm-opt `-Os`.

| Build | Raw | gzip |
|---|---|---|
| TS `donut.ts` minified | 1 040 B | 662 B |
| TS all 210 pieces, each minified separately | 827 KB | 338 KB (sum) |
| TS all 210 pieces, concatenated | — | 277 KB |
| TS median piece | 3.2 KB | 1.3 KB |
| TS largest (`tokyo-rain`) | 23.5 KB | 10.7 KB |
| TS shell (`mount.ts` + `ascii.ts`, bundled) | — | 2.4 KB |
| Rust donut, `std`, raw exports, no bindgen | 26.6 KB | 9.8 KB |
| Rust donut, `std`, sin/cos imported from JS | 18.6 KB | 6.7 KB |
| Rust donut, `no_std` + libm | 7.3 KB | 3.9 KB |
| Rust donut, `no_std`, sin/cos from JS | 2.1 KB | 1.5 KB |
| **This scaffold**: 2 pieces + serde_json + wasm-bindgen, wasm-pack release | **77.6 KB** | **35.8 KB** |
| Scaffold's wasm-bindgen JS glue (`animate_unicode.js`) | 12.9 KB | — |
| Site bundle (Elm + shell + glue) | 43.8 KB | 16.1 KB |
| After Phase 3: 3 pieces (quadrant-fire added) | 90.5 KB | 42.3 KB |
| **After Phase 6: 11 pieces** (2026-10-08) | **139.6 KB** | **64.6 KB** |

Reading: the fixed cost of `std` + serde_json + bindgen is ~33 KB gz; each additional piece should add roughly what its TS does (1–3 KB gz) since the runtime is already paid for. Re-measured after eleven pieces: the eight Phase 6 pieces added 22.3 KB gz, about 2.8 KB each, the top of the projected range (octant-sphere carries its 256-glyph table). At that rate the 500 KB gate is some 150 pieces away.

<a id="appendix-c"></a>
## Appendix C: glyph encodings

**Braille** (U+2800–28FF): 2 columns × 4 rows of dots per cell. Dot numbering is columnar: left column dots 1,2,3,7 and right column 4,5,6,8, top to bottom. Bit values: 1→0x01, 2→0x02, 3→0x04, 4→0x08, 5→0x10, 6→0x20, 7→0x40, 8→0x80. `util::braille([row0, row1, row2, row3])` takes per-row masks (bit 0 = left dot, bit 1 = right) and is unit-tested. An all-empty cell should be drawn as a space, not U+2800, so `text()` and the "nearly empty" check behave.

**Quadrants** (2 × 2): `▘` U+2598 (TL), `▝` U+259D (TR), `▖` U+2596 (BL), `▗` U+2597 (BR), `▀` U+2580 (top), `▄` U+2584 (bottom), `▌` U+258C (left), `▐` U+2590 (right), `▚` U+259A (TL+BR), `▞` U+259E (TR+BL), `▛` U+259B, `▜` U+259C, `▙` U+2599, `▟` U+259F, `█` U+2588, space. Build a 16-entry lookup indexed by bits (TL=1, TR=2, BL=4, BR=8).

**Sextants** (2 × 3, U+1FB00–1FB3B, Symbols for Legacy Computing): bits TL=1, TR=2, ML=4, MR=8, BL=16, BR=32, giving n in 0..63. The block omits four values that exist elsewhere: n=0 (space), n=21 (`▌` U+258C), n=42 (`▐` U+2590), n=63 (`█` U+2588). For the rest: `code = 0x1FB00 + n - 1 - (n > 21) - (n > 42)`. Sanity: n=1 → U+1FB00 BLOCK SEXTANT-1; n=20 → U+1FB13 BLOCK SEXTANT-35; n=22 → U+1FB14 BLOCK SEXTANT-235. Unit-test those three.

**Octants** (2 × 4, U+1CD00–1CDE5, Unicode 16): 256 combos minus those that exist as blocks/quadrants/sextants/half-blocks; the exclusion list is long, so generate the lookup table from `UnicodeData.txt` names ("BLOCK OCTANT-…") rather than by formula, and commit the 256-entry table. Font support is thin as of 2025 **(unverified for 2026)**; verify before building on it.

**Wide glyphs**: East Asian Width `W` and `F`, plus most emoji presentation sequences. Rust: the `unicode-width` crate gives `UnicodeWidthChar::width()`; add it (it is small) when the first `Wide` piece lands, and have `Grid::set_wide` debug-assert width 2. The contract test should reject a `set` (narrow) of a width-2 char in any charset.
