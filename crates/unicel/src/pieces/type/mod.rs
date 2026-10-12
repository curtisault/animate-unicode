//! Lettering and text effects: the `type` category, and the `type` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod big_text;
pub mod dissolve;
pub mod glitch;
pub mod marquee;
pub mod morse;
pub mod scramble;
pub mod split_flap;
pub mod typewriter;
pub mod wave_text;

/// The type pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &typewriter::META, make: typewriter::make },
    Entry { meta: &scramble::META, make: scramble::make },
    Entry { meta: &wave_text::META, make: wave_text::make },
    Entry { meta: &split_flap::META, make: split_flap::make },
    Entry { meta: &big_text::META, make: big_text::make },
    Entry { meta: &marquee::META, make: marquee::make },
    Entry { meta: &morse::META, make: morse::make },
    Entry { meta: &glitch::META, make: glitch::make },
    Entry { meta: &dissolve::META, make: dissolve::make },
];
