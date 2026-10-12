//! hawa mahal: the palace of winds in Jaipur at night. Five storeys of
//! narrow and wide bays step up to a domed crown, with kiosks on every
//! terrace. The windows are jali screens, and lamps behind them come and go.
//! After ascii.rest's hawa-mahal.ts (MIT, @bas3line).
//!
//! The facade and its lamps are ascii.rest's. The facade is drawn in
//! octants, each of ascii.rest's strokes a pattern of dots: the walls and
//! frames solid, the hoods pointed and the windows' sides rounded, so the
//! palace stands as a mass of stone with its bays in relief, and every jali
//! screen a lattice of dots. In colour it is the pink sandstone of Jaipur,
//! cream along its ledges and its balcony, each screen dark until the lamp
//! behind it comes up amber, under twinkling stars.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};

/// The palace's meta.
pub static META: Meta = Meta {
    name: "hawa mahal",
    slug: "hawa-mahal",
    category: Category::Objects,
    note: "jaipur's palace of winds in pink octant stone, amber lamps in the jali",
    cols: 61,
    rows: 24,
    fps: 10,
    charset: Charset::Extended,
    // The wall, the hoods, the ledges, the balcony; a screen dark, glowing,
    // lit; the ground; a star.
    palette: Some(&["#d4685a", "#ec8a7a", "#f2c4a4", "#f6e0c4", "#3e1c1a", "#c46a2a", "#ffc050", "#4a3430", "#8a90b0"]),
    ground: Some("#090a12"),
    ..Meta::DEFAULT
};

const WALL: u8 = 0;
const HOOD: u8 = 1;
const LEDGE: u8 = 2;
const BALCONY: u8 = 3;
const JALI: [u8; 3] = [4, 5, 6];
const EARTH: u8 = 7;
const STAR: u8 = 8;

/// Every "+" is a lattice cell. The top three storeys wear pointed hoods,
/// the fourth sits under a balcony with rounded ones, and the first has tall
/// arched windows on a plinth.
const FACADE: [&str; 24] = [
    "                        '",
    "                       .^.",
    "                 '   .-' '-.   '",
    "                .^.  (+++++)  .^.",
    "              __|_|__|+++++|__|_|__",
    "              .-^-..^..-^-..^..-^-.",
    "          '   (+++)(+)(+++)(+)(+++)   '",
    "         .^.  |+++||+||+++||+||+++|  .^.",
    "        _|_|__|___________________|__|_|_",
    "        .^..^..-^-..^..-^-..^..-^-..^..^.",
    "   '    (+)(+)(+++)(+)(+++)(+)(+++)(+)(+)    '",
    "  .^.   |+||+||+++||+||+++||+||+++||+||+|   .^.",
    "__|_|___|_______________________________|___|_|__",
    ".^..-^-..^..^..-^-..^..-^-..^..-^-..^..^..-^-..^.",
    "(+)(+++)(+)(+)(+++)(+)(+++)(+)(+++)(+)(+)(+++)(+)",
    "|+||+++||+||+||+++||+||+++||+||+++||+||+||+++||+|",
    "=================================================",
    ".-..---..-..-..---..-..---..-..---..-..-..---..-.",
    "(+)(+++)(+)(+)(+++)(+)(+++)(+)(+++)(+)(+)(+++)(+)",
    "|+||+++||+||+||+++||+||+++||+||+++||+||+||+++||+|",
    "/^\\/'^'\\/^\\/^\\/'^'\\/^\\/'^'\\/^\\/'^'\\/^\\/^\\/'^'\\/^\\",
    "|+||+++||+||+||+++||+||+++||+||+++||+||+||+++||+|",
    "|+||+++||+||+||+++||+||+++||+||+++||+||+||+++||+|",
    "|_||___||_||_||___||_||___||_||___||_||_||___||_|",
];
/// Stars in the open sky either side of the steps: row, column, twinkle rate, phase.
const STARS: [(i64, i64, f64, f64); 8] =
    [(0, 4, 0.9, 0.0), (1, 15, 1.3, 2.0), (3, 8, 0.7, 4.0), (6, 2, 1.1, 1.0), (0, 53, 1.2, 3.0), (2, 45, 0.8, 5.0), (4, 57, 1.0, 0.5), (7, 52, 1.4, 2.5)];
/// The share of lamps lit, which the night drifts back toward; seconds for
/// a lamp to come up, and for one to go out.
const SHARE: f64 = 0.3;
const RISE: f64 = 0.3;
const FADE: f64 = 1.8;

