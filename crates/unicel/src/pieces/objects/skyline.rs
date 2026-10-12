//! skyline: a city at night across a river. Office windows go on and off,
//! sometimes a whole floor at once, the mast on the tallest tower blinks,
//! and the low lights and the moon are drawn down into the water as streaks.
//! After ascii.rest's skyline.ts (MIT, @bas3line).
//!
//! The towers, their windows and the switching are ascii.rest's; the city is
//! drawn in octants. The near towers are fine outlines with their setbacks,
//! the far ones a sparse haze; every pane is a square of four dots, lit or a
//! single dim dot. In colour some offices burn warm tungsten and some cool
//! fluorescent, the mast's light blinks red, the moon is a pale crescent, and
//! each low light and the moon run down into the river as streaks of their
//! own colour, broken by the ripples.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};

/// The skyline's meta.
pub static META: Meta = Meta {
    name: "skyline",
    slug: "skyline",
    category: Category::Objects,
    note: "an octant city at night, warm and cool windows, a red mast, a river",
    cols: 64,
    rows: 23,
    fps: 10,
    charset: Charset::Extended,
    // Outlines; haze; a dark pane; warm and cool panes; the mast's light; the
    // moon; a star; the embankment.
    palette: Some(&["#5a6478", "#2c3244", "#2a2e3a", "#ffc464", "#9ad4ff", "#ff3a3a", "#f4ecc8", "#8a90a8", "#3a3e4a"]),
    ground: Some("#05060c"),
    ..Meta::DEFAULT
};

