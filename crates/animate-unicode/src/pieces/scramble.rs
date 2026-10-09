//! scramble: a decode effect. Each phrase breaks into flickering glyphs from
//! the left, and the next one settles out of them a letter at a time. After
//! ascii.rest's scramble.ts (MIT, @bas3line).
//!
//! The timing is ascii.rest's, from its seeded generator; the noise is drawn
//! from Unicode rather than ASCII punctuation, braille, sextants, quadrants,
//! arrows and geometric shapes, so the scrambled line looks like a signal
//! being pulled out of static; the noise flickers in cyan, green and violet
//! and the settled letters are bright.

use crate::util::Mulberry32;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// The scramble's meta.
pub static META: Meta = Meta {
    name: "scramble",
    slug: "scramble",
    category: Category::Type,
    note: "phrases decoding out of flickering braille, sextants and arrows",
    cols: 44,
    rows: 3,
    fps: 20,
    charset: Charset::Extended,
    options: Some(r#"{"phrases": ["hello, world", "decoding the signal", "nothing is lost", "see you on the other side"]}"#),
    // Three noise colours; the settled text.
    palette: Some(&["#4ac8e8", "#5ad890", "#a888f0", "#f4f6fa"]),
    ground: Some("#0b0c10"),
    ..Meta::DEFAULT
};

const TEXT: u8 = 3;
const NOISE: &str = "⠋⠛⡏⣿⢷⣼⠿⡱⢎⣤🬀🬃🬎🬞🬲🬹🬐🬥▚▞▙▟▘▗◢◣◤◥◆◇○●◐◑←↑→↓↖↗↘↙▒░";
/// Seconds between columns breaking up, between columns settling, of pure
/// noise before the first settles, and that a settled phrase stays.
const OUT: f64 = 0.022;
const IN: f64 = 0.05;
const LEAD: f64 = 0.32;
const HOLD: f64 = 1.9;

/// A phrase and where it sits.
#[derive(Clone)]
struct Placed {
    text: Vec<char>,
    x: i64,
}

/// One change of phrase: from, to, the columns it covers, when each breaks
/// up and settles, and when it starts and ends.
struct Step {
    from: Placed,
    to: Placed,
    lo: i64,
    hi: i64,
    out: Vec<f64>,
    settle: Vec<f64>,
    end: f64,
}

/// The scramble: its changes, the loop's length, and where play begins.
pub struct Scramble {
    steps: Vec<Step>,
    period: f64,
    offset: f64,
    noise: Vec<char>,
}

/// A scramble; reads `phrases`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let mut list = options.list_str("phrases", &["hello, world", "decoding the signal", "nothing is lost", "see you on the other side"]);
    if list.is_empty() {
        list.push(String::new());
    }
    let cols = META.cols as i64;
    let placed: Vec<Placed> = list
        .iter()
        .map(|p| {
            let text: Vec<char> = p.chars().take(META.cols - 4).collect();
            Placed { x: (cols - text.len() as i64) / 2, text }
        })
        .collect();
    let mut rand = Mulberry32::new(77);
    // One change per phrase, into it from the one before: when each column
    // breaks up and when it settles, a little ragged.
    let mut at = 0.0;
    let mut steps = vec![];
    for i in 0..placed.len() {
        let (from, to) = (placed[(i + placed.len() - 1) % placed.len()].clone(), placed[i].clone());
        let lo = from.x.min(to.x);
        let hi = (from.x + from.text.len() as i64).max(to.x + to.text.len() as i64);
        let (mut out, mut settle) = (vec![], vec![]);
        for c in lo..hi {
            out.push(at + (c - lo) as f64 * OUT + rand.next_f64() * 0.06);
            settle.push(at + LEAD + (c - lo) as f64 * IN + rand.next_f64() * 0.14);
        }
        at = settle.iter().copied().fold(at + LEAD, f64::max) + HOLD;
        steps.push(Step { from, to, lo, hi, out, settle, end: at });
    }
    // Frame 0 holds the first phrase, settled, just before it breaks up.
    let offset = steps[0].end - 0.7;
    Box::new(Scramble { steps, period: at, offset, noise: NOISE.chars().collect() })
}

fn char_at(p: &Placed, c: i64) -> char {
    if c >= p.x && c < p.x + p.text.len() as i64 {
        p.text[(c - p.x) as usize]
    } else {
        ' '
    }
}

impl Piece for Scramble {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let now = (t + self.offset).rem_euclid(self.period);
        let s = self.steps.iter().find(|s| now < s.end).unwrap_or(&self.steps[0]);
        let tick = (now * 20.0).floor() as i64;
        let y = META.rows as i64 / 2;
        grid.clear();
        for c in s.lo..s.hi {
            let k = (c - s.lo) as usize;
            let (a, b) = (char_at(&s.from, c), char_at(&s.to, c));
            if now < s.out[k] {
                grid.set(c, y, a, TEXT);
            } else if now >= s.settle[k] {
                grid.set(c, y, b, TEXT);
            } else if !(a == ' ' && b == ' ' && now < s.out[k] + 0.12) {
                // A new glyph for this column on every tick, the same in every instance.
                let mut h = ((c as u32) ^ (tick as u32).wrapping_mul(0x27d4_eb2d)).wrapping_mul(0x9e37_79b1);
                h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca77);
                h ^= h >> 13;
                grid.set(c, y, self.noise[h as usize % self.noise.len()], (h >> 8) as u8 % 3);
            }
        }
    }
}
