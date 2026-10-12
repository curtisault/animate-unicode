//! Scenes of cities: the `cities` category, one of the three the scenes are split into,
//! and the `cities` feature that builds it into its own wasm module. A piece here
//! may use the crate's shared code (`crate::pixels` draws them, `crate::noise`
//! shades them) and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod kyoto_dusk;
pub mod marine_drive;
pub mod taj_dawn;
pub mod tokyo_rain;
pub mod varanasi_ghats;

/// The cities scenes, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &tokyo_rain::META, make: tokyo_rain::make },
    Entry { meta: &marine_drive::META, make: marine_drive::make },
    Entry { meta: &taj_dawn::META, make: taj_dawn::make },
    Entry { meta: &varanasi_ghats::META, make: varanasi_ghats::make },
    Entry { meta: &kyoto_dusk::META, make: kyoto_dusk::make },
];