fn hash(x: i64, y: i64) -> f64 {
    let mut h = ((x + 1) as u32).wrapping_mul(0x27d4_eb2d) ^ ((y + 1) as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

/// A stroke as octant dots, a row of two a line top to bottom, and its paint.
fn stroke(ch: char) -> Option<([u8; 4], u8)> {
    Some(match ch {
        '\'' => ([0b00, 0b10, 0b10, 0b00], HOOD),
        '.' => ([0b00, 0b00, 0b10, 0b11], HOOD),
        '^' | '`' => ([0b10, 0b11, 0b11, 0b11], HOOD),
        '-' => ([0b00, 0b00, 0b11, 0b11], HOOD),
        '/' => ([0b01, 0b01, 0b11, 0b11], HOOD),
        '\\' => ([0b10, 0b10, 0b11, 0b11], HOOD),
        '(' => ([0b01, 0b11, 0b11, 0b01], WALL),
        ')' => ([0b10, 0b11, 0b11, 0b10], WALL),
        '_' => ([0b11, 0b11, 0b11, 0b11], LEDGE),
        '=' => ([0b11, 0b11, 0b11, 0b11], BALCONY),
        '|' => ([0b11, 0b11, 0b11, 0b11], WALL),
        _ => return None,
    })
}

/// The palace: its fixed dots, the jali cells (row, column, jharokha,
/// checker), each lamp's state and when it last changed, the generator, when
/// the next lamp changes, the clock, and the dots.
pub struct HawaMahal {
    stone: Vec<(i64, i64, u8)>,
    cells: Vec<(i64, i64, usize, usize)>,
    lit: Vec<bool>,
    since: Vec<f64>,
    rand: Mulberry32,
    next: f64,
    clock: f64,
    dots: ColorOctants,
}

/// The palace; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let left = (cols - 49) / 2;
    let mut rand = Mulberry32::new(8);
    let mut stone = vec![];
    let mut cells = vec![];
    // A jharokha is a run of "+" and the run straight below it.
    let mut at = std::collections::HashMap::new();
    let mut count = 0;
    for (y, line) in FACADE.iter().enumerate() {
        let y = y as i64;
        let chars: Vec<char> = line.chars().collect();
        let mut x = 0;
        while x < chars.len() {
            if chars[x] == '+' {
                let start = x;
                while x < chars.len() && chars[x] == '+' {
                    x += 1;
                }
                let w = *at.get(&(y - 1, start)).unwrap_or(&count);
                if w == count {
                    count += 1;
                }
                at.insert((y, start), w);
                for i in start..x {
                    cells.push((y, left + i as i64, w, (i + y as usize) & 1));
                }
                continue;
            }
            if let Some((rows_of, paint)) = stroke(chars[x]) {
                let c = left + x as i64;
                for (j, bits) in rows_of.iter().enumerate() {
                    for i in 0..2 {
                        if bits >> (1 - i) & 1 == 1 {
                            stone.push((c * 2 + i, y * 4 + j as i64, paint));
                        }
                    }
                }
            }
            x += 1;
        }
    }
    // The balcony runs a column past each side, and the ground either side of the plinth.
    let band = FACADE.iter().position(|l| l.starts_with('=')).unwrap_or(16) as i64;
    for j in 0..4 {
        for i in [(left - 1) * 2, (left - 1) * 2 + 1, (left + 49) * 2, (left + 49) * 2 + 1] {
            stone.push((i, band * 4 + j, BALCONY));
        }
    }
    for i in 0..cols * 2 {
        if !(left * 2..(left + 49) * 2).contains(&i) {
            stone.push((i, (rows - 1) * 4 + 3, EARTH));
        }
    }
    // Each lamp keeps whether it is lit and when that last changed. The night
    // opens with the share lit and a few lamps still going out.
    let mut lit = vec![false; count];
    let mut n = (SHARE * count as f64).round() as usize;
    while n > 0 {
        let w = (rand.next_f64() * count as f64) as usize;
        if !lit[w] {
            lit[w] = true;
            n -= 1;
        }
    }
    let mut since = vec![-99.0; count];
    let mut n = 3;
    while n > 0 {
        let w = (rand.next_f64() * count as f64) as usize;
        if !lit[w] && since[w] < -9.0 {
            since[w] = -FADE * (0.3 + 0.5 * rand.next_f64());
            n -= 1;
        }
    }
    Box::new(HawaMahal { stone, cells, lit, since, rand, next: 0.2, clock: 0.0, dots: ColorOctants::new(META.cols, META.rows) })
}

impl HawaMahal {
    /// One lamp changes at a time, a fraction of a second apart. Below the
    /// share, lighting one is likelier than putting one out.
    fn advance(&mut self, t: f64) {
        let count = self.lit.len();
        while self.next <= t {
            let share = self.lit.iter().filter(|&&l| l).count() as f64 / count as f64;
            let light = self.rand.next_f64() < 0.5 + (SHARE - share) * 3.0;
            let pool: Vec<usize> = (0..count).filter(|&w| self.lit[w] != light).collect();
            let pick = (self.rand.next_f64() * pool.len() as f64) as usize;
            if let Some(&w) = pool.get(pick) {
                self.lit[w] = light;
                self.since[w] = self.next;
            }
            self.next += 0.3 + self.rand.next_f64() * 0.5;
        }
    }

    /// A lamp comes up through a glow and goes out through one, and a lit
    /// lamp now and then gutters for a moment.
    fn level(&self, w: usize) -> usize {
        let age = self.clock - self.since[w];
        if !self.lit[w] {
            return if age < 0.2 {
                2
            } else if age < FADE {
                1
            } else {
                0
            };
        }
        if age < RISE || hash(w as i64, (self.clock * 10.0 + 1e-6).floor() as i64) < 0.03 {
            1
        } else {
            2
        }
    }
}

impl Piece for HawaMahal {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        self.clock += (t - self.clock).clamp(0.0, 1.0);
        self.advance(self.clock);
        self.dots.clear();
        for &(x, y, c) in &self.stone {
            self.dots.set(x, y, c);
        }
        // Each screen a lattice of dots: a checker, two dots of four lit.
        for &(y, x, w, k) in &self.cells {
            let level = self.level(w);
            for j in 0..4 {
                for i in 0..2 {
                    if (i + j + k as i64) % 2 == 0 {
                        self.dots.set(x * 2 + i, y * 4 + j, JALI[level]);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        for &(r, c, f, p) in &STARS {
            let b = (f * self.clock + p).sin();
            grid.set(
                c,
                r,
                if b > 0.85 {
                    '•'
                } else if b > -0.3 {
                    '·'
                } else {
                    '.'
                },
                STAR,
            );
        }
    }
}
