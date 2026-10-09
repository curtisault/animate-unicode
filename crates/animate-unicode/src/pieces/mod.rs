//! Every piece. To add one: a file here, a `pub mod`, and one line in REGISTRY.
//! The contract test (`tests/contract.rs`) checks it against the rules in
//! docs/IMPLEMENTATION_PLAN.md.

use crate::registry::Entry;

pub mod arrow_field;
pub mod box_frames;
pub mod braille_donut;
pub mod braille_lissajous;
pub mod braille_wave;
pub mod cube;
pub mod dna_helix;
pub mod donut;
pub mod epicycles;
pub mod flow_field;
pub mod geometric_tiles;
pub mod glider_gun;
pub mod glxgears;
pub mod gyroscope;
pub mod heart;
pub mod hilbert_curve;
pub mod icosahedron;
pub mod julia_set;
pub mod kanji_rain;
pub mod langtons_ant;
pub mod mandelbrot;
pub mod maze;
pub mod mobius_strip;
pub mod octant_sphere;
pub mod plasma;
pub mod quadrant_fire;
pub mod reaction_diffusion;
pub mod rule_30;
pub mod sextant_plasma;
pub mod sierpinski;
pub mod spring;
pub mod tesseract;
pub mod torus_knot;
pub mod twisted_ring;
pub mod voronoi;

// One piece a line, so this stays the one place to see every piece.
/// Every piece, in the order the site lists them within a category.
#[rustfmt::skip]
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &donut::META, make: donut::make },
    Entry { meta: &braille_wave::META, make: braille_wave::make },
    Entry { meta: &braille_lissajous::META, make: braille_lissajous::make },
    Entry { meta: &sextant_plasma::META, make: sextant_plasma::make },
    Entry { meta: &geometric_tiles::META, make: geometric_tiles::make },
    Entry { meta: &arrow_field::META, make: arrow_field::make },
    Entry { meta: &quadrant_fire::META, make: quadrant_fire::make },
    Entry { meta: &kanji_rain::META, make: kanji_rain::make },
    Entry { meta: &braille_donut::META, make: braille_donut::make },
    Entry { meta: &octant_sphere::META, make: octant_sphere::make },
    Entry { meta: &box_frames::META, make: box_frames::make },
    Entry { meta: &cube::META, make: cube::make },
    Entry { meta: &dna_helix::META, make: dna_helix::make },
    Entry { meta: &heart::META, make: heart::make },
    Entry { meta: &icosahedron::META, make: icosahedron::make },
    Entry { meta: &tesseract::META, make: tesseract::make },
    Entry { meta: &torus_knot::META, make: torus_knot::make },
    Entry { meta: &mobius_strip::META, make: mobius_strip::make },
    Entry { meta: &twisted_ring::META, make: twisted_ring::make },
    Entry { meta: &glxgears::META, make: glxgears::make },
    Entry { meta: &gyroscope::META, make: gyroscope::make },
    Entry { meta: &spring::META, make: spring::make },
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
