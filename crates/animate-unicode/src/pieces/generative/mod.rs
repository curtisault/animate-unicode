//! Patterns from a rule or a formula: the `generative` category, and the `generative` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod braille_lissajous;
pub mod braille_wave;
pub mod epicycles;
pub mod flow_field;
pub mod geometric_tiles;
pub mod glider_gun;
pub mod hilbert_curve;
pub mod julia_set;
pub mod langtons_ant;
pub mod mandelbrot;
pub mod maze;
pub mod plasma;
pub mod reaction_diffusion;
pub mod rule_30;
pub mod sextant_plasma;
pub mod sierpinski;
pub mod voronoi;

/// The generative pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &braille_wave::META, make: braille_wave::make },
    Entry { meta: &braille_lissajous::META, make: braille_lissajous::make },
    Entry { meta: &sextant_plasma::META, make: sextant_plasma::make },
    Entry { meta: &geometric_tiles::META, make: geometric_tiles::make },
    Entry { meta: &epicycles::META, make: epicycles::make },
    Entry { meta: &flow_field::META, make: flow_field::make },
    Entry { meta: &glider_gun::META, make: glider_gun::make },
    Entry { meta: &hilbert_curve::META, make: hilbert_curve::make },
    Entry { meta: &julia_set::META, make: julia_set::make },
    Entry { meta: &langtons_ant::META, make: langtons_ant::make },
    Entry { meta: &mandelbrot::META, make: mandelbrot::make },
    Entry { meta: &maze::META, make: maze::make },
    Entry { meta: &plasma::META, make: plasma::make },
    Entry { meta: &reaction_diffusion::META, make: reaction_diffusion::make },
    Entry { meta: &rule_30::META, make: rule_30::make },
    Entry { meta: &sierpinski::META, make: sierpinski::make },
    Entry { meta: &voronoi::META, make: voronoi::make },
];
