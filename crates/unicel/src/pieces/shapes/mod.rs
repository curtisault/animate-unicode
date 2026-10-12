//! Geometry turning or morphing: the `shapes` category, and the `shapes` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod braille_donut;
pub mod cube;
pub mod dna_helix;
pub mod donut;
pub mod glxgears;
pub mod gyroscope;
pub mod heart;
pub mod icosahedron;
pub mod mobius_strip;
pub mod octant_sphere;
pub mod spring;
pub mod tesseract;
pub mod torus_knot;
pub mod twisted_ring;

/// The shapes pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &donut::META, make: donut::make },
    Entry { meta: &braille_donut::META, make: braille_donut::make },
    Entry { meta: &octant_sphere::META, make: octant_sphere::make },
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
];
