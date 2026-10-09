//! Cyberpunk cities in neon and rain: the `cyberpunk` category, scenes of our
//! own rather than ascii.rest's, and the `cyberpunk` feature that builds it
//! into its own wasm module. A piece here may use the crate's shared code
//! (`crate::pixels` draws them, `crate::noise` shades them) and this folder's
//! other pieces, never another category's.

use crate::registry::Entry;

pub mod arcology;
pub mod holo_koi;
pub mod maglev;
pub mod noodle_bar;
pub mod undercity;

/// The cyberpunk scenes, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &arcology::META, make: arcology::make },
    Entry { meta: &undercity::META, make: undercity::make },
    Entry { meta: &noodle_bar::META, make: noodle_bar::make },
    Entry { meta: &holo_koi::META, make: holo_koi::make },
    Entry { meta: &maglev::META, make: maglev::make },
];
