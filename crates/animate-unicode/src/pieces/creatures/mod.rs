//! Animals and characters: the `creatures` category, and the `creatures`
//! feature that builds it into its own wasm module. A piece here may use the
//! crate's shared code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod aquarium;
pub mod butterfly;
pub mod cat;
pub mod fox;
pub mod jellyfish;
pub mod owl;
pub mod snake;
pub mod spider;
pub mod starlings;
pub mod whale;

/// The creatures pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &aquarium::META, make: aquarium::make },
    Entry { meta: &butterfly::META, make: butterfly::make },
    Entry { meta: &cat::META, make: cat::make },
    Entry { meta: &fox::META, make: fox::make },
    Entry { meta: &jellyfish::META, make: jellyfish::make },
    Entry { meta: &owl::META, make: owl::make },
    Entry { meta: &snake::META, make: snake::make },
    Entry { meta: &spider::META, make: spider::make },
    Entry { meta: &starlings::META, make: starlings::make },
    Entry { meta: &whale::META, make: whale::make },
];
