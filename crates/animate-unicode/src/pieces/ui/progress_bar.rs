//! progress bar: five progress bar styles stacked. Each fills at its own
//! uneven pace, easing into short stalls the way real jobs do, holds at
//! 100%, restarts. After ascii.rest's progress-bar.ts (MIT, @bas3line).
//!
//! The same five, with ascii.rest's `#` bar turned into braille, filling
//! half a cell at a time over a dotted track; and every bar's filled part
//! shades along its length from teal through green to amber, so the colour
//! says how far it has got as much as the length does.

use crate::util::Mulberry32;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// The progress bars' meta.
pub static META: Meta = Meta {
    name: "progress bar",
    slug: "progress-bar",
    category: Category::Ui,
    note: "five progress bars, one braille, filling teal to amber",
    cols: 48,
    rows: 11,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"labels": ["fetching", "unpacking", "indexing", "building", "linking"]}"#),
    // The fill from start to end; the track and brackets; the labels; done.
    palette: Some(&["#3ac8c0", "#3ad0a0", "#5ad87a", "#9ad860", "#d8d050", "#f0b048", "#4a505c", "#a8aebb", "#ffffff"]),
    ground: Some("#0d0e12"),
    ..Meta::DEFAULT
};

const TRACK: u8 = 6;
const LABEL: u8 = 7;
const DONE: u8 = 8;
/// Every bar is this many cells, ends included.
const W: usize = 28;
const EIGHTHS: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
/// Seconds a full bar stays full, and an empty one waits before the next run.
const HOLD: f64 = 1.1;
const REST: f64 = 0.4;

/// Each style draws progress p (0 to 1) in exactly W cells: a glyph and
/// whether it is part of the fill.
fn style(s: usize, p: f64) -> Vec<(char, bool)> {
    let mut out = vec![];
    match s {
        0 => {
            let n = (p * (W - 2) as f64) as usize;
            out.push(('[', false));
            out.extend(std::iter::repeat_n(('=', true), n));
            if n < W - 2 {
                out.push(('>', true));
                out.extend(std::iter::repeat_n((' ', false), W - 3 - n));
            }
            out.push((']', false));
        }
        1 => {
            let e = (p * (W - 2) as f64 * 8.0).round() as usize;
            let n = e >> 3;
            out.push(('▕', false));
            out.extend(std::iter::repeat_n(('█', true), n));
            if n < W - 2 {
                out.push((EIGHTHS[e & 7], true));
                out.extend(std::iter::repeat_n((' ', false), W - 3 - n));
            }
            out.push(('▏', false));
        }
        2 => {
            // Braille, half a cell at a time: a full cell, a left column, then the dotted track.
            let h = (p * (W - 2) as f64 * 2.0) as usize;
            let n = h / 2;
            out.push(('⢸', false));
            out.extend(std::iter::repeat_n(('⣿', true), n));
            if n < W - 2 {
                out.push(if h % 2 == 1 { ('⡇', true) } else { ('⣀', false) });
                out.extend(std::iter::repeat_n(('⣀', false), W - 3 - n));
            }
            out.push(('⡇', false));
        }
        3 => {
            // Fourteen lamps that light in whole steps.
            let n = (p * 14.0 + 1e-9) as usize;
            for k in 0..14 {
                out.push(if k < n { ('█', true) } else { ('░', false) });
                out.push((' ', false));
            }
        }
        _ => {
            let h = (p * W as f64 * 2.0) as usize;
            let n = h >> 1;
            out.extend(std::iter::repeat_n(('━', true), n));
            if n < W {
                out.push((if h & 1 == 1 { '╾' } else { '─' }, h & 1 == 1));
                out.extend(std::iter::repeat_n(('─', false), W - 1 - n));
            }
        }
    }
    out.truncate(W);
    out
}

/// One bar: its run length and phase, and the knots of its current run.
struct Bar {
    fill: f64,
    phase: f64,
    k: i64,
    knots: Vec<(f64, f64)>,
}

/// The bars and their labels.
pub struct ProgressBar {
    bars: Vec<Bar>,
    labels: Vec<String>,
}

/// One run: a few legs, each eased in and out, so the bar surges and stalls.
fn knots(i: usize, k: i64) -> Vec<(f64, f64)> {
    let mut rand = Mulberry32::new((i as i64 * 7919 + k * 104_729 + 17) as u32);
    let legs = 4 + (rand.next_f64() * 3.0) as usize;
    let (mut dt, mut dv) = (vec![], vec![]);
    for _ in 0..legs {
        dt.push(0.5 + rand.next_f64());
        dv.push(0.2 + rand.next_f64());
    }
    let (st, sv): (f64, f64) = (dt.iter().sum(), dv.iter().sum());
    let mut out = vec![(0.0, 0.0)];
    let (mut u, mut v) = (0.0, 0.0);
    for j in 0..legs {
        u += dt[j] / st;
        v += dv[j] / sv;
        out.push((u, v));
    }
    out[legs] = (1.0, 1.0);
    out
}

impl ProgressBar {
    fn progress(&mut self, i: usize, t: f64) -> f64 {
        let bar = &mut self.bars[i];
        let cycle = bar.fill + HOLD + REST;
        let tt = t + bar.phase;
        let k = (tt / cycle).floor() as i64;
        let u = tt - k as f64 * cycle;
        if u >= bar.fill + HOLD {
            return 0.0;
        }
        if u >= bar.fill {
            return 1.0;
        }
        if bar.k != k {
            bar.k = k;
            bar.knots = knots(i, k);
        }
        let x = u / bar.fill;
        let kn = &bar.knots;
        let mut j = 1;
        while kn[j].0 < x {
            j += 1;
        }
        let ((u0, v0), (u1, v1)) = (kn[j - 1], kn[j]);
        let e = ((x - u0) / (u1 - u0)).clamp(0.0, 1.0);
        v0 + (v1 - v0) * e * e * (3.0 - 2.0 * e)
    }
}

/// Progress bars; reads `labels`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let labels = options.list_str("labels", &["fetching", "unpacking", "indexing", "building", "linking"]);
    // Run lengths and where each bar starts, so frame 0 shows them part way.
    let bars = [(4.2, 2.9), (3.4, 0.7), (5.6, 4.6), (3.8, 1.6), (4.8, 1.9)].iter().map(|&(fill, phase)| Bar { fill, phase, k: -1, knots: vec![] }).collect();
    Box::new(ProgressBar { bars, labels })
}

impl Piece for ProgressBar {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        grid.clear();
        for i in 0..self.bars.len() {
            let p = self.progress(i, t);
            let y = 2 * i as i64 + 1;
            let label: String = self.labels.get(i).map_or("", String::as_str).chars().take(9).collect();
            grid.text_at(2, y, &label, LABEL);
            let cells = style(i, p);
            let filled = cells.iter().filter(|c| c.1).count().max(1);
            let mut k = 0;
            for (x, &(ch, fill)) in cells.iter().enumerate() {
                // The fill shades from teal toward amber along its length.
                let c = if fill {
                    k += 1;
                    ((k as f64 / filled as f64) * p * 5.0).round() as u8
                } else {
                    TRACK
                };
                grid.set(12 + x as i64, y, ch, c);
            }
            let pct = format!("{:>4}", format!("{}%", (p * 100.0) as u32));
            grid.text_at(12 + W as i64 + 2, y, &pct, if p >= 1.0 { DONE } else { LABEL });
        }
    }
}
