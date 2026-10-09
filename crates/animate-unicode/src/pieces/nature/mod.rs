//! Weather, water, plants: the `nature` category, and the `nature` feature
//! that builds it into its own wasm module. A piece here may use the crate's
//! shared code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod aurora;
pub mod bonsai;
pub mod campfire;
pub mod cherry_blossom;
pub mod contour_map;
pub mod fern;
pub mod fireflies;
pub mod fractal_tree;
pub mod landscape;
pub mod lightning;
pub mod rain;
pub mod ruled_mountains;
pub mod sea_swell;
pub mod snowfall;
pub mod sunrise;
pub mod wind;

/// The nature pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &aurora::META, make: aurora::make },
    Entry { meta: &bonsai::META, make: bonsai::make },
    Entry { meta: &campfire::META, make: campfire::make },
    Entry { meta: &cherry_blossom::META, make: cherry_blossom::make },
    Entry { meta: &contour_map::META, make: contour_map::make },
    Entry { meta: &fern::META, make: fern::make },
    Entry { meta: &fireflies::META, make: fireflies::make },
    Entry { meta: &fractal_tree::META, make: fractal_tree::make },
    Entry { meta: &lightning::META, make: lightning::make },
    Entry { meta: &rain::META, make: rain::make },
    Entry { meta: &ruled_mountains::META, make: ruled_mountains::make },
    Entry { meta: &sea_swell::META, make: sea_swell::make },
    Entry { meta: &snowfall::META, make: snowfall::make },
    Entry { meta: &wind::META, make: wind::make },
    Entry { meta: &landscape::META, make: landscape::make },
    Entry { meta: &sunrise::META, make: sunrise::make },
];
