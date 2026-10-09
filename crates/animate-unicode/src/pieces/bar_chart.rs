//! bar chart: a vertical bar chart with a value axis, gridlines and category
//! labels. The bars ease from one dataset to the next with their values on
//! top. After ascii.rest's bar-chart.ts (MIT, @bas3line).
//!
//! In colour, as a dashboard draws it: each dataset has its own colour, the
//! bars take the colour of the one they are showing and its tab is lit, so
//! a change of dataset is a change of colour as well as of height; the axis
//! and the dashed gridlines sit back in grey.

use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The bar chart's meta.
pub static META: Meta = Meta {
    name: "bar chart",
    slug: "bar-chart",
    category: Category::Data,
    note: "bars easing between datasets, each dataset its own colour",
    cols: 60,
    rows: 19,
    fps: 24,
    options: Some(
        r#"{"title": "visits by day", "labels": ["mon", "tue", "wed", "thu", "fri", "sat", "sun"], "datasets": [{"name": "week 1", "values": [320, 410, 380, 460, 540, 240, 180]}, {"name": "week 2", "values": [210, 480, 300, 560, 390, 340, 150]}, {"name": "week 3", "values": [450, 290, 520, 380, 580, 160, 270]}]}"#,
    ),
    // Four dataset colours (more datasets cycle); axis, grid, text, values.
    palette: Some(&["#5aa0f4", "#4ad8b0", "#f4b04a", "#e070c0", "#6a707c", "#3a3f4a", "#c8ccd4", "#ffffff"]),
    ground: Some("#0e0f13"),
    ..Meta::DEFAULT
};

const AXIS: u8 = 4;
const GRID: u8 = 5;
const TEXT: u8 = 6;
const VALUE: u8 = 7;
const EIGHTHS: [char; 7] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇'];
/// Plot height in rows, bar width; seconds each dataset rests, then moves;
/// where in the first rest play time begins.
const PH: i64 = 12;
const BW: i64 = 4;
const HOLD: f64 = 1.4;
const MOVE: f64 = 1.0;
const START: f64 = 0.8;

/// The smallest round step that splits the range into at most four bands.
fn nice_step(max: f64) -> f64 {
    let raw = max / 4.0;
    let mag = 10f64.powf(raw.log10().floor());
    [1.0, 2.0, 2.5, 5.0, 10.0].iter().map(|m| m * mag).find(|&s| s >= raw).unwrap_or(10.0 * mag)
}

/// The chart: its datasets, the axes drawn once, and where the bars go.
pub struct BarChart {
    datasets: Vec<(String, Vec<f64>)>,
    base: Grid,
    top: f64,
    bx: Vec<i64>,
    grid_rows: Vec<i64>,
    ax: i64,
    pw: i64,
}

/// A bar chart; reads `title`, `labels` and `datasets` (a list of `{name, values}`).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let title = options.string("title", "visits by day");
    let labels = options.list_str("labels", &["mon", "tue", "wed", "thu", "fri", "sat", "sun"]);
    let mut datasets: Vec<(String, Vec<f64>)> = options
        .json("datasets")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|d| {
                    let name = d.get("name")?.as_str()?.to_string();
                    let values = d.get("values")?.as_array()?.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                    Some((name, values))
                })
                .collect()
        })
        .unwrap_or_default();
    if datasets.is_empty() {
        datasets.push(("data".into(), vec![0.0; labels.len()]));
    }
    let cols = META.cols as i64;
    let n = labels.len() as i64;
    let peak = datasets.iter().flat_map(|d| d.1.iter().copied()).fold(1.0, f64::max);
    let step = nice_step(peak);
    let top = (peak / step).ceil() * step;
    let ticks = (top / step).round() as i64;
    let tick_labels: Vec<String> = (0..=ticks).map(|i| format!("{}", (i as f64 * step) as i64)).collect();
    let lw = tick_labels.iter().map(String::len).max().unwrap_or(1) as i64;
    // Layout: tick labels, the axis, then the bars with a gap's width either side.
    let gap = ((cols - 4 - lw - n * BW) / (n + 1)).clamp(1, 3);
    let pw = n * BW + (n + 1) * gap;
    let x0 = (cols - (lw + 1 + pw)) / 2;
    let ax = x0 + lw;
    let bx: Vec<i64> = (0..n).map(|i| ax + 1 + gap + i * (BW + gap)).collect();
    let ay = 4 + PH; // the axis row, which is the zero line
    let mut base = Grid::new(META.cols, META.rows);
    base.text_at(x0, 1, &title, TEXT);
    for r in 4..ay {
        base.set(ax, r, '│', AXIS);
    }
    let mut grid_rows = vec![];
    for (i, s) in tick_labels.iter().enumerate() {
        let r = ay - (i as f64 * PH as f64 / ticks as f64).round() as i64;
        base.text_at(ax - 1 - s.len() as i64, r, s, AXIS);
        if i > 0 {
            base.set(ax, r, '┤', AXIS);
            grid_rows.push(r);
        }
    }
    base.set(ax, ay, '└', AXIS);
    for c in ax + 1..=ax + pw {
        base.set(c, ay, '─', AXIS);
    }
    for (i, s) in labels.iter().enumerate() {
        let s: String = s.chars().take((BW + gap) as usize).collect();
        base.text_at(bx[i] + (BW - s.len() as i64) / 2, ay + 1, &s, TEXT);
    }
    Box::new(BarChart { datasets, base, top, bx, grid_rows, ax, pw })
}

