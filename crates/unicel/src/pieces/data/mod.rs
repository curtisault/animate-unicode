//! Charts and readouts: the `data` category, and the `data` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod bar_chart;
pub mod candlesticks;
pub mod cpu_meters;
pub mod equalizer;
pub mod gauge;
pub mod heartbeat;
pub mod heatmap;
pub mod radar;
pub mod sparkline;
pub mod uptime_bar;

/// The data pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &bar_chart::META, make: bar_chart::make },
    Entry { meta: &candlesticks::META, make: candlesticks::make },
    Entry { meta: &cpu_meters::META, make: cpu_meters::make },
    Entry { meta: &equalizer::META, make: equalizer::make },
    Entry { meta: &gauge::META, make: gauge::make },
    Entry { meta: &heartbeat::META, make: heartbeat::make },
    Entry { meta: &heatmap::META, make: heatmap::make },
    Entry { meta: &radar::META, make: radar::make },
    Entry { meta: &sparkline::META, make: sparkline::make },
    Entry { meta: &uptime_bar::META, make: uptime_bar::make },
];
