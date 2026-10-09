//! Screen effects: fire, rain, glitches: the `effects` category, and the `effects` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod kanji_rain;
pub mod quadrant_fire;

/// The effects pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[Entry { meta: &quadrant_fire::META, make: quadrant_fire::make }, Entry { meta: &kanji_rain::META, make: kanji_rain::make }];
