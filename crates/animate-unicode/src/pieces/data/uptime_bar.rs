//! uptime bar: a status page's 60-day uptime bars, one thin bar a day, for
//! up to three services. A degraded day is a short bar, a down day flat.
//! After ascii.rest's uptime-bar.ts (MIT, @bas3line).
//!
//! The days are ascii.rest's, from its seeded generator, and its bars slide
//! in by half a cell. In a status page's colours: a day up is green, a
//! degraded one amber, an outage red, and the headline and each service's
//! word take the colour of how things stand today.

use crate::util::Mulberry32;
use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The uptime bars' meta.
pub static META: Meta = Meta {
    name: "uptime bar",
    slug: "uptime-bar",
    category: Category::Data,
    note: "60-day uptime bars, green, amber and red, a new day sliding in",
    cols: 64,
    rows: 15,
    fps: 8,
    options: Some(r#"{"services": ["api", "website", "database"]}"#),
    // Down, degraded, up; labels; text.
    palette: Some(&["#e85a5a", "#f0b040", "#4ac870", "#6a707c", "#e5e9f0"]),
    ground: Some("#0d0e12"),
    ..Meta::DEFAULT
};

const LABEL: u8 = 3;
const TEXT: u8 = 4;
/// Days shown, services that fit, seconds a day, the end of each step when
/// the bars sit half a cell along, and today at t = 0.
const DAYS: i64 = 60;
const MAX: usize = 3;
const STEP: f64 = 1.25;
const SLIDE: f64 = 0.25;
const START: i64 = 2646;
/// Each state as a left-half bar, and as the same bar half a cell to the
/// left. A down day is flat on the floor.
const BAR: [char; 3] = ['▁', '▖', '▌'];
const MID: [char; 3] = ['▁', '▗', '▐'];
const RATES: [(f64, f64); 3] = [(0.03, 0.01), (0.045, 0.014), (0.025, 0.008)];

fn hash(s: usize, d: i64, k: i64) -> f64 {
    Mulberry32::new((s as i64 * 7919 + d * 104_729 + k * 15_485_863) as u32).next_f64()
}

/// 2 up, 1 degraded, 0 down.
fn base(s: usize, d: i64) -> usize {
    let (deg, down) = RATES[s % RATES.len()];
    let r = hash(s, d, 1);
    if r < down {
        0
    } else if r < down + deg {
        1
    } else {
        2
    }
}

/// An outage is often followed by a degraded day.
fn state(s: usize, d: i64) -> usize {
    let b = base(s, d);
    if b < 2 {
        b
    } else if base(s, d - 1) < 2 && hash(s, d, 2) < 0.5 {
        1
    } else {
        2
    }
}

/// Percent of the day the service was up.
fn uptime(s: usize, d: i64) -> f64 {
    match state(s, d) {
        2 => 100.0,
        1 => 99.9 - hash(s, d, 3) * 0.6,
        _ => 96.0 - hash(s, d, 4) * 9.0,
    }
}

/// The status page's services.
pub struct UptimeBar {
    names: Vec<String>,
}

/// Uptime bars; read `services`, up to three.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let names = options.list_str("services", &["api", "website", "database"]).into_iter().take(MAX).map(|n| n.chars().take(24).collect()).collect();
    Box::new(UptimeBar { names })
}

impl Piece for UptimeBar {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let left = 2;
        let p = (t + 0.5) / STEP;
        let today = START + p.floor() as i64;
        let half = p.fract() > 1.0 - SLIDE / STEP;
        let now: Vec<usize> = (0..self.names.len()).map(|s| state(s, today)).collect();
        let worst = now.iter().copied().min().unwrap_or(2);
        // Fewer services sit in the middle of the frame.
        let mut y = 1 + (MAX - self.names.len()) as i64 * 2;
        grid.clear();
        let head = ["partial outage", "degraded performance", "all systems operational"][worst];
        grid.text_at(left, y, head, worst as u8);
        y += 2;
        for (s, name) in self.names.iter().enumerate() {
            let sum: f64 = (today - DAYS + 1..=today).map(|d| uptime(s, d)).sum();
            let word = ["outage", "degraded", "operational"][now[s]];
            grid.text_at(left, y, name, TEXT);
            grid.text_at(left + DAYS - word.len() as i64, y, word, now[s] as u8);
            // Mid-slide, each cell holds the next day, drawn in its left neighbour's right half.
            for j in 0..DAYS {
                let d = today - DAYS + 1 + j + i64::from(half);
                let st = state(s, d);
                grid.set(left + j, y + 1, if half { MID[st] } else { BAR[st] }, st as u8);
            }
            // "60 days ago ···· 99.92% uptime ···· today", dotted to the bar's width.
            let (a, b, mid) = (format!("{DAYS} days ago "), " today".to_string(), format!(" {:.2}% uptime ", sum / DAYS as f64));
            let gap = DAYS as usize - a.len() - b.len() - mid.len();
            let l = gap / 2;
            grid.text_at(left, y + 2, &format!("{a}{}", "·".repeat(l)), LABEL);
            grid.text_at(left + (a.len() + l) as i64, y + 2, &mid, TEXT);
            grid.text_at(left + (a.len() + l + mid.len()) as i64, y + 2, &format!("{}{b}", "·".repeat(gap - l)), LABEL);
            y += 4;
        }
    }
}