impl Piece for BarChart {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let ease = |x: f64| if x < 0.5 { 4.0 * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(3) / 2.0 };
        let nd = self.datasets.len();
        let cycle = HOLD + MOVE;
        let tt = t + START;
        let k = (tt / cycle).floor() as usize;
        let u = ((tt - k as f64 * cycle - HOLD) / MOVE).max(0.0);
        let (a, b) = (&self.datasets[k % nd], &self.datasets[(k + 1) % nd]);
        let e = ease(u);
        let live = if e < 0.5 { k % nd } else { (k + 1) % nd };
        let hue = (live % 4) as u8;
        grid.cells_mut().copy_from_slice(self.base.cells());
        grid.colors_mut().copy_from_slice(self.base.colors());
        // The dataset names as tabs, the live one in brackets and lit.
        let tabs: Vec<String> = self.datasets.iter().enumerate().map(|(i, d)| if i == live { format!("[{}]", d.0) } else { format!(" {} ", d.0) }).collect();
        let width = tabs.iter().map(|s| s.chars().count() + 1).sum::<usize>() as i64 - 1;
        let mut x = self.ax + self.pw + 1 - width;
        for (i, s) in tabs.iter().enumerate() {
            grid.text_at(x, 1, s, if i == live { (i % 4) as u8 } else { AXIS });
            x += s.chars().count() as i64 + 1;
        }
        let ay = 4 + PH;
        let mut tops = vec![];
        for (i, &bx) in self.bx.iter().enumerate() {
            let v = a.1.get(i).copied().unwrap_or(0.0) * (1.0 - e) + b.1.get(i).copied().unwrap_or(0.0) * e;
            let h = (v / self.top * PH as f64 * 8.0).round() as i64;
            let (full, part) = (h >> 3, h & 7);
            for r in 0..full {
                for c in 0..BW {
                    grid.set(bx + c, ay - 1 - r, '█', hue);
                }
            }
            if part > 0 {
                for c in 0..BW {
                    grid.set(bx + c, ay - 1 - full, EIGHTHS[part as usize - 1], hue);
                }
            }
            tops.push((v, ay - 1 - full - i64::from(part > 0)));
        }
        // Dashed gridlines behind the bars, kept a cell clear of each bar and value.
        for &r in &self.grid_rows {
            for c in self.ax + 1..=self.ax + self.pw {
                let near = self.bx.iter().enumerate().any(|(i, &x)| c >= x - 1 && c <= x + BW && r >= tops[i].1);
                if !near {
                    grid.set(c, r, '┈', GRID);
                }
            }
        }
        for (i, &(v, lr)) in tops.iter().enumerate() {
            let label = format!("{}", v.round() as i64);
            grid.text_at(self.bx[i] + (BW - label.len() as i64) / 2, lr, &label, VALUE);
        }
    }
}
