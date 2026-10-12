//! heatmap: a contribution calendar, weeks across and weekdays down, shaded
//! by how busy each day was. Days fill in one at a time; a full week moves
//! it left. After ascii.rest's heatmap.ts (MIT, @bas3line).
//!
//! The activity is ascii.rest's, from its seeded generator. Each day is a
//! square from the geometric shapes, growing with the work done that day,
//! `· ▪ ◼ ■ █`, so the busy weeks read in one ink, and in colour the squares
//! take a code host's greens, from a dark empty day to a bright full one.

use crate::util::Mulberry32;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// The heatmap's meta.
pub static META: Meta = Meta {
    name: "heatmap",
    slug: "heatmap",
    category: Category::Data,
    note: "a contribution calendar in squares that grow, in a code host's greens",
    cols: 66,
    rows: 12,
    fps: 10,
    charset: Charset::Extended,
    options: Some(r#"{"unit": "contributions", "seed": 7}"#),
    // The five levels, none to most; labels; text.
    palette: Some(&["#2a2f38", "#0e4429", "#006d32", "#26a641", "#39d353", "#6a707c", "#c8ccd4"]),
    ground: Some("#0d1117"),
    ..Meta::DEFAULT
};

const LABEL: u8 = 5;
const TEXT: u8 = 6;
/// Weeks shown, seconds a day, and today at t = 0 as days since 1970-01-01.
const WEEKS: i64 = 30;
const STEP: f64 = 0.5;
const START: i64 = 20150;
const LEVELS: [char; 5] = ['·', '▪', '◼', '■', '█'];
const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
const DAYS: [&str; 7] = ["mon", "", "wed", "", "fri", "", ""];

/// Month (0 to 11) of a day counted from 1970-01-01.
fn month_of(z: i64) -> usize {
    let w = crate::Wall { unix: z as f64 * 86_400.0, offset: 0 };
    w.local().month as usize - 1
}

/// The calendar: its unit and seed.
pub struct Heatmap {
    unit: String,
    seed: f64,
}

impl Heatmap {
    fn hash(&self, d: i64, k: i64) -> f64 {
        Mulberry32::new((self.seed * 7919.0 + d as f64 * 104_729.0 + k as f64 * 15_485_863.0) as i64 as u32).next_f64()
    }

    /// Busy and quiet stretches: value noise over weeks, eased between knots.
    fn busy(&self, d: i64) -> f64 {
        let w = d as f64 / 7.0 / 3.0;
        let (i, f) = (w.floor() as i64, w.fract());
        let e = f * f * (3.0 - 2.0 * f);
        self.hash(i, 9) * (1.0 - e) + self.hash(i + 1, 9) * e
    }

    fn count(&self, d: i64) -> i64 {
        let weekend = (d + 3).rem_euclid(7) >= 5;
        let a = (0.08 + 1.05 * self.busy(d).powf(1.4)) * if weekend { 0.3 } else { 1.0 };
        if self.hash(d, 1) > a + 0.08 {
            return 0;
        }
        ((a * (3.0 + 14.0 * self.hash(d, 2))).round() as i64).max(1)
    }
}

fn level(n: i64) -> usize {
    match n {
        0 => 0,
        1..=3 => 1,
        4..=7 => 2,
        8..=11 => 3,
        _ => 4,
    }
}

/// A heatmap; reads `unit` and `seed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(Heatmap { unit: options.string("unit", "contributions"), seed: options.f64("seed", 7.0) })
}

impl Piece for Heatmap {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let left = 6; // weekday labels
        let today = START + (t / STEP).floor() as i64;
        let monday = today - (today + 3).rem_euclid(7);
        let first = monday - (WEEKS - 1) * 7;
        grid.clear();
        let mut total = 0;
        for (r, d) in DAYS.iter().enumerate() {
            grid.text_at(0, 2 + r as i64, d, LABEL);
        }
        for w in 0..WEEKS {
            for r in 0..7 {
                let d = first + w * 7 + r;
                if d <= today {
                    let n = self.count(d);
                    total += n;
                    let l = level(n);
                    grid.set(left + w * 2, 2 + r, LEVELS[l], l as u8);
                }
            }
        }
        // A month's name sits over the first week that starts in it.
        let mut free = 0;
        for w in 0..WEEKS {
            let m = month_of(first + w * 7);
            if w > 0 && m == month_of(first + (w - 1) * 7) {
                continue;
            }
            let x = left + w * 2;
            if x < free || x + 3 > META.cols as i64 || (w == 0 && month_of(first + 14) != m) {
                continue;
            }
            grid.text_at(x, 1, MONTHS[m], LABEL);
            free = x + 5;
        }
        // The total with thousands separated, and the key.
        let digits = total.to_string();
        let mut sep = String::new();
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i) % 3 == 0 {
                sep.push(',');
            }
            sep.push(c);
        }
        grid.text_at(left, 10, &format!("{sep} {} in the last {WEEKS} weeks", self.unit), TEXT);
        let key_x = META.cols as i64 - 20;
        grid.text_at(key_x, 10, "less", LABEL);
        for (i, ch) in LEVELS.iter().enumerate() {
            grid.set(key_x + 5 + 2 * i as i64, 10, *ch, i as u8);
        }
        grid.text_at(key_x + 15, 10, "more", LABEL);
    }
}
