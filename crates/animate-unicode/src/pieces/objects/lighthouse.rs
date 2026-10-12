//! lighthouse: a banded lighthouse on a heap of rocks at night. Its beam
//! turns round the lantern, long when it crosses the frame and a flash when
//! it faces us, lighting the haze and the waves beneath; surf bursts on the
//! rocks. After ascii.rest's lighthouse.ts (MIT, @bas3line).
//!
//! The tower, the rocks, the turning beam, the sea and the surf are
//! ascii.rest's. The beam, the sea and the spray are braille: the beam a
//! cone of dots dithered by its light in the haze, the crests fine lines,
//! the spray single drops. The tower and its rocks are octants, moonlit from
//! the left, each boulder washed by the swell at its own waterline. In
//! colour the tower is banded red and white with a dark gallery and dome,
//! the lantern and its beam are a warm gold, the sea is dark blue lit gold
//! under the beam, the rocks are grey and the surf white.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The lighthouse's meta.
pub static META: Meta = Meta {
    name: "lighthouse",
    slug: "lighthouse",
    category: Category::Objects,
    note: "a red and white lighthouse sweeping a gold braille beam over the sea",
    cols: 64,
    rows: 30,
    fps: 20,
    charset: Charset::Extended,
    // The beam, faint to bright; red bands, white bands; the gallery and
    // dome; the lantern; rocks; the sea, and the sea lit; surf; a star.
    palette: Some(&[
        "#4a4630", "#a89a60", "#f0dc96", "#fff6d4", "#c43a3a", "#eeeef0", "#2a2c34", "#ffe27a", "#6a6a6e", "#1e3c5a", "#c8b46e", "#e8f2ff", "#8a90a8",
    ]),
    ground: Some("#05070d"),
    ..Meta::DEFAULT
};

const BEAM: Ramp = Ramp::new(0, 4);
const RED: u8 = 4;
const WHITE: u8 = 5;
const DARK: u8 = 6;
const LANTERN: u8 = 7;
const ROCK: u8 = 8;
const SEA: u8 = 9;
const SEA_LIT: u8 = 10;
const SURF: u8 = 11;
const STAR: u8 = 12;
/// The tower's column, the lamp's row, the horizon, seconds a turn, the
/// shaft's rows, its half width at the top and the foot in rows, and rows to a band.
const TC: f64 = 32.0;
const LAMP: f64 = 5.0;
const HORIZON: f64 = 19.0;
const PERIOD: f64 = 8.0;
const TOP: f64 = 9.0;
const FOOT: f64 = 24.0;
const WT: f64 = 2.1;
const WB: f64 = 3.1;
const BAND: f64 = 3.0;
/// Boulders heaped round the foot, back to front: x in rows from the tower,
/// y, half width, half height, waterline. The first two stand behind it.
const ROCKS: [(f64, f64, f64, f64, f64); 8] = [
    (-3.9, 21.1, 2.3, 1.7, 99.0),
    (3.9, 20.9, 2.4, 1.8, 99.0),
    (-6.6, 22.9, 2.3, 1.6, 23.9),
    (6.7, 22.7, 2.2, 1.5, 23.6),
    (-1.7, 23.5, 2.7, 1.8, 24.8),
    (3.1, 23.8, 2.5, 1.7, 25.1),
    (-9.3, 24.5, 1.5, 0.9, 25.0),
    (9.2, 24.2, 1.3, 0.8, 24.7),
];
const BEHIND: usize = 2;
/// Where surf strikes: the boulder, the side it breaks on (0 for foam
/// alone), and when in the swell; seconds between breakers.
const SURFS: [(usize, f64, f64); 6] = [(6, -1.0, 0.0), (7, 1.0, 0.45), (3, 1.0, 0.8), (2, -1.0, 0.3), (4, 0.0, 0.62), (5, 0.0, 0.15)];
const SWELL: f64 = 3.2;

fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

/// Lit from the upper left, by the moon.
fn moon(u: f64, v: f64, nz: f64) -> f64 {
    (-0.5 * u - 0.45 * v + 0.75 * nz).max(0.0)
}

/// The beam in the haze at (dx, dy) rows from the lamp: a cone seen side on,
/// so the more it turns toward or away from us the shorter, wider and
/// brighter it looks.
fn beam_at(dx: f64, dy: f64, c: f64, s: f64) -> f64 {
    let fore = c.abs().max(0.1);
    let (ax, ay) = (if c < 0.0 { -1.0f64 } else { 1.0 }, (0.02 / fore).min(0.35));
    let m = ax.hypot(ay);
    let a = (dx * ax + dy * ay) / m;
    if a <= 0.0 {
        return 0.0;
    }
    let across = (dx * ay - dy * ax).abs() / m;
    let half = 0.5 + a * (0.11 / fore);
    (-(across / half).powi(4)).exp() * (-a / (22.0 * fore)).exp() * (1.0 / fore.powf(0.3)).min(1.6) * if s < 0.0 { 0.75 } else { 1.0 }
}

