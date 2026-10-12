//! dividers: a sheet of rules to set between sections of a page, from a
//! plain line through short centred rules down to ornaments, chains and
//! waves. After ascii.rest's dividers.ts (MIT, @bas3line).
//!
//! The same sheet, each rule shading along its length from teal through
//! violet to rose and back: a cell has one colour, so a rule many cells long
//! becomes a smooth gradient, which a page's rule can never be in one ink.

use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The dividers' meta.
pub static META: Meta = Meta {
    name: "dividers",
    slug: "dividers",
    category: Category::Ui,
    note: "rules and ornaments for between sections, shaded along their length",
    cols: 63,
    rows: 25,
    fps: 0,
    options: Some(r#"{"width": 59}"#),
    // A gradient from teal through violet to rose.
    palette: Some(&["#3ac8c0", "#4aa8d8", "#6a8ae8", "#8a70e8", "#a860d8", "#c858c0", "#e05a9a", "#f06a78"]),
    ground: Some("#0d0c12"),
    ..Meta::DEFAULT
};

/// `unit` repeated to n characters.
fn rep(unit: &str, n: i64) -> String {
    unit.chars().cycle().take(n.max(0) as usize).collect()
}

/// The same string seen in a mirror, so its slanted and pointed glyphs turn too.
fn mirror(s: &str) -> String {
    s.chars()
        .rev()
        .map(|c| match c {
            '<' => '>',
            '>' => '<',
            '╱' => '╲',
            '╲' => '╱',
            '╶' => '╴',
            '╴' => '╶',
            '╡' => '╞',
            '╞' => '╡',
            c => c,
        })
        .collect()
}

/// A left half, an optional middle and the half again in the mirror.
fn sym(half: &str, mid: &str) -> String {
    format!("{half}{mid}{}", mirror(half))
}

fn len(s: &str) -> i64 {
    s.chars().count() as i64
}

/// A centrepiece with a rule run out to either side, n wide.
fn around(mid: &str, fill: &str, n: i64) -> String {
    let side = (n - len(mid)) / 2;
    let centre = if len(mid) + 2 * side < n { format!("{fill}{mid}") } else { mid.to_string() };
    sym(&rep(fill, side), &centre)
}

/// Thin at the ends, heavy in the middle: dots, then dashes, then rules.
fn tapered(n: i64) -> String {
    let h = n / 2;
    if h < 12 {
        return rep("─", n);
    }
    let run = h - 8;
    let thin = (run as f64 * 0.45).ceil() as i64;
    sym(&format!("· · ─ ─ {}{}", rep("─", thin), rep("━", run - thin)), if n % 2 == 1 { "━" } else { "" })
}

/// The sheet, drawn once.
pub struct Dividers {
    sheet: Grid,
}

/// Dividers; reads `width`, the measure the rules run to.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let cols = META.cols as i64;
    let w = (options.f64("width", 59.0).floor() as i64).clamp(13, cols);
    // The ornaments run a fraction of the measure, so they shrink with it.
    let part = |f: f64| ((w as f64 * f).round() as i64).max(5);
    let odd = |n: i64| if n % 2 == 1 { n } else { n - 1 };
    let sheet = [
        rep("─", w),
        rep("═", w),
        rep("· ", odd(w)),
        rep("━", odd(part(0.25))),
        tapered(w),
        around(" · ° · ", "─", w),
        "*     *     *".to_string(),
        sym(&format!("╶{} ", rep("─", (odd(part(0.5)) - 5) / 2)), "°"),
        around("╡ ° ╞", "═", odd(part(0.7))),
        sym(&rep("─<>", (part(0.73) - 1) / 6 * 3), "─"),
        rep("╱╲", part(0.55) - part(0.55) % 2),
        format!("{}_", rep("_.·°·.", (part(0.73) - 1) / 6 * 6)),
    ];
    let mut g = Grid::new(META.cols, META.rows);
    for (i, s) in sheet.iter().enumerate() {
        let n = len(s);
        let left = (cols - n) / 2;
        for (k, ch) in s.chars().enumerate() {
            // Teal to rose and back across the rule, whatever its length.
            let u = if n > 1 { k as f64 / (n - 1) as f64 } else { 0.5 };
            let c = ((1.0 - (2.0 * u - 1.0).abs()) * 7.0).round() as u8;
            g.set(left + k as i64, 2 * i as i64 + 1, ch, c);
        }
    }
    Box::new(Dividers { sheet: g })
}

impl Piece for Dividers {
    fn frame(&mut self, _: f64, _: &Env, grid: &mut Grid) {
        grid.cells_mut().copy_from_slice(self.sheet.cells());
        grid.colors_mut().copy_from_slice(self.sheet.colors());
    }
}
