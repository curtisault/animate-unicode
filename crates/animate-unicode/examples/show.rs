//! Prints a piece's frames at t = 0, 1, 2.5 and 5 seconds inside a ruled box:
//! a quick look at a piece without a browser. A port of the `--show` printer
//! in ascii.rest's scripts/check.ts (MIT, @bas3line).
//!
//!     cargo run --release -q -p animate-unicode --example show -- donut
//!     cargo run --release -q -p animate-unicode --example show -- braille-wave --paper --options '{"speed": 2}'
//!
//! With no slugs it shows every piece. `mise run show donut` is the same.

use animate_unicode::{registry, Env, Grid};

const SHOW_AT: [f64; 4] = [0.0, 1.0, 2.5, 5.0];

fn main() {
    let mut slugs = vec![];
    let (mut paper, mut options) = (false, String::new());
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--paper" => paper = true,
            "--options" => options = args.next().unwrap_or_else(|| fail("--options needs a JSON object")),
            "-h" | "--help" => fail(""),
            s if s.starts_with('-') => fail(&format!("unknown flag {s}")),
            s => slugs.push(s.to_owned()),
        }
    }
    if slugs.is_empty() {
        slugs = registry::all().iter().map(|e| e.meta.slug.to_owned()).collect();
    }
    for slug in &slugs {
        let (meta, mut piece) = registry::make(slug, &options).unwrap_or_else(|e| fail(&e.to_string()));
        let mut grid = Grid::new(meta.cols, meta.rows);
        let env = Env { paper };
        // Played from 0 in 1/fps steps, as a viewer would see it.
        let step = 1.0 / f64::from(meta.fps.max(1));
        let rule = format!("+{}+", "-".repeat(meta.cols));
        let mut i = 0u32;
        for want in SHOW_AT {
            loop {
                let t = f64::from(i) * step;
                piece.frame(t, &env, &mut grid);
                i += 1;
                if t + step / 2.0 > want {
                    break;
                }
            }
            println!("\n{slug} at t={want}s\n{rule}");
            for line in grid.text().lines() {
                println!("|{line}|");
            }
            println!("{rule}");
        }
        println!();
    }
}

fn fail(err: &str) -> ! {
    if !err.is_empty() {
        eprintln!("{err}\n");
    }
    eprintln!("show [slug ...] [--paper] [--options '{{\"speed\": 2}}']   no slugs = every piece");
    std::process::exit(if err.is_empty() { 0 } else { 2 });
}
