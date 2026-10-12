//! Scenes of night skies: the `nights` category, one of the three the scenes are split into,
//! and the `nights` feature that builds it into its own wasm module. A piece here
//! may use the crate's shared code (`crate::pixels` draws them, `crate::noise`
//! shades them) and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod aurora_fjord;
pub mod desert_night;
pub mod earthrise;
pub mod lantern_lake;
pub mod night_coast;

/// The nights scenes, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &night_coast::META, make: night_coast::make },
    Entry { meta: &desert_night::META, make: desert_night::make },
    Entry { meta: &aurora_fjord::META, make: aurora_fjord::make },
    Entry { meta: &earthrise::META, make: earthrise::make },
    Entry { meta: &lantern_lake::META, make: lantern_lake::make },
];
