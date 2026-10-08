//! The browser boundary. A `Player` owns one piece and its grid; JS asks for a
//! frame, then reads the grid out of wasm memory as typed-array views over
//! `cells_ptr()` and `colors_ptr()`. Nothing is copied or stringified per
//! frame; `text()` exists for the server's first frame and for a `<pre>`.
//!
//! Built by `mise run build:wasm` into web/pkg with wasm-pack (`--target web`).
//! The TS shell in web/src/mount.ts is the only consumer of this API.

use animate_unicode::{registry, Env, Grid, Meta, Piece};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Player {
    meta: &'static Meta,
    piece: Box<dyn Piece>,
    grid: Grid,
}

#[wasm_bindgen]
impl Player {
    /// A piece by slug with JSON option overrides (`""` for none). Throws for an unknown slug or bad JSON.
    #[wasm_bindgen(constructor)]
    pub fn new(slug: &str, options_json: &str) -> Result<Player, JsError> {
        let (meta, piece) = registry::make(slug, options_json).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Player { meta, piece, grid: Grid::new(meta.cols, meta.rows) })
    }

    pub fn cols(&self) -> usize {
        self.meta.cols
    }

    pub fn rows(&self) -> usize {
        self.meta.rows
    }

    pub fn fps(&self) -> u32 {
        self.meta.fps
    }

    /// The piece's `Meta` as JSON: palette, ground, cell, charset and the rest.
    pub fn meta_json(&self) -> String {
        self.meta.to_json()
    }

    /// Draws the frame at `t` seconds into the grid.
    pub fn frame(&mut self, t: f64, paper: bool) {
        self.piece.frame(t, &Env { paper }, &mut self.grid);
    }

    /// Start of the `cols × rows` code points in wasm memory. Re-read after
    /// every `frame`: the buffer may move if memory grows.
    pub fn cells_ptr(&self) -> *const u32 {
        self.grid.cells().as_ptr()
    }

    /// Start of the `cols × rows` palette indices in wasm memory.
    pub fn colors_ptr(&self) -> *const u8 {
        self.grid.colors().as_ptr()
    }

    /// The frame as text, for a `<pre>` or a server-rendered first frame.
    pub fn text(&self) -> String {
        self.grid.text()
    }
}

/// Every piece's slug, sorted, as a JSON array.
#[wasm_bindgen]
pub fn slugs_json() -> String {
    let s = registry::slugs();
    format!("[{}]", s.iter().map(|x| format!("\"{x}\"")).collect::<Vec<_>>().join(","))
}

/// Every piece's `Meta`, in registry order, as a JSON array. What the site's Elm app gets as flags.
#[wasm_bindgen]
pub fn metas_json() -> String {
    format!("[{}]", registry::all().iter().map(|e| e.meta.to_json()).collect::<Vec<_>>().join(","))
}

/// The right half of a double-width glyph in `cells`; the shell skips these cells.
#[wasm_bindgen]
pub fn wide_tail() -> u32 {
    animate_unicode::WIDE_TAIL
}
