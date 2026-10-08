//! The piece contract, run against every registered piece. A port of
//! ascii.rest's scripts/check.ts; the rules are spelled out in
//! docs/IMPLEMENTATION_PLAN.md under "The piece contract".
//!
//! Timing budgets only mean something optimised: `cargo test --release`
//! (what `mise run test` does). In a debug build they are skipped.

use animate_unicode::{registry, Category, Charset, Env, Grid, Options, Piece, WIDE_TAIL};
use std::time::Instant;

const SAMPLE_AT: [f64; 6] = [0.0, 1.0, 2.0, 3.0, 4.0, 6.0];

struct Run {
    frames: Vec<Grid>,
    ms: Vec<f64>,
}

/// Plays a fresh instance from t = 0 in 1/fps steps, keeping the frames at the sample times.
fn play(entry: &registry::Entry, times: &[f64], paper: bool) -> Run {
    let meta = entry.meta;
    let mut piece: Box<dyn Piece> = (entry.make)(&Options::default().with_defaults(meta.options));
    let mut grid = Grid::new(meta.cols, meta.rows);
    let env = Env { paper };
    let step = if meta.fps > 0 { 1.0 / meta.fps as f64 } else { 0.5 };
    let end = times.iter().cloned().fold(0.0, f64::max);
    let mut frames = vec![None; times.len()];
    let mut ms = vec![];
    let mut i = 0;
    loop {
        let t = (i as f64 * step * 1e6).round() / 1e6;
        if t > end + 1e-9 {
            break;
        }
        let at = Instant::now();
        piece.frame(t, &env, &mut grid);
        if i > 0 {
            ms.push(at.elapsed().as_secs_f64() * 1000.0);
        }
        for (k, want) in times.iter().enumerate() {
            if (want - t).abs() < step / 2.0 + 1e-9 && frames[k].is_none() {
                frames[k] = Some(grid.clone());
            }
        }
        i += 1;
    }
    Run { frames: frames.into_iter().map(|f| f.expect("every sample time is hit")).collect(), ms }
}

fn allowed(charset: Charset, cp: u32) -> bool {
    let basic = matches!(cp, 0x20..=0x7E | 0xB7 | 0xB0 | 0x2022 | 0x25CF | 0x2500..=0x259F);
    let extended = matches!(cp, 0x2190..=0x21FF | 0x25A0..=0x25FF | 0x2800..=0x28FF | 0x1FB00..=0x1FBFF);
    match charset {
        Charset::Basic => basic,
        Charset::Extended => basic || extended,
        Charset::Wide => char::from_u32(cp).is_some(),
    }
}

