//! The browser boundary. A `Player` owns one piece and its grid; JS asks for a
//! frame, then reads the grid out of wasm memory as typed-array views over
//! `cells_ptr()` and `colors_ptr()`. Nothing is copied or stringified per
//! frame; `text()` exists for the server's first frame and for a `<pre>`.
//!
//! Built by `mise run build:wasm` into web/pkg with wasm-pack (`--target web`).
//! The TS shell in web/src/mount.ts is the only consumer of this API.
//!
//! # Panics
//!
//! The release profile has `panic = "abort"`. In wasm that means a panicking
//! call traps: it throws `RuntimeError: unreachable` to JS and stops there.
//! The instance itself survives, and other players keep drawing. The player
//! whose piece panicked does not: wasm-bindgen marked it borrowed for the call,
//! the trap skipped the release, and every later call on it throws "recursive
//! use of an object". So one broken piece stops its own tag, not the page.
//! Whatever the piece was halfway through (a buffer, an allocation) stays as
//! it was; a page that keeps hitting panics leaks a little each time.
//!
//! Catching panics per piece would need unwinding, which costs size and is
//! impossible with `abort`. The guard is the contract test
//! (crates/animate-unicode/tests/contract.rs), which plays every registered
//! piece over six seconds, with `paper` on and off, before anything ships.
//! Pieces draw with the grid's clipping methods and keep raw indexing for
//! proven hot loops.
//!
//! With the `debug` feature (what `mise run dev` builds) a panic first prints
//! its message and source location with `console.error`, and `Player::new`
//! accepts the unregistered slug `debug-panic` (in [`debug`]), a piece that
//! panics from one second in, to show all of this in a page.
//!
//! The shell (web/src/mount.ts) stops a tag whose piece throws, rather than
//! calling it again every frame, and the element frees a stuck player inside a
//! `try`: freeing it throws too.

use animate_unicode::{registry, Env, Grid, Meta, Piece};
use wasm_bindgen::prelude::*;

/// Runs once when the module is instantiated: in a debug build, routes panics
/// to `console.error` with their location.
#[cfg(feature = "debug")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[cfg(feature = "debug")]
pub mod debug;

/// One piece and the grid it draws into. Call `free()` when done with it.
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
        #[cfg(feature = "debug")]
        if let Some(entry) = debug::find(slug) {
            return Ok(Player { meta: entry.meta, piece: (entry.make)(&Default::default()), grid: Grid::new(entry.meta.cols, entry.meta.rows) });
        }
        let (meta, piece) = registry::make(slug, options_json).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Player { meta, piece, grid: Grid::new(meta.cols, meta.rows) })
    }

    /// Frame width in cells.
    pub fn cols(&self) -> usize {
        self.meta.cols
    }

    /// Frame height in cells.
    pub fn rows(&self) -> usize {
        self.meta.rows
    }

    /// Frames a second; 0 for a still.
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
