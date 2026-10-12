//! Screen effects: fire, rain, glitches: the `effects` category, and the `effects` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod fireworks;
pub mod kanji_rain;
pub mod quadrant_fire;
pub mod rotozoomer;
pub mod sparks;
pub mod synthwave;
pub mod tunnel;
pub mod tv_static;

/// The effects pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &quadrant_fire::META, make: quadrant_fire::make },
    Entry { meta: &kanji_rain::META, make: kanji_rain::make },
    Entry { meta: &fireworks::META, make: fireworks::make },
    Entry { meta: &sparks::META, make: sparks::make },
    Entry { meta: &rotozoomer::META, make: rotozoomer::make },
    Entry { meta: &tunnel::META, make: tunnel::make },
    Entry { meta: &synthwave::META, make: synthwave::make },
    Entry { meta: &tv_static::META, make: tv_static::make },
];
