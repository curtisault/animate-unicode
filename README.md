# unicel

**Uni**code and **cel**: every glyph is one cell, and a cel is a frame of drawn
animation.

Animated unicode art for web pages. The pieces are Rust, compiled to
WebAssembly; a small TypeScript shell draws them in a `<pre>` or on a
`<canvas>`; the site that shows them all is Elm.

```html
<script type="module" src="/element.js"></script>
<unicel-art piece="braille-wave"></unicel-art>
```

A successor to the idea in [ascii.rest](https://ascii.rest) (TypeScript, ASCII
and box-drawing glyphs), with cells that are code points rather than UTF-16
units, so braille, sextants, geometric shapes and double-width glyphs are each
one cell like any other.

**Status: early.** Two hundred and sixteen pieces, the contract test, the wasm boundary, the
`<unicel-art>` tag and an Elm site that lists them. The plan for building it
out is in [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md).

## Layout

```
crates/unicel   the pieces, the Grid, Meta, the Piece trait, the registry (pure Rust, no DOM)
crates/wasm              wasm-bindgen Player: a piece whose grid JS reads out of wasm memory
crates/cli               plays a piece in a terminal
web/                     the TS shell: mount (atlas renderer), <unicel-art>, the npm package
site/                    the Elm site (Vite + vite-plugin-elm)
docs/                    the implementation plan
```

## Getting started

```sh
mise install          # rust (+wasm32 target), node, elm, wasm-pack, elm-format, elm-test, wasm-tools, watchexec
mise run install      # npm install for web/ and site/
mise run test         # Rust tests and the piece contract, then the wasm from Node
mise run dev          # a debug wasm build, then the site at http://localhost:5173
mise run check        # lint, test, build
mise run signoff      # CI, locally: checks each open PR's head and signs it off on GitHub
```

With `mise run dev` up, http://localhost:5173/test.html is the shell's checklist
page: every rendering case on one page, to tick through in each browser.

`mise run` lists every task.

## Pull requests and CI

There is no CI on GitHub: the contract's per-frame time budgets assume a fast
machine, and GitHub's runners are several times slower. CI runs locally
instead, and `main` will not take a pull request without it:

- `mise run signoff` copies each open pull request's head commit into
  `target/ci/tree` and runs `npm ci`, `mise run lint`, `test`, `build` and
  `size` on it, then posts a `signoff` status (pass or fail) to that commit.
  `mise run signoff 18` or `mise run signoff <branch>` checks just one.
- `main` requires the `signoff` status, so after every push to a pull request,
  sign it off again before merging. Logs are in `target/ci/logs/`.

In a terminal: `mise run play braille-donut` plays a piece until you press a
key (`--seconds`, `--mono`, `--light`, `--fps`, `--options`; `list` names them
all). Braille, sextants and octants need a recent monospace font, kanji-rain
one with CJK.

## Adding a piece

A file in its category's folder, `crates/unicel/src/pieces/<category>/`,
a `pub mod` and one line in `REGISTRY` in that folder's `mod.rs`. `mise run test`
holds it to the contract: a `cols × rows` grid every frame, deterministic in
`t`, only the glyphs its `charset` allows, under its time budget.
`shapes/donut.rs` is the reference for a shaded piece; `generative/braille_wave.rs`
for one that draws with dots.

Each category is built into its own wasm module, so a page loads only the
categories it shows. A piece may use the crate's shared code and its own
folder's pieces, never another category's: its category's module would not
have them.

While working on one, `mise run show <slug>` prints its frames at a few times
in a box, `PIECE=<slug> cargo test --release --test contract -- --nocapture`
checks only that piece, and `mise run bench` times every piece. `Dots`,
`Sextants` and `Quadrants` draw at sub-cell resolution; `Ramp` shades with
palette colours.
