//! animate-unicode: plays a piece in the terminal until a key is pressed.
//! `animate-unicode list` names every piece, by category.
//!
//! Behaviour after ascii.rest's cli.ts by @bas3line (MIT): the same flags,
//! the same list, the same help for a mistyped name, and a still frame as text
//! when the output is not a terminal.

mod paint;
mod play;

use animate_unicode::{registry, Category, Env, Grid, Meta};
use std::io::IsTerminal;
use std::process::ExitCode;

const HELP: &str = "animate-unicode: animated unicode art, in your terminal.

  animate-unicode <piece>     plays a piece until you press a key
  animate-unicode list        every piece, by category

  --mono            a coloured piece in the terminal's own colour
  --light           for a light terminal: shading flipped (a piece on its
                    own ground keeps it)
  --fps <n>         frames a second, instead of the piece's own (up to 60)
  --seconds <n>     stops after n seconds
  --options <json>  the piece's options: '{\"speed\": 2}'
  -h, --help        this help
  -V, --version     the version

  animate-unicode braille-donut
  animate-unicode quadrant-fire --seconds 10
  animate-unicode donut --light

Glyphs: braille, sextants and octants need a recent monospace font (or a
Nerd Font); kanji-rain needs one with CJK, and a terminal that draws it two
cells wide. Piped or redirected, it prints the first frame as text.
";

/// A wrong command line: said on stderr, exit code 2.
struct Usage(String);

struct Args {
    piece: Option<String>,
    mono: bool,
    light: bool,
    fps: Option<u32>,
    seconds: Option<f64>,
    options: String,
    help: bool,
    version: bool,
}

fn parse(args: impl IntoIterator<Item = String>) -> Result<Args, Usage> {
    let mut a = Args { piece: None, mono: false, light: false, fps: None, seconds: None, options: String::new(), help: false, version: false };
    let mut it = args.into_iter();
    let number = |flag: &str, v: Option<String>, max: f64| -> Result<f64, Usage> {
        let v = v.ok_or_else(|| Usage(format!("--{flag} needs a number")))?;
        match v.parse::<f64>() {
            Ok(n) if n > 0.0 && n <= max => Ok(n),
            _ => {
                Err(Usage(format!("--{flag} takes a number above 0{}, not \"{v}\"", if max.is_finite() { format!(" and up to {max}") } else { String::new() })))
            }
        }
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--mono" => a.mono = true,
            "--light" => a.light = true,
            "--fps" => a.fps = Some(number("fps", it.next(), 60.0)?.round() as u32),
            "--seconds" => a.seconds = Some(number("seconds", it.next(), f64::INFINITY)?),
            "--options" => a.options = it.next().ok_or_else(|| Usage("--options needs a JSON object".into()))?,
            "-h" | "--help" => a.help = true,
            "-V" | "--version" => a.version = true,
            s if s.starts_with('-') => return Err(Usage(format!("there is no option {s}; --help lists them"))),
            s if a.piece.is_some() => return Err(Usage(format!("one piece at a time, not also \"{s}\""))),
            s => a.piece = Some(s.to_owned()),
        }
    }
    Ok(a)
}

/// Edit distance, a swap of two neighbours counting as one edit, so "dontu" is one from "donut".
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + usize::from(a[i - 1] != b[j - 1]));
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

/// A piece by its slug, or by the name it shows ("braille donut").
fn find(wanted: &str) -> Result<&'static Meta, Usage> {
    let word = wanted.trim().to_lowercase();
    let slug = word.split_whitespace().collect::<Vec<_>>().join("-");
    if let Some(e) = registry::find(&slug).or_else(|| registry::all().iter().find(|e| e.meta.name == word)) {
        return Ok(e.meta);
    }
    // The names it is part of first, then the nearest typos.
    let mut close: Vec<(bool, usize, &str)> = registry::all()
        .iter()
        .map(|e| (slug.len() > 2 && e.meta.slug.contains(&slug), distance(&slug, e.meta.slug).min(distance(&word, e.meta.name)), e.meta.slug))
        .filter(|&(part, d, _)| part || d <= (slug.chars().count() / 3).max(1))
        .collect();
    close.sort_by(|a, b| b.0.cmp(&a.0).then(if a.0 { std::cmp::Ordering::Equal } else { a.1.cmp(&b.1) }).then(a.2.cmp(b.2)));
    let close: Vec<&str> = close.into_iter().take(8).map(|c| c.2).collect();
    let hint = match close.as_slice() {
        [] => String::new(),
        [one] => format!(" Did you mean {one}?"),
        [rest @ .., last] => format!(" Did you mean {} or {last}?", rest.join(", ")),
    };
    Err(Usage(format!("there is no piece called \"{wanted}\".{hint}\nanimate-unicode list shows every piece.")))
}

