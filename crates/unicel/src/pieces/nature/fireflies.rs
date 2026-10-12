//! fireflies: a meadow at night, tufts of grass under a far treeline and a
//! crescent moon. Fireflies drift over it, each flashing on its own slow
//! beat, until one lights early and the flash runs fly to fly across the
//! meadow. After ascii.rest's fireflies.ts (MIT, @bas3line).
//!
//! The flies, their beats and the sweeping flash are ascii.rest's; the night
//! is redrawn in braille. Each fly's light is a halo that falls off dot by
//! dot round it, pale yellow at the heart through yellow-green to a faint
//! green edge that stays behind the grass, with a dim trail where it was
//! brighter a moment ago. The treeline is a dark silhouette, the moon a pale
//! crescent, the grass fine blades swaying in dark green.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::collections::HashMap;
use std::f64::consts::TAU;

/// Fireflies' meta.
pub static META: Meta = Meta {
    name: "fireflies",
    slug: "fireflies",
    category: Category::Nature,
    note: "braille fireflies glowing yellow-green over grass, a flash spreading",
    cols: 60,
    rows: 18,
    fps: 15,
    charset: Charset::Extended,
    options: Some(r#"{"seed": 7, "count": 44}"#),
    // Glow, faint edge to heart; the treeline; the moon; a star; grass; ground.
    palette: Some(&["#3c7a2c", "#9ad83c", "#e4f878", "#fbffd8", "#15241c", "#efe6bc", "#7c8296", "#2c5a2c", "#24321e"]),
    ground: Some("#05070c"),
    ..Meta::DEFAULT
};

const GLOW: Ramp = Ramp::new(0, 4);
const TREES: u8 = 4;
const MOON: u8 = 5;
const STAR: u8 = 6;
const GRASS: u8 = 7;
const EARTH: u8 = 8;

/// The far treeline, as ascii.rest drew it: its tops give the silhouette.
const TREELINE: [&str; 3] = [
    "                  .-.                         _",
    "     .-.   .--. .'   `.  .-.       .--.    .-' `-.   .-.",
    "__.-'   `-'    `       `-'   `-.__.'    `--'      `-'   `-._",
];
/// Seconds between flashes that sweep the meadow, how fast one runs in
/// columns a second, seconds into a sweep at t = 0, the ground row, and where
/// the treeline meets the meadow.
const WAVE: f64 = 11.0;
const SPEED: f64 = 13.0;
const START: f64 = 2.4;
const GROUND: f64 = 17.0;
const HORIZON: f64 = 8.0;

fn hash(a: i64, b: i64) -> f64 {
    Mulberry32::new(((a as u32).wrapping_mul(0x9e37_79b1)) ^ ((b + 1) as u32).wrapping_mul(0x85eb_ca6b)).next_f64()
}

/// How bright a fly is a fraction ph through its beat: a quick rise, then a slower fade.
fn glow(ph: f64) -> f64 {
    if ph < 0.04 {
        ph / 0.04
    } else {
        (-(ph - 0.04) / 0.13).exp()
    }
}

/// A fly: where it hovers, and the rates and phases of the waves it wanders on.
struct Fly {
    x: f64,
    y: f64,
    w: [f64; 3],
    p: [f64; 3],
}

impl Fly {
    fn place(&self, s: f64) -> (f64, f64) {
        (
            self.x + 6.0 * (self.w[0] * s + self.p[0]).sin() + 2.5 * (self.w[1] * s + self.p[1]).sin(),
            self.y + 1.4 * (self.w[2] * s + self.p[2]).sin() + 0.6 * (self.w[1] * 1.7 * s + self.p[0]).sin(),
        )
    }
}

/// The meadow: the fixed sky and treeline dots, the stars (dot, phase), the
/// grass blades (column, height, lean, phase), the flies, each sweep's flash
/// times, the glow field, the seed, and the dots.
pub struct Fireflies {
    sky: Vec<(i64, i64, u8)>,
    stars: Vec<(i64, i64, f64)>,
    blades: Vec<(f64, f64, f64, f64)>,
    flies: Vec<Fly>,
    hits: HashMap<i64, Vec<(f64, f64)>>,
    field: Vec<f32>,
    seed: i64,
    dots: ColorDots,
}

/// Fireflies; reads `seed` and `count`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let cols = META.cols as f64;
    let seed = options.f64("seed", 7.0) as i64;
    let count = options.f64("count", 44.0).clamp(1.0, 200.0) as usize;
    let mut rnd = Mulberry32::new(seed as u32);
    let dots = ColorDots::new(META.cols, META.rows);
    let (w, h) = (dots.width() as i64, dots.height() as i64);
    let stars = (0..16).map(|_| ((rnd.next_f64() * (cols - 14.0) * 2.0) as i64, (rnd.next_f64() * 5.0 * 4.0) as i64, rnd.next_f64())).collect();
    let mut sky = vec![];
    // A crescent moon: its disc, less the same disc moved toward the sun.
    let (mx, my, mr) = ((cols - 8.5) * 2.0, 3.0 * 4.0, 10.0);
    for y in 0..24 {
        for x in (mx - 12.0) as i64..(mx + 12.0) as i64 {
            let (dx, dy) = (x as f64 + 0.5 - mx, y as f64 + 0.5 - my);
            if dx.hypot(dy) < mr && (dx - 4.5).hypot(dy + 1.5) > mr * 0.92 {
                sky.push((x, y, MOON));
            }
        }
    }
    // The treeline: where each column's tree tops are in ascii.rest's
    // outline, filled down to the horizon.
    for x in 0..w {
        let c = (x / 2) as usize;
        let top = TREELINE.iter().enumerate().find_map(|(i, line)| {
            let ch = line.chars().nth(c).filter(|&ch| ch != ' ')?;
            Some(
                HORIZON - 3.0
                    + i as f64
                    + if ch == '_' {
                        0.8
                    } else if ch == '.' || ch == '-' || ch == '`' || ch == '\'' {
                        0.45
                    } else {
                        0.2
                    },
            )
        });
        let top = top.unwrap_or(HORIZON - 0.2);
        for y in (top * 4.0) as i64..(HORIZON * 4.0) as i64 {
            sky.push((x, y, TREES));
        }
    }
    for x in 0..w {
        sky.push((x, (GROUND * 4.0) as i64 + 2, EARTH));
    }
    // Grass in tufts with room between them: blades fanning out from one
    // root, a straight one in the middle the tallest.
    let fans: [&[f64]; 5] = [&[-1.0, 0.0, 1.0], &[-1.0, 0.0, 1.0], &[0.0, 1.0], &[-1.0, 0.0], &[-1.0, 1.0]];
    let mut blades = vec![];
    let mut x = 2.0 + rnd.next_f64() * 3.0;
    while x < cols - 2.0 {
        let hgt = 2.0 + (rnd.next_f64() * 4.0).floor();
        let c = x.floor();
        for &lean in fans[(rnd.next_f64() * fans.len() as f64) as usize] {
            let bh = if lean != 0.0 { (hgt - 1.0 - (rnd.next_f64() * 1.6).floor()).max(1.0) } else { hgt };
            blades.push((c, bh, lean, x * 0.23));
        }
        x += 4.0 + rnd.next_f64() * 7.0;
    }
    // Each fly wanders on a few slow waves over the meadow and the grass tips.
    let flies = (0..count)
        .map(|_| Fly {
            x: 3.0 + rnd.next_f64() * (cols - 6.0),
            y: 9.3 + rnd.next_f64() * 4.6,
            w: [0.05 + rnd.next_f64() * 0.08, 0.11 + rnd.next_f64() * 0.1, 0.07 + rnd.next_f64() * 0.08],
            p: [rnd.next_f64() * TAU, rnd.next_f64() * TAU, rnd.next_f64() * TAU],
        })
        .collect();
    Box::new(Fireflies { sky, stars, blades, flies, hits: HashMap::new(), field: vec![0.0; (w * h) as usize], seed, dots })
}