/// The lighthouse: the tower and rocks by dot (light, paint, boulder), the
/// stars, the surf's places, and the two canvases.
pub struct Lighthouse {
    still: Vec<Option<(f64, u8, i8)>>,
    stars: Vec<(i64, i64)>,
    surf: Vec<(f64, f64, f64, f64)>,
    haze: ColorDots,
    tower: ColorOctants,
}

/// A lighthouse; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let tower = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (tower.width(), tower.height());
    let mut still = vec![None; w * h];
    for j in 0..h {
        for i in 0..w {
            let k = j * w + i;
            let (x, y) = (((i as f64 + 0.5) / 2.0 - 0.5 - TC) / 2.0, (j as f64 + 0.5) / 4.0);
            // The rocks behind the tower first, then the tower, then the rocks before it.
            let boulder = |n: usize, still: &mut Vec<Option<(f64, u8, i8)>>| {
                let (bx, by, rx, ry, wl) = ROCKS[n];
                let (u, v) = ((x - bx) / rx, (y - by) / ry);
                let q = u * u + v * v;
                if q < 1.0 && y <= wl + 0.5 {
                    let nz = (1.0 - q).sqrt();
                    // A dark rim on each, so heaped boulders keep apart; a little grain.
                    let l = (0.06 + 0.86 * moon(u, v, nz)) * (0.15 + nz * 1.8).min(1.0) + 0.06 * (hash(i as i64, j as i64) - 0.5);
                    still[k] = Some((l, ROCK, n as i8));
                }
            };
            for n in 0..BEHIND {
                boulder(n, &mut still);
            }
            // The shaft: banded, round, a door at its foot and a slit of a light in a band.
            if (TOP..=FOOT + 1.0).contains(&y) {
                let wd = WT + (WB - WT) * (y - TOP) / (FOOT - TOP);
                if x.abs() <= wd {
                    let u = x / (wd + 0.25);
                    let nz = (1.0 - u * u).sqrt();
                    let red = ((y - TOP) / BAND).floor() as i64 % 2 == 1;
                    // The red bands darker than the white, so they show in one ink too.
                    let mut l = if red { 0.3 } else { 0.92 } * (0.25 + 0.75 * moon(u, 0.0, nz));
                    let mut paint = if red { RED } else { WHITE };
                    if (y >= FOOT - 2.5 && x.abs() < 0.6) || ((y - TOP - BAND - 1.5).abs() < 0.4 && x.abs() < 0.3) {
                        (l, paint) = (0.0, DARK);
                    }
                    still[k] = Some((l, paint, -1));
                }
            }
            // The gallery: a rail of posts over a deck in shadow.
            if (TOP - 2.0..TOP - 1.0).contains(&y) && x.abs() <= 2.9 && (x.abs() > 2.6 || (i / 2) % 2 == 0 || y > TOP - 1.3) {
                still[k] = Some((0.5, DARK, -1));
            }
            if (TOP - 1.0..TOP).contains(&y) && x.abs() <= 3.0 {
                still[k] = Some((0.4, DARK, -1));
            }
            // The roof: a dome with a vent on top.
            let dome = (x / 1.85).powi(2) + ((y - (LAMP - 1.6)) / 1.5).powi(2);
            if dome < 1.0 && y < LAMP - 1.4 {
                still[k] = Some((0.4 + 0.5 * moon(x / 1.85, -0.4, (1.0 - dome).sqrt()), RED, -1));
            }
            if x.abs() < 0.25 && (LAMP - 3.6..LAMP - 2.9).contains(&y) {
                still[k] = Some((0.5, DARK, -1));
            }
            for n in BEHIND..ROCKS.len() {
                boulder(n, &mut still);
            }
        }
    }
    // Stars, a few and faint.
    let mut stars = vec![];
    for n in 0..24 {
        let (c, r) = ((hash(n, 1) * META.cols as f64) as i64, (hash(n, 2) * (HORIZON - 2.0)) as i64);
        if (c - TC as i64).abs() > 4 {
            stars.push((c * 2, r * 4 + 1));
        }
    }
    // The surf: where each breaker strikes, at the waterline on its boulder's outer flank.
    let surf = SURFS.iter().map(|&(n, side, ph)| (TC + 0.5 + 2.0 * (ROCKS[n].0 + ROCKS[n].2 * side * 0.95), ROCKS[n].4 - 0.2, side, ph)).collect();
    Box::new(Lighthouse { still, stars, surf, haze: ColorDots::new(META.cols, META.rows), tower })
}

