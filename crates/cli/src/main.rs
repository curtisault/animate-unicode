//! animate-unicode <piece> [--seconds n] [--paper] [--options json]: plays a
//! piece in the terminal. `animate-unicode list` names every piece.
//!
//! A stub for the scaffold: plain ANSI, no raw mode, no key to stop it, no
//! colour. Phase 7 of docs/IMPLEMENTATION_PLAN.md replaces this with a
//! crossterm player (alternate screen, truecolor from the palette, resize).

use animate_unicode::{registry, Env, Grid};
use std::io::Write;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seconds = f64::INFINITY;
    let mut paper = false;
    let mut options = String::new();
    let mut slug = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                seconds = args.get(i).and_then(|s| s.parse().ok()).unwrap_or_else(|| usage("--seconds needs a number"));
            }
            "--paper" => paper = true,
            "--options" => {
                i += 1;
                options = args.get(i).cloned().unwrap_or_else(|| usage("--options needs JSON"));
            }
            "-h" | "--help" => usage(""),
            s if s.starts_with('-') => usage(&format!("unknown flag {s}")),
            s => slug = Some(s.to_string()),
        }
        i += 1;
    }
    let Some(slug) = slug else { usage("") };
    if slug == "list" {
        for e in registry::all() {
            println!("{:<20} {:<10} {}x{} {}fps  {}", e.meta.slug, e.meta.category.slug(), e.meta.cols, e.meta.rows, e.meta.fps, e.meta.note);
        }
        return;
    }
    let (meta, mut piece) = match registry::make(&slug, &options) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let mut grid = Grid::new(meta.cols, meta.rows);
    let env = Env { paper };
    let out = std::io::stdout();
    let mut out = out.lock();
    let start = Instant::now();
    let step = if meta.fps > 0 { Duration::from_secs_f64(1.0 / meta.fps as f64) } else { Duration::MAX };
    // \x1b[2J clears, \x1b[H homes the cursor, \x1b[?25l hides it until we leave.
    let _ = write!(out, "\x1b[2J\x1b[?25l");
    loop {
        let t = start.elapsed().as_secs_f64();
        if t > seconds {
            break;
        }
        piece.frame(t, &env, &mut grid);
        let _ = writeln!(out, "\x1b[H{}", grid.text().replace('\n', "\x1b[K\n"));
        let _ = out.flush();
        if meta.fps == 0 {
            break;
        }
        std::thread::sleep(step);
    }
    let _ = writeln!(out, "\x1b[?25h");
}

fn usage(err: &str) -> ! {
    if !err.is_empty() {
        eprintln!("{err}\n");
    }
    eprintln!("animate-unicode <piece> [--seconds n] [--paper] [--options '{{\"speed\":2}}']\nanimate-unicode list");
    std::process::exit(if err.is_empty() { 0 } else { 2 });
}
