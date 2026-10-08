//! Times 60 frames of every piece and prints a table, for spotting a
//! regression before it reaches the contract test's hard budgets.
//!
//!     cargo run --release -q -p animate-unicode --example bench
//!     cargo run --release -q -p animate-unicode --example bench -- donut braille-wave
//!
//! `mise run bench` is the same. The budgets are the contract's: 4 ms average
//! and 30 ms worst a frame, or 10 and 40 for scenes. Native timings; wasm in a
//! browser is typically slower.

use animate_unicode::{registry, Category, Env, Grid, Options};
use std::time::Instant;

const FRAMES: u32 = 60;

fn main() {
    if cfg!(debug_assertions) {
        eprintln!("note: a debug build; run with --release for numbers that mean anything\n");
    }
    let want: Vec<String> = std::env::args().skip(1).collect();
    for w in &want {
        if registry::find(w).is_none() {
            eprintln!("there is no piece named \"{w}\" (slugs: {})", registry::slugs().join(", "));
            std::process::exit(2);
        }
    }
    println!("{:<24} {:>7} {:>6} {:>8} {:>8} {:>10}", "piece", "size", "cells", "avg ms", "worst ms", "budget");
    let mut over = 0;
    for entry in registry::all().iter().filter(|e| want.is_empty() || want.iter().any(|w| w == e.meta.slug)) {
        let m = entry.meta;
        let mut piece = (entry.make)(&Options::default().with_defaults(m.options));
        let mut grid = Grid::new(m.cols, m.rows);
        let env = Env::default();
        let step = 1.0 / f64::from(m.fps.max(1));
        piece.frame(0.0, &env, &mut grid); // first frame: allocations, caches
        let mut ms = Vec::with_capacity(FRAMES as usize);
        for i in 1..=FRAMES {
            let at = Instant::now();
            piece.frame(f64::from(i) * step, &env, &mut grid);
            ms.push(at.elapsed().as_secs_f64() * 1000.0);
        }
        let avg = ms.iter().sum::<f64>() / ms.len() as f64;
        let worst = ms.iter().copied().fold(0.0, f64::max);
        let (b_avg, b_worst) = if m.category == Category::Scenes { (10.0, 40.0) } else { (4.0, 30.0) };
        let flag = if avg > b_avg || worst > b_worst {
            over += 1;
            "  OVER"
        } else {
            ""
        };
        let size = format!("{}x{}", m.cols, m.rows);
        println!("{:<24} {size:>7} {:>6} {avg:>8.3} {worst:>8.3} {:>10}{flag}", m.slug, m.cols * m.rows, format!("{b_avg}/{b_worst}"));
    }
    if over > 0 {
        println!("\n{over} over budget");
    }
}