impl Fireflies {
    /// Each sweep starts at the fly furthest to one side, the sides taking
    /// turns, and reaches every other fly as the flash travels; after it,
    /// each fly keeps a beat of its own until the next.
    fn sweep(&mut self, w: i64) -> &Vec<(f64, f64)> {
        if !self.hits.contains_key(&w) {
            let at: Vec<(f64, f64)> = self.flies.iter().map(|f| f.place(w as f64 * WAVE)).collect();
            let o = at.iter().copied().reduce(|b, p| if (w & 1 == 1 && p.0 > b.0) || (w & 1 == 0 && p.0 < b.0) { p } else { b }).unwrap_or((0.0, 0.0));
            let out = at
                .iter()
                .enumerate()
                .map(|(i, &(x, y))| (w as f64 * WAVE + (x - o.0).hypot(2.0 * (y - o.1)) / SPEED, 1.0 / (5.0 + 3.5 * hash(self.seed + i as i64, w))))
                .collect();
            if self.hits.len() > 4 {
                let oldest = *self.hits.keys().min().unwrap_or(&w);
                self.hits.remove(&oldest);
            }
            self.hits.insert(w, out);
        }
        &self.hits[&w]
    }

    fn bright(&mut self, i: usize, s: f64) -> f64 {
        let w = (s / WAVE).floor() as i64;
        let mut h = self.sweep(w)[i];
        if s < h.0 {
            h = self.sweep(w - 1)[i];
        }
        let ph = (s - h.0) * h.1;
        if ph < 0.0 {
            0.0
        } else {
            glow(ph - ph.floor())
        }
    }
}