/// Every piece's slug by category, in the site's sidebar order, wrapped to `width`.
fn list(width: usize) -> String {
    const PAD: usize = 12;
    let mut s = String::new();
    for &category in Category::ALL {
        let mut slugs: Vec<&str> = registry::all().iter().filter(|e| e.meta.category == category).map(|e| e.meta.slug).collect();
        if slugs.is_empty() {
            continue;
        }
        slugs.sort_unstable();
        let mut line = format!("{:<PAD$}", category.slug());
        for slug in slugs {
            if line.len() > PAD && line.chars().count() + 2 + slug.len() > width {
                s += &line;
                s.push('\n');
                line = " ".repeat(PAD);
            }
            if line.len() > PAD {
                line += "  ";
            }
            line += slug;
        }
        s += &line;
        s.push('\n');
    }
    format!("{s}\n{} pieces. animate-unicode <piece> plays one.\n", registry::all().len())
}

fn run() -> Result<ExitCode, Usage> {
    let args = parse(std::env::args().skip(1))?;
    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }
    let Some(wanted) = args.piece.filter(|_| !args.help) else {
        print!("{HELP}");
        return Ok(ExitCode::SUCCESS);
    };
    if wanted == "list" {
        let width = crossterm::terminal::size().map_or(80, |(c, _)| usize::from(c)).min(100);
        print!("{}", list(width));
        return Ok(ExitCode::SUCCESS);
    }
    let meta = find(&wanted)?;
    let (meta, piece) = registry::make(meta.slug, &args.options).map_err(|e| Usage(e.to_string()))?;

    // Piped or redirected there is nothing to play on: the first frame, as text.
    if !std::io::stdout().is_terminal() {
        let mut piece = piece;
        let mut grid = Grid::new(meta.cols, meta.rows);
        piece.frame(0.0, &Env { paper: args.light }, &mut grid);
        let mut p = paint::Painter::new(meta, false);
        p.paint(&grid);
        println!("{}", p.text());
        return Ok(ExitCode::SUCCESS);
    }

    let settings = play::Settings { mono: args.mono, light: args.light, fps: args.fps, seconds: args.seconds };
    let played = play::play(meta, piece, &settings).map_err(|e| Usage(format!("the terminal failed: {e}")))?;
    if played.cropped {
        let (p, t) = (played.piece, played.terminal);
        eprintln!("{} is {}x{} and this terminal is {}x{}, so only its middle showed. A bigger window shows all of it.", meta.slug, p.0, p.1, t.0, t.1);
    }
    Ok(if played.interrupted { ExitCode::from(130) } else { ExitCode::SUCCESS })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(Usage(message)) => {
            eprintln!("animate-unicode: {message}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Result<Args, Usage> {
        parse(s.split_whitespace().map(String::from))
    }

    #[test]
    fn flags_parse_and_bad_ones_are_usage_errors() {
        let a = args("donut --mono --light --fps 12 --seconds 2.5").ok().unwrap();
        assert_eq!((a.piece.as_deref(), a.mono, a.light, a.fps, a.seconds), (Some("donut"), true, true, Some(12), Some(2.5)));
        for bad in ["donut --fps 0", "donut --fps 99", "donut --seconds -1", "donut --what", "donut wave", "donut --seconds"] {
            assert!(args(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn find_takes_slugs_and_names_and_suggests_the_close_ones() {
        assert_eq!(find("braille-donut").ok().unwrap().slug, "braille-donut");
        assert_eq!(find("Braille Donut").ok().unwrap().slug, "braille-donut");
        let Err(Usage(msg)) = find("dontu") else { panic!("dontu is not a piece") };
        assert!(msg.contains("Did you mean donut?"), "{msg}");
        let Err(Usage(msg)) = find("braille") else { panic!() };
        assert!(msg.contains("braille-donut, braille-lissajous or braille-wave"), "{msg}");
        let Err(Usage(msg)) = find("zzzzzzzz") else { panic!() };
        assert!(!msg.contains("Did you mean"), "{msg}");
    }

    #[test]
    fn distance_counts_a_swap_as_one() {
        assert_eq!(distance("dontu", "donut"), 1);
        assert_eq!(distance("donut", "donut"), 0);
        assert_eq!(distance("", "abc"), 3);
    }

    #[test]
    fn list_has_every_piece_once_by_category() {
        let out = list(60);
        for e in registry::all() {
            assert_eq!(out.matches(&format!(" {}", e.meta.slug)).count() + out.matches(&format!("\n{}", e.meta.slug)).count(), 1, "{}", e.meta.slug);
        }
        assert!(out.lines().all(|l| l.chars().count() <= 60), "{out}");
        assert!(out.contains(&format!("{} pieces.", registry::all().len())));
        // Sidebar order: ui before shapes before effects.
        let at = |c: &str| out.find(&format!("\n{c} ")).or_else(|| out.starts_with(c).then_some(0)).unwrap();
        assert!(at("ui") < at("shapes") && at("shapes") < at("effects"));
    }
}