impl Piece for Lighthouse {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let rows = META.rows as f64;
        let (w, h) = (self.tower.width(), self.tower.height());
        let th = -0.35 + TAU * t / PERIOD;
        let (s, c) = th.sin_cos();
        let face = s.max(0.0).powi(10); // the lens turned square to us
        self.haze.clear();
        for &(x, y) in &self.stars {
            self.haze.set(x, y, STAR);
        }
        // The beam and the glare round the lamp, in the haze; the sea below,
        // its crests lit where the beam falls.
        let mut light = vec![0.0; w];
        for j in 0..h {
            let y = (j as f64 + 0.5) / 4.0;
            #[allow(clippy::needless_range_loop)] // the index reads several arrays at once
            for i in 0..w {
                let dx = ((i as f64 + 0.5) / 2.0 - 0.5 - TC) / 2.0;
                let dy = y - 0.5 - LAMP;
                let glare = (-(dx.hypot(dy) / (1.2 + 2.2 * face)).powi(2)).exp() * (0.5 + 1.2 * face);
                if y < HORIZON {
                    let b = beam_at(dx, dy, c, s) + glare;
                    if (y - LAMP - 0.5).abs() < 0.13 {
                        light[i] = b;
                    }
                    if b > 0.1 && dither(i, j, b * 0.85) {
                        self.haze.set(i as i64, j as i64, BEAM.at(b / 1.4));
                    }
                    continue;
                }
                let d = y - HORIZON + 1.0;
                let x = (i as f64 / 2.0) / (3.2 / d + 0.6);
                let wave = (x * 0.9 + d * 1.7 - t * 2.2 + (x * 0.31 + d).sin() * 1.5).sin() + 0.6 * (x * 0.43 - d * 0.9 + t * 1.3).sin();
                let lit = (light[i] * 0.9 + glare * 0.5).min(1.0) * (-(d - 1.0) / 6.0).exp();
                // A crest is the top dot of a wave's rise, so the sea is a field of fine lines.
                let crest = wave > 0.9 - 0.5 * lit && (j % 2 == 0 || lit > 0.3);
                if crest {
                    self.haze.set(i as i64, j as i64, if lit > 0.2 { SEA_LIT } else { SEA });
                }
            }
        }
        // Surf: a breaker bursts on a boulder, its spray thrown up in a fan that falls back as drops.
        for (n, &(x0, y0, side, ph)) in self.surf.iter().enumerate() {
            let tt = (t / SWELL + ph).rem_euclid(1.0) * SWELL;
            if tt > 1.6 {
                continue;
            }
            if tt < 1.3 {
                let spread = 1.0 + tt * 3.0;
                let mut j = -spread;
                while j <= spread {
                    self.haze.set(((x0 + j) * 2.0) as i64, ((y0 + 0.7) * 4.0) as i64, SURF);
                    j += 0.5;
                }
            }
            if side == 0.0 {
                continue;
            }
            for k in 0..18 {
                let f = f64::from(k) / 17.0;
                let (h1, h2) = (hash(k.into(), n as i64 + 7), hash(k.into(), n as i64 + 19));
                let (vx, vy) = (side * (1.0 + 13.0 * f * f) + (h1 - 0.5) * 3.0, (11.0 - 6.0 * f) * (0.8 + 0.4 * h2));
                let (x, y) = (x0 + vx * tt, y0 - vy * tt + 9.0 * tt * tt);
                if y < y0 + 0.3 && y > 0.0 && y < rows {
                    self.haze.set((x * 2.0) as i64, (y * 4.0) as i64, SURF);
                }
            }
        }
        // The lighthouse and rocks over the rest, each boulder washed by the
        // swell at its own waterline.
        let swell: Vec<f64> =
            ROCKS.iter().enumerate().map(|(n, b)| b.4 + 0.45 * (TAU * t / SWELL + n as f64 * 1.9).sin() + 0.25 * (t * 1.3 + n as f64).sin()).collect();
        self.tower.clear();
        for (k, s) in self.still.iter().enumerate() {
            let Some((l, paint, rock)) = *s else { continue };
            let (i, j) = (k % w, k / w);
            if rock >= 0 && (j as f64 + 0.5) / 4.0 > swell[rock as usize] {
                continue;
            }
            let ink = if env.paper { 1.05 - l } else { 0.25 + 0.8 * l };
            if paint == DARK || dither(i, j, ink) {
                self.tower.set(i as i64, j as i64, paint);
            }
        }
        // The lantern: glass glowing from within between its bars.
        let glow = 0.75 + 0.25 * face;
        for j in ((LAMP - 1.0) * 4.0) as i64..((LAMP + 2.0) * 4.0) as i64 {
            for i in ((TC - 3.0) * 2.0) as i64..((TC + 4.0) * 2.0) as i64 {
                let x = ((i as f64 + 0.5) / 2.0 - 0.5 - TC) / 2.0;
                let bar = (x.abs() - 1.0).abs() < 0.15 || x.abs() > 1.6;
                if x.abs() <= 1.75 && (bar || dither(i as usize, j as usize, glow)) {
                    self.tower.set(i, j, if bar { DARK } else { LANTERN });
                }
            }
        }
        grid.clear();
        self.haze.draw(grid, 0, 0);
        self.tower.draw(grid, 0, 0);
        // The lamp, and the rays when it faces us.
        let (lc, lr) = (TC as i64, LAMP as i64);
        grid.set(
            lc,
            lr,
            if face > 0.3 {
                '●'
            } else if s > -0.2 {
                '•'
            } else {
                '·'
            },
            BEAM.at(1.0),
        );
        if face > 0.5 {
            grid.set(lc, lr - 1, '│', BEAM.at(1.0));
            grid.set(lc, lr + 1, '│', BEAM.at(1.0));
        }
    }
}