#[test]
fn every_piece_keeps_the_contract() {
    let mut failures = vec![];
    for entry in registry::all() {
        let m = entry.meta;
        let mut errors: Vec<String> = vec![];
        let slug = m.slug;

        // meta
        let kebab = !slug.is_empty() && slug.split('-').all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
        if !kebab {
            errors.push("slug must be kebab-case".into());
        }
        if m.name.is_empty() {
            errors.push("meta.name".into());
        }
        if m.note.is_empty() || m.note.chars().count() > 72 {
            errors.push("meta.note must be 1-72 chars".into());
        }
        let scene = m.category == Category::Scenes;
        let (max_cols, max_rows) = if scene { (320, 120) } else { (80, 32) };
        if m.cols < 1 || m.cols > max_cols {
            errors.push(format!("meta.cols must be 1-{max_cols}"));
        }
        if m.rows < 1 || m.rows > max_rows {
            errors.push(format!("meta.rows must be 1-{max_rows}"));
        }
        if m.fps > 60 {
            errors.push("meta.fps must be 0-60".into());
        }
        if let Some(p) = m.palette {
            let hex = |c: &str| c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|h| h.is_ascii_hexdigit());
            if p.len() < 2 || p.len() > 64 || !p.iter().all(|c| hex(c)) {
                errors.push("meta.palette must be 2-64 colours as #rrggbb".into());
            }
        }
        if scene && (m.palette.is_none() || m.ground.is_none()) {
            errors.push("a scene needs meta.palette and meta.ground".into());
        }
        if !(m.cell == 1 || m.cell == 2) {
            errors.push("meta.cell must be 1 or 2".into());
        }
        if let Some(o) = m.options {
            if Options::from_json(o).map(|o| o.is_empty()).unwrap_or(true) {
                errors.push("meta.options must be a non-empty JSON object, or None".into());
            }
        }
        if !errors.is_empty() {
            failures.push(format!("{slug}: {}", errors.join("; ")));
            continue;
        }

        // frames
        let run = play(entry, &SAMPLE_AT, false);
        let run2 = play(entry, &SAMPLE_AT, false);
        let paper = play(entry, &[0.0, 1.0], true);
        for (k, g) in run.frames.iter().enumerate() {
            let at = format!("t={}", SAMPLE_AT[k]);
            let text = g.text();
            if text.lines().count() != m.rows {
                errors.push(format!("{at}: {} lines, meta.rows is {}", text.lines().count(), m.rows));
            }
            for (x, cp) in g.cells().iter().enumerate() {
                if *cp == WIDE_TAIL {
                    if x % m.cols == 0 || g.cells()[x - 1] == WIDE_TAIL {
                        errors.push(format!("{at}: wide tail at cell {x} with no wide glyph before it"));
                        break;
                    }
                    if m.charset != Charset::Wide {
                        errors.push(format!("{at}: wide glyph in a {:?} piece", m.charset));
                        break;
                    }
                } else if !allowed(m.charset, *cp) {
                    errors.push(format!("{at}: U+{cp:04X} is outside charset {:?}", m.charset));
                    break;
                }
            }
            if let Some(p) = m.palette {
                if let Some(bad) = g.colors().iter().position(|&c| c as usize >= p.len()) {
                    errors.push(format!("{at}: color[{bad}] is past the {}-colour palette", p.len()));
                }
            }
        }
        let f0 = &run.frames[0];
        let ink = f0.cells().iter().filter(|&&c| c != 0x20 && c != WIDE_TAIL).count();
        let cells = (m.cols * m.rows) as f64;
        if (ink as f64) / cells < 0.02 && ink < 3 {
            errors.push("first frame is nearly empty (frame 0 should be a good still)".into());
        }
        let mut distinct: Vec<&[u32]> = run.frames.iter().map(|g| g.cells()).collect();
        distinct.sort_unstable();
        distinct.dedup();
        if m.fps > 0 && distinct.len() < 3 {
            errors.push(format!("animated ({} fps) but only {} distinct frames over 6s", m.fps, distinct.len()));
        }
        if m.fps == 0 && distinct.len() > 1 && !m.clock {
            errors.push("fps 0 but frames change".into());
        }
        if !m.clock && run.frames != run2.frames {
            errors.push("not deterministic (seed your PRNG; a clock only with meta.clock)".into());
        }
        if paper.frames[0].text().lines().count() != m.rows {
            errors.push("paper frame is the wrong size".into());
        }
        if let Some(p) = m.palette {
            let used: std::collections::HashSet<u8> = f0.cells().iter().zip(f0.colors()).filter(|(c, _)| **c != 0x20).map(|(_, &k)| k).collect();
            if scene && used.len() < 4 {
                errors.push(format!("only {} of {} palette colours on inked cells in the first frame", used.len(), p.len()));
            }
        }
        if cfg!(not(debug_assertions)) && !run.ms.is_empty() {
            let avg = run.ms.iter().sum::<f64>() / run.ms.len() as f64;
            let max = run.ms.iter().cloned().fold(0.0, f64::max);
            let (budget, worst) = if scene { (10.0, 40.0) } else { (4.0, 30.0) };
            if avg > budget {
                errors.push(format!("slow: {avg:.2}ms a frame on average (budget {budget}ms)"));
            }
            if max > worst {
                errors.push(format!("slow: worst frame {max:.1}ms (budget {worst}ms)"));
            }
        }

        if errors.is_empty() {
            let avg = if run.ms.is_empty() { 0.0 } else { run.ms.iter().sum::<f64>() / run.ms.len() as f64 };
            println!("ok   {slug}  {}  {}x{}  {}fps  {avg:.2}ms", m.category.slug(), m.cols, m.rows, m.fps);
        } else {
            failures.push(format!("{slug}: {}", errors.join("; ")));
        }
    }
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

/// The donut must match ascii.rest's donut.ts cell for cell: the port's proof.
/// `fixtures/donut.txt` holds the TS piece's frames at t = 0, 1.3 and 7.7,
/// captured with `node -e` against ../ascii/src/pieces/donut.ts.
#[test]
fn donut_matches_the_typescript_original() {
    let fixture = include_str!("fixtures/donut.txt");
    let (meta, mut piece) = registry::make("donut", "").unwrap();
    let mut grid = Grid::new(meta.cols, meta.rows);
    for block in fixture.split("\n\n") {
        let (head, body) = block.split_once('\n').unwrap();
        let t: f64 = head.strip_prefix("t=").unwrap().parse().unwrap();
        piece.frame(t, &Env::default(), &mut grid);
        assert_eq!(grid.text(), body.trim_end_matches('\n'), "donut at t={t}");
    }
}
