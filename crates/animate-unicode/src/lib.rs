//! animate-unicode: animated unicode art, as pure functions of time.
//!
//! A piece is a value that, asked for the picture at `t` seconds, writes one
//! code point and one palette index per cell into a [`Grid`]. Nothing here
//! knows about the DOM, a canvas or a terminal: the wasm crate exposes the grid
//! to a small TypeScript shell that draws it, and the CLI prints it.
//!
//! ```
//! use animate_unicode::{registry, Env, Grid, Options};
//!
//! let entry = registry::find("donut").unwrap();
//! let mut piece = (entry.make)(&Options::default());
//! let mut grid = Grid::new(entry.meta.cols, entry.meta.rows);
//! piece.frame(1.5, &Env::default(), &mut grid);
//! assert_eq!(grid.text().lines().count(), entry.meta.rows);
//! ```

#![warn(missing_docs)]

pub mod grid;
pub mod meta;
pub mod options;
pub mod palette;
pub mod piece;
pub mod pieces;
pub mod registry;
pub mod subcell;
pub mod util;

pub use grid::{Grid, BLANK, WIDE_TAIL};
pub use meta::{Category, Charset, Meta};
pub use options::Options;
pub use palette::Ramp;
pub use piece::{Env, Piece};
pub use subcell::{Dots, Octants, Quadrants, Sextants};