const OUTLINE: u8 = 0;
const HAZE: u8 = 1;
const UNLIT: u8 = 2;
const WARM: u8 = 3;
const COOL: u8 = 4;
const MAST: u8 = 5;
const MOON_LIGHT: u8 = 6;
const STAR: u8 = 7;
const BANK: u8 = 8;
/// The embankment row (towers stand on it, water below), and the share of
/// windows lit, which the night drifts back toward.
const BASE: i64 = 18;
const SHARE: f64 = 0.4;
/// Far towers in haze: left column, width, height in rows.
const FAR: [(i64, i64, i64); 6] = [(3, 6, 9), (14, 6, 12), (25, 5, 9), (36, 6, 12), (48, 7, 10), (57, 6, 12)];
/// A near tower: left column, width, height, its setbacks stacked on top
/// (inset, rows), and what stands on its roof.
type Tower = (i64, i64, i64, &'static [(i64, i64)], Option<&'static [&'static str]>);
/// Near towers, and their rooftops, bottom row first.
const NEAR: [Tower; 9] = [
    (-3, 8, 5, &[], None),
    (6, 7, 8, &[], Some(&["╨ ╨", "▄▄▄"])),
    (13, 5, 6, &[], None),
    (19, 9, 10, &[(1, 1), (1, 1), (1, 1)], None),
    (29, 7, 13, &[(1, 1)], Some(&["╱│╲", " │ ", " │ "])),
    (37, 5, 7, &[], None),
    (42, 7, 10, &[], Some(&["╱   ╲", " ╱ ╲ ", "  ╿  "])),
    (50, 5, 5, &[], None),
    (55, 11, 8, &[(1, 2)], Some(&["╭───╮", " ╭─╮ "])),
];
const STARS: [(i64, i64); 12] = [(1, 18), (4, 23), (2, 27), (6, 40), (1, 45), (7, 61), (3, 56), (5, 50), (0, 36), (3, 38), (8, 1), (0, 59)];
/// The moon: centre column and row, radius in rows.
const MOON: (f64, f64, f64) = (9.0, 3.5, 1.9);

/// The city: its still dots, the rooftops' glyphs, the panes (row, column,
/// far, tower, warm), which are lit, when the next pane and the next floor
/// switch, the clock, the mast's light, the generator, and the dots.
pub struct Skyline {
    still: Vec<(i64, i64, u8)>,
    tops: Vec<(i64, i64, char)>,
    panes: Vec<(i64, i64, bool, i64, bool)>,
    lit: Vec<bool>,
    next: f64,
    floor: f64,
    clock: f64,
    mast: (i64, i64),
    rand: Mulberry32,
    dots: ColorOctants,
}

/// A skyline; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let cols = META.cols as i64;
    let mut rand = Mulberry32::new(19);
    let mut still = vec![];
    let mut tops = vec![];
    let mut near_cell = vec![false; (cols * BASE) as usize];
    let mut panes: Vec<(i64, i64, bool, i64, bool)> = vec![];
    // The near towers: outlines round a grid of panes, each setback's walls
    // landing on the tier below it.
    for (n, &(x, w, h, steps, top)) in NEAR.iter().enumerate() {
        let (mut l, mut r, mut t) = (x, x + w - 1, BASE - h);
        let mut tier = |row: i64, l: i64, r: i64, panes: &mut Vec<(i64, i64, bool, i64, bool)>, still: &mut Vec<(i64, i64, u8)>| {
            for c in l..=r {
                if (0..cols).contains(&c) {
                    near_cell[(row * cols + c) as usize] = true;
                }
            }
            for j in 0..4 {
                still.push((l * 2, row * 4 + j, OUTLINE));
                still.push((r * 2 + 1, row * 4 + j, OUTLINE));
            }
            let mut c = l + 2;
            while c < r {
                panes.push((row, c, false, n as i64, false));
                c += 2;
            }
        };
        let roof = |t: i64, l: i64, r: i64, still: &mut Vec<(i64, i64, u8)>| {
            for i in l * 2..=r * 2 + 1 {
                still.push((i, t * 4 + 2, OUTLINE));
            }
        };
        for row in t + 1..BASE {
            tier(row, l, r, &mut panes, &mut still);
        }
        roof(t, l, r, &mut still);
        for &(inset, tall) in steps {
            l += inset;
            r -= inset;
            for row in t - tall + 1..t {
                tier(row, l, r, &mut panes, &mut still);
            }
            t -= tall;
            roof(t, l, r, &mut still);
            for j in t * 4 + 2..(t + tall) * 4 + 3 {
                still.push((l * 2, j, OUTLINE));
                still.push((r * 2 + 1, j, OUTLINE));
            }
        }
        if let Some(art) = top {
            for (i, s) in art.iter().enumerate() {
                let start = (l + r + 1 - s.chars().count() as i64) / 2;
                for (j, ch) in s.chars().enumerate() {
                    if ch != ' ' {
                        tops.push((start + j as i64, t - 1 - i as i64, ch));
                    }
                }
            }
        }
    }
    // The far towers as haze, with their panes where no near tower stands in front.
    for &(x, w, h) in &FAR {
        for row in BASE - h..BASE {
            for c in x..x + w {
                if near_cell[(row * cols + c) as usize] {
                    continue;
                }
                for j in 0..4 {
                    for i in 0..2 {
                        if dither((c * 2 + i) as usize, (row * 4 + j) as usize, 0.22) {
                            still.push((c * 2 + i, row * 4 + j, HAZE));
                        }
                    }
                }
            }
        }
        let mut row = BASE - h + 1;
        while row < BASE {
            let mut c = x + 1;
            while c < x + w - 1 {
                if !near_cell[(row * cols + c) as usize] {
                    panes.push((row, c, true, -1, false));
                }
                c += 2;
            }
            row += 2;
        }
    }
    panes.retain(|&(_, c, ..)| (0..cols).contains(&c));
    // Some offices burn warm tungsten, some cool fluorescent: a tower's panes
    // mostly one, with a few of the other.
    for p in &mut panes {
        p.4 = rand.next_f64() < if p.3 % 2 == 0 { 0.75 } else { 0.3 };
    }
    // The moon, a crescent: lit where inside its disc and outside the shadow's.
    let (mx, my, mr) = MOON;
    for j in ((my - mr) * 4.0) as i64..((my + mr) * 4.0) as i64 {
        for i in ((mx - 2.0 * mr) * 2.0) as i64..((mx + 2.0 * mr) * 2.0) as i64 {
            let (x, y) = (((i as f64 + 0.5) / 2.0 - mx) / 2.0, (j as f64 + 0.5) / 4.0 - my);
            if x.hypot(y) < mr && (x - 1.1).hypot(y) > mr * 0.95 {
                still.push((i, j, MOON_LIGHT));
            }
        }
    }
    for i in 0..cols * 2 {
        for j in 0..2 {
            still.push((i, BASE * 4 + j, BANK));
        }
    }
    let lit = panes.iter().map(|_| rand.next_f64() < SHARE).collect();
    let (_, mw, mh, ..) = NEAR[4];
    let mast = (NEAR[4].0 + mw / 2, BASE - mh - 5);
    Box::new(Skyline { still, tops, panes, lit, next: 0.2, floor: 0.9, clock: 0.0, mast, rand, dots: ColorOctants::new(META.cols, META.rows) })
}

impl Skyline {
    /// A pane to switch: one in the other state, and for a floor a near one.
    fn pick(&mut self, on: bool, near: bool) -> usize {
        let n = self.lit.len();
        let mut w = (self.rand.next_f64() * n as f64) as usize;
        for _ in 0..n {
            if self.lit[w] != on && !(near && self.panes[w].2) {
                break;
            }
            w = (w + 1) % n;
        }
        w
    }

    /// One pane changes at a time, and below the share lighting one is
    /// likelier than putting one out; every couple of seconds a whole floor
    /// of one near tower switches together.
    fn advance(&mut self, t: f64) {
        while self.next.min(self.floor) <= t {
            let share = self.lit.iter().filter(|&&l| l).count() as f64 / self.lit.len() as f64;
            let on = self.rand.next_f64() < 0.5 + (SHARE - share) * 4.0;
            if self.floor < self.next {
                let w = self.pick(on, true);
                let (r, _, _, n, _) = self.panes[w];
                for (i, p) in self.panes.iter().enumerate() {
                    if p.0 == r && p.3 == n {
                        self.lit[i] = on;
                    }
                }
                self.floor += 1.6 + self.rand.next_f64() * 1.2;
            } else {
                let w = self.pick(on, false);
                self.lit[w] = on;
                self.next += 0.12 + self.rand.next_f64() * 0.3;
            }
        }
    }
}

impl Piece for Skyline {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        self.clock += (t - self.clock).clamp(0.0, 1.0);
        self.advance(self.clock);
        self.dots.clear();
        for &(x, y, c) in &self.still {
            self.dots.set(x, y, c);
        }
        // Each pane a square of four dots, lit, or one dim dot.
        let mut glow = vec![None; cols as usize]; // lit panes low enough to reach the water
        for (i, &(r, c, far, _, warm)) in self.panes.iter().enumerate() {
            let color = if warm { WARM } else { COOL };
            if self.lit[i] {
                for (dx, dy) in [(0, 1), (1, 1), (0, 2), (1, 2)] {
                    self.dots.set(c * 2 + dx, r * 4 + dy, color);
                }
                if !far && r > BASE - 4 {
                    glow[c as usize] = Some((r - (BASE - 4), color));
                }
            } else {
                self.dots.set(c * 2, r * 4 + 1, UNLIT);
            }
        }
        // Stars that twinkle, and the blinking mast.
        for &(r, c) in &STARS {
            if (self.clock * 1.3 + c as f64 * 2.0).sin() > -0.3 {
                self.dots.set(c * 2, r * 4 + 1, STAR);
            }
        }
        if self.clock % 1.6 < 0.4 {
            let (mc, mr) = self.mast;
            for (dx, dy) in [(0, 1), (1, 1), (0, 2), (1, 2)] {
                self.dots.set(mc * 2 + dx, mr * 4 + dy, MAST);
            }
        }
        // The water: under each low light and under the moon, a streak running
        // straight down, broken by ripples and thinning with depth.
        let mx = MOON.0;
        for j in (BASE * 4 + 2)..rows * 4 {
            let k = (j as f64 / 4.0) - BASE as f64;
            for i in 0..cols * 2 {
                let c = i as f64 / 2.0;
                let ripple = (c * 1.7 + k * 2.3 - self.clock * 3.1).sin() + 0.6 * (c * 0.53 - k * 1.3 + self.clock * 1.9).sin();
                let in_moon = (c + 0.5 - (mx - 1.0)).abs() < 2.8 - k * 0.45;
                if in_moon && ripple > -1.1 + k * 0.35 {
                    self.dots.set(i, j, MOON_LIGHT);
                } else if let Some((g, color)) = glow[(i / 2) as usize] {
                    if i % 2 == 0 && ripple > -0.9 + k * 0.45 - g as f64 * 0.2 {
                        self.dots.set(i, j, color);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        for &(c, r, ch) in &self.tops {
            grid.set(c, r, ch, OUTLINE);
        }
    }
}
