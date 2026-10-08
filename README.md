# animate-unicode

Animated unicode art for web pages. The pieces are Rust, compiled to
WebAssembly; a small TypeScript shell draws them in a `<pre>` or on a
`<canvas>`; the site that shows them all is Elm.

```html
<script type="module" src="/element.js"></script>
<unicode-art piece="braille-wave"></unicode-art>
```

A successor to the idea in [ascii.rest](https://ascii.rest) (TypeScript, ASCII
and box-drawing glyphs), with cells that are code points rather than UTF-16
units, so braille, sextants, geometric shapes and double-width glyphs are each
one cell like any other.

**Status: scaffold.** Two pieces, the contract test, the wasm boundary, the
`<unicode-art>` tag and an Elm site that lists them. The plan for building it
out is in [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md).

## Layout

```
crates/animate-unicode   the pieces, the Grid, Meta, the Piece trait, the registry (pure Rust, no DOM)
crates/wasm              wasm-bindgen Player: a piece whose grid JS reads out of wasm memory
crates/cli               plays a piece in a terminal
web/                     the TS shell: mount (atlas renderer), <unicode-art>, the npm package
site/                    the Elm site (Vite + vite-plugin-elm)
docs/                    the implementation plan
```

## Getting started

```sh
mise install          # rust (+wasm32 target), node, elm, wasm-pack, elm-format, elm-test, wasm-tools, watchexec
mise run install      # npm install for web/ and site/
mise run test         # cargo test --release: unit tests and the piece contract
mise run dev          # builds the wasm, then the site at http://localhost:5173
mise run check        # what CI runs: lint, test, build
```

`mise run` lists every task. The CLI: `cargo run -p animate-unicode-cli --release -- donut --seconds 5`.

## Adding a piece

A file in `crates/animate-unicode/src/pieces/`, a `pub mod` and one line in
`REGISTRY` in `pieces/mod.rs`. `mise run test` holds it to the contract: a
`cols × rows` grid every frame, deterministic in `t`, only the glyphs its
`charset` allows, under its time budget. `donut.rs` is the reference for a
shaded piece; `braille_wave.rs` for one that draws with dots.
