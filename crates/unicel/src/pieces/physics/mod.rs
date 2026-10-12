//! Simulations: pendulums, cradles, particles: the `physics` category, and the `physics` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod arrow_field;
pub mod bouncing_balls;
pub mod chladni;
pub mod double_pendulum;
pub mod falling_sand;
pub mod flag;
pub mod fountain;
pub mod harmonograph;
pub mod lorenz;
pub mod newtons_cradle;
pub mod pendulum_wave;
pub mod plucked_string;
pub mod pond_ripples;
pub mod smoke;
pub mod wave_interference;

/// The physics pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &arrow_field::META, make: arrow_field::make },
    Entry { meta: &bouncing_balls::META, make: bouncing_balls::make },
    Entry { meta: &chladni::META, make: chladni::make },
    Entry { meta: &double_pendulum::META, make: double_pendulum::make },
    Entry { meta: &falling_sand::META, make: falling_sand::make },
    Entry { meta: &flag::META, make: flag::make },
    Entry { meta: &fountain::META, make: fountain::make },
    Entry { meta: &harmonograph::META, make: harmonograph::make },
    Entry { meta: &lorenz::META, make: lorenz::make },
    Entry { meta: &newtons_cradle::META, make: newtons_cradle::make },
    Entry { meta: &pendulum_wave::META, make: pendulum_wave::make },
    Entry { meta: &plucked_string::META, make: plucked_string::make },
    Entry { meta: &pond_ripples::META, make: pond_ripples::make },
    Entry { meta: &smoke::META, make: smoke::make },
    Entry { meta: &wave_interference::META, make: wave_interference::make },
];
