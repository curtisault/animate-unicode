//! Things: clocks, machines: the `objects` category, and the `objects`
//! feature that builds it into its own wasm module. A piece here may use the
//! crate's shared code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod analog_clock;
pub mod candle;
pub mod coffee;
pub mod ferris_wheel;
pub mod hawa_mahal;
pub mod hourglass;
pub mod kite;
pub mod lava_lamp;
pub mod lighthouse;
pub mod skyline;
pub mod sundial;
pub mod train;
pub mod vinyl;
pub mod windmill;

/// The objects pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &analog_clock::META, make: analog_clock::make },
    Entry { meta: &candle::META, make: candle::make },
    Entry { meta: &coffee::META, make: coffee::make },
    Entry { meta: &ferris_wheel::META, make: ferris_wheel::make },
    Entry { meta: &hawa_mahal::META, make: hawa_mahal::make },
    Entry { meta: &hourglass::META, make: hourglass::make },
    Entry { meta: &kite::META, make: kite::make },
    Entry { meta: &lava_lamp::META, make: lava_lamp::make },
    Entry { meta: &lighthouse::META, make: lighthouse::make },
    Entry { meta: &skyline::META, make: skyline::make },
    Entry { meta: &sundial::META, make: sundial::make },
    Entry { meta: &train::META, make: train::make },
    Entry { meta: &vinyl::META, make: vinyl::make },
    Entry { meta: &windmill::META, make: windmill::make },
];
