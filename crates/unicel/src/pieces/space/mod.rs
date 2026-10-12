//! Planets, stars, orbits: the `space` category, and the `space` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod black_hole;
pub mod earth;
pub mod eclipse;
pub mod galaxy;
pub mod moon_phases;
pub mod planet;
pub mod rocket;
pub mod saptarishi;
pub mod solar_system;
pub mod starfield;
pub mod three_body;

/// The space pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &black_hole::META, make: black_hole::make },
    Entry { meta: &earth::META, make: earth::make },
    Entry { meta: &eclipse::META, make: eclipse::make },
    Entry { meta: &galaxy::META, make: galaxy::make },
    Entry { meta: &moon_phases::META, make: moon_phases::make },
    Entry { meta: &planet::META, make: planet::make },
    Entry { meta: &rocket::META, make: rocket::make },
    Entry { meta: &saptarishi::META, make: saptarishi::make },
    Entry { meta: &solar_system::META, make: solar_system::make },
    Entry { meta: &starfield::META, make: starfield::make },
    Entry { meta: &three_body::META, make: three_body::make },
];