impl Piece for Fireflies {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let w = self.dots.width() as i64;
        let s = WAVE * 3.0 + START + t;
        // Each fly's light, a halo falling off round it, with a faint trail
        // where it was brighter a moment ago. Only the meadow glows.
        self.field.fill(0.0);
        let (lo, hi) = ((HORIZON * 4.0) as i64, (GROUND * 4.0) as i64);
        for i in 0..self.flies.len() {
            let v = self.bright(i, s);
            let mut lights = vec![];
            if v > 0.05 {
                lights.push((self.flies[i].place(s), v, 1.0 + 3.2 * v));
            }
            for k in 1..=4 {
                let old = self.bright(i, s - f64::from(k) * 0.3);
                if old > v {
                    lights.push((self.flies[i].place(s - f64::from(k) * 0.3), 0.25 * old, 0.8));
                }
            }
            for ((x, y), v, r) in lights {
                let (cx, cy) = (x * 2.0, y * 4.0);
                for j in ((cy - r * 2.0).floor() as i64).max(lo)..=((cy + r * 2.0).ceil() as i64).min(hi - 1) {
                    for i in ((cx - r * 2.0).floor() as i64).max(0)..=((cx + r * 2.0).ceil() as i64).min(w - 1) {
                        let d = (i as f64 + 0.5 - cx).hypot(j as f64 + 0.5 - cy) / r;
                        let k = (j * w + i) as usize;
                        self.field[k] = self.field[k].max((v * (-d * d).exp()) as f32);
                    }
                }
            }
        }
        self.dots.clear();
        for &(x, y, color) in &self.sky {
            self.dots.set(x, y, color);
        }
        for &(x, y, p) in &self.stars {
            if p >= 0.25 || (t * 1.3 + p * 50.0).sin() > -0.3 {
                self.dots.set(x, y, STAR);
            }
        }
        // The glow's faint edge first, behind the grass; its bright heart after, in front.
        let mut bright = vec![];
        for j in lo..hi {
            for i in 0..w {
                let v = f64::from(self.field[(j * w + i) as usize]);
                if v > 0.07 && dither(i as usize, j as usize, v * 1.4) {
                    if v >= 0.34 {
                        bright.push((i, j, v));
                    } else {
                        self.dots.set(i, j, GLOW.at(v / 0.34 * 0.4));
                    }
                }
            }
        }
        for &(c, bh, lean, ph) in &self.blades {
            let sway = 0.6 * (s * 0.8 - ph).sin();
            let steps = (bh * 8.0) as i64;
            for k in 0..=steps {
                let f = k as f64 / steps as f64 * bh;
                let x = c + 0.5 + lean * f + sway * (f / bh).powi(2);
                self.dots.set((x * 2.0).floor() as i64, ((GROUND + 0.6 - f) * 4.0).floor() as i64, GRASS);
            }
        }
        for (i, j, v) in bright {
            self.dots.set(i, j, GLOW.at(v));
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
