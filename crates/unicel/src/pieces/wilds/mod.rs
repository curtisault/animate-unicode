//! Scenes of wild places: the `wilds` category, one of the three the scenes are split into,
//! and the `wilds` feature that builds it into its own wasm module. A piece here
//! may use the crate's shared code (`crate::pixels` draws them, `crate::noise`
//! shades them) and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod alpine_dawn;
pub mod deep_reef;
pub mod misty_forest;
pub mod ocean_sunset;
pub mod storm_plains;

/// The wilds scenes, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &alpine_dawn::META, make: alpine_dawn::make },
    Entry { meta: &misty_forest::META, make: misty_forest::make },
    Entry { meta: &ocean_sunset::META, make: ocean_sunset::make },
    Entry { meta: &storm_plains::META, make: storm_plains::make },
    Entry { meta: &deep_reef::META, make: deep_reef::make },
];
