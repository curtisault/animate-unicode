//! rocket: a night launch. Boil-off vents from the upper stage, the arm swings
//! back, the engine lights, exhaust billows out of the trench and the rocket
//! climbs out of the frame. It comes back down on a landing burn and settles.
//! After ascii.rest's rocket.ts (MIT, @bas3line).
//!
//! The timeline, the climb and landing, and every puff of exhaust are
//! ascii.rest's; the picture is redrawn in octants, eight dots a cell. The
//! rocket is its sprite's outline at dot resolution, a smooth nose and a
//! flared bell, shaded as a cylinder lit from the tower's side and moving a
//! dot at a time rather than a row. The flame is white at the nozzle through
//! yellow and orange to red at its tips, the smoke is grey and lit orange
//! where it rolls past the flame, the tower is a braced lattice with a red
//! beacon, and the stars twinkle.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};

/// The rocket's meta.
pub static META: Meta = Meta {
    name: "rocket",
    slug: "rocket",
    category: Category::Space,
    note: "an octant rocket lifting off at night, flame and smoke, then landing",
    cols: 48,
    rows: 28,
    fps: 20,
    charset: Charset::Extended,
    // The white paint, shadow to light; the interstage; the bell's metal; the
    // flame, tip to nozzle; smoke, thin to thick; smoke lit by the flame; the
    // tower; the beacon; the deck; a star; vapour.
    palette: Some(&[
        "#5c606a", "#9094a0", "#c8ccd4", "#f6f7fa", "#2c2e36", "#4c4e58", "#6e665c", "#a89c8a", "#c8301c", "#f0701e", "#ffb43a", "#fff0b0", "#3c3e46",
        "#5e6068", "#8a8c94", "#b4b6bc", "#d88a4a", "#7a4a3e", "#ff3a2e", "#4a4c52", "#8a8ea4", "#c8ccd8",
    ]),
    ground: Some("#05060c"),
    ..Meta::DEFAULT
};

const PAINT: Ramp = Ramp::new(0, 4);
const DARK: Ramp = Ramp::new(4, 2);
const METAL: Ramp = Ramp::new(6, 2);
const FLAME: Ramp = Ramp::new(8, 4);
const SMOKE: Ramp = Ramp::new(12, 4);
const LIT_SMOKE: u8 = 16;
const TOWER: u8 = 17;
const BEACON: u8 = 18;
const DECK: u8 = 19;
const STAR: u8 = 20;
const VAPOUR: u8 = 21;

/// The sprite's size in cells (14 wide, 21 tall), the loop, and its moments:
/// the arm starts to swing away, the engine lights, the clamps let go, rows
/// per second squared on the way up, rows above the pad where it is well out
/// of sight, touchdown, and braking on the landing burn, which starts out of
/// sight. Then exhaust puffs a second.
const W: f64 = 14.0;
const HT: f64 = 21.0;
const LOOP: f64 = 18.0;
const ARM: f64 = 0.8;
const IGNITE: f64 = 1.5;
const LIFT: f64 = 3.0;
const CLIMB: f64 = 2.6;
const OUT: f64 = 40.0;
const LAND: f64 = 14.6;
const DECEL: f64 = 4.4;
const RATE: f64 = 36.0;

fn burn() -> f64 {
    (2.0 * OUT / DECEL).sqrt()
}

/// Rows the rocket's base stands above the pad at time t in the loop, or
/// None while it is out of sight.
fn height(t: f64) -> Option<f64> {
    if !(LIFT..LAND).contains(&t) {
        return Some(0.0);
    }
    let up = 0.5 * CLIMB * (t - LIFT).powi(2);
    if up < OUT {
        return Some(up);
    }
    let left = LAND - t;
    (left < burn()).then_some(0.5 * DECEL * left * left)
}

fn burning(t: f64) -> bool {
    (IGNITE..LAND).contains(&t) && height(t).is_some()
}

fn on_pad(t: f64) -> bool {
    !(ARM - 0.3..LAND + 1.0).contains(&t)
}

/// What paints the rocket: white, the interstage, the bell, the fins.
#[derive(Clone, Copy, PartialEq)]
enum Paint {
    White,
    Interstage,
    Bell,
    Fin,
}

/// The rocket's outline at a height in its sprite (rows from the nose's tip):
/// the body's half width in columns, what paints it, and the fins' half width.
fn profile(y: f64) -> Option<(f64, Paint, f64)> {
    Some(match y {
        y if !(0.0..HT).contains(&y) => return None,
        y if y < 4.0 => (4.0 * (y / 4.0).powf(0.75), Paint::White, 0.0), // the nose, a little fuller than a cone
        y if y < 6.0 => (4.0, Paint::White, 0.0),
        y if y < 7.0 => (4.0 - (y - 6.0), Paint::White, 0.0), // the fairing's waist
        y if y < 9.0 => (3.0, Paint::White, 0.0),
        y if y < 10.0 => (3.0 + (y - 9.0), Paint::Interstage, 0.0),
        y if y < 15.0 => (4.0, Paint::White, 0.0),
        y if y < 18.0 => (4.0, Paint::White, 4.0 + (y - 15.0)),
        y if y < 19.0 => (3.0 - (y - 18.0), Paint::Bell, 0.0), // the throat
        y => (2.0 + (y - 19.0), Paint::Bell, 0.0),             // the bell
    })
}

/// One puff of exhaust: its sideways jitter, which way it fans out along the
/// deck and how far, how fast it rises, how long it lives, and how fast it swells.
struct Puff {
    jx: f64,
    side: f64,
    fan: f64,
    rise: f64,
    life: f64,
    grow: f64,
}

/// The launch: its stars (dot, phase, big), the puffs, the smoke field, and the dots.
pub struct Rocket {
    stars: Vec<(i64, i64, f64, bool)>,
    puffs: Vec<Puff>,
    fog: Vec<f32>,
    dots: ColorOctants,
}

/// A rocket; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as f64, META.rows as f64);
    let left = ((cols - W) / 2.0).floor();
    let pad = rows - 2.0;
    let tower = left - 8.0;
    let arm_row = pad - HT + 8.0;
    let mut rand = Mulberry32::new(3);
    // Stars, kept off the rocket, its vapour and the tower's top so none reads as a spark.
    let mut stars = vec![];
    for r in 0..(pad - 9.0) as i64 {
        for c in 0..cols as i64 {
            let (cf, rf) = (c as f64, r as f64);
            let by_rocket = cf >= left - 2.0 && cf <= left + W + 1.0 + if (rf - arm_row).abs() < 4.0 { 12.0 } else { 0.0 } && rf >= pad - HT - 2.0;
            let by_tower = (cf - tower - 1.0).abs() < 4.0 && rf >= pad - 24.0;
            if rand.next_f64() < 0.026 && !by_rocket && !by_tower {
                #[allow(clippy::approx_constant)] // ascii.rest's 6.283, so the twinkle is theirs
                let phase = rand.next_f64() * 6.283;
                stars.push((c * 2 + (c + r) % 2, r * 4 + 1 + (c * 7 + r) % 3, phase, rand.next_f64() < 0.2));
            }
        }
    }
    // Every puff of exhaust keeps its own spread, speed and life.
    let puffs = (0..(LOOP * RATE).ceil() as usize)
        .map(|_| Puff {
            jx: (rand.next_f64() - 0.5) * 2.4,
            side: if rand.next_f64() < 0.5 { -1.0 } else { 1.0 },
            fan: 14.0 + rand.next_f64() * 12.0,
            rise: 0.5 + rand.next_f64() * 0.9,
            life: 2.2 + rand.next_f64() * 1.4,
            grow: 0.6 + rand.next_f64() * 0.6,
        })
        .collect();
    let dots = ColorOctants::new(META.cols, META.rows);
    let fog = vec![0.0; dots.width() * dots.height()];
    Box::new(Rocket { stars, puffs, fog, dots })
}

impl Rocket {
    /// Adds a soft round puff of smoke centred at (x, y) in cells, rad columns across.
    fn splat(&mut self, x: f64, y: f64, rad: f64, s: f64) {
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        // A column is two dots and a row four, so a round puff is rad * 2 dots either way.
        let (cx, cy, rd) = (x * 2.0, y * 4.0, rad * 2.0);
        for j in ((cy - rd).ceil() as i64).max(0)..=((cy + rd).floor() as i64).min(h - 1) {
            for i in ((cx - rd).ceil() as i64).max(0)..=((cx + rd).floor() as i64).min(w - 1) {
                let q = ((i as f64 - cx) / rd).powi(2) + ((j as f64 - cy) / rd).powi(2);
                if q < 1.0 {
                    self.fog[(j * w + i) as usize] += (s * (1.0 - q)) as f32;
                }
            }
        }
    }

    /// A puff leaves the nozzle going down; where it meets the deck it fans out
    /// sideways, and either way it slows, swells, rises and thins.
    fn puff(&mut self, k: usize, x0: f64, y0: f64, age: f64, thick: f64, pad: f64) {
        let p = &self.puffs[k];
        let (jx, side, fan, rise, life, grow) = (p.jx, p.side, p.fan, p.rise, p.life, p.grow);
        let down = 26.0 * 0.2 * (1.0 - (-age / 0.2).exp());
        let (mut x, mut y) = (x0 + jx * (1.0 + age), y0 + down);
        if y > pad {
            let hit = -0.2 * (1.0 - (pad - y0) / (26.0 * 0.2)).ln();
            y = pad - 0.2;
            x = x0 + jx + side * fan * 0.7 * (1.0 - (-(age - hit) / 0.7).exp());
        }
        y -= rise * age * age * 0.5 + 0.3 * age;
        let fade = (1.0 - age / life).max(0.0);
        self.splat(x + 0.8 * age, y, 1.2 + grow * age * 2.2, 0.9 * thick * fade * fade);
    }
}

impl Piece for Rocket {
    fn frame(&mut self, time: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        let t = time.rem_euclid(LOOP);
        let left = ((cols - W) / 2.0).floor();
        let (cx, mid) = (left + W / 2.0, left + W / 2.0 - 0.5);
        let pad = rows - 2.0;
        let tower = left - 8.0;
        let arm_row = pad - HT + 8.0;
        self.dots.clear();
        self.fog.fill(0.0);
        let dots = &mut self.dots;
        for &(x, y, ph, big) in &self.stars {
            let tw = (time * 1.7 + ph).sin();
            if big || tw > -0.6 {
                dots.set(x, y, STAR);
                if big && tw > 0.3 {
                    // A bright star flares into a small cross.
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        dots.set(x + dx, y + dy, STAR);
                    }
                }
            }
        }

        // The tower: two rails braced corner to corner, a mast and its beacon.
        let (t0, t1) = ((tower * 2.0) as i64, (tower * 2.0) as i64 + 5);
        let (top, bottom) = (((pad - 20.0) * 4.0) as i64, (pad * 4.0) as i64 + 3);
        for y in top..=bottom {
            dots.set(t0, y, TOWER);
            dots.set(t1, y, TOWER);
            let k = (y - top).rem_euclid(8);
            // A diagonal brace every two rows, crossing the tower one way then the other.
            let across = if k < 4 { k } else { 7 - k };
            dots.set(t0 + 1 + across, y, TOWER);
        }
        for y in ((pad - 22.0) * 4.0) as i64 + 2..top {
            dots.set(t0 + 2, y, TOWER);
        }
        if (time * 1.25).floor() as i64 % 2 == 1 {
            let by = ((pad - 22.0) * 4.0) as i64;
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                dots.set(t0 + 2 + dx, by + dy, BEACON);
            }
        }
        // The service arm, swinging out to the upper stage and back.
        let swing = if t < ARM {
            1.0
        } else if t < IGNITE {
            1.0 - (t - ARM) / (IGNITE - ARM)
        } else if t < LAND + 1.0 {
            0.0
        } else {
            ((t - LAND - 1.0) / 0.6).min(1.0)
        };
        let reach = (swing * (left + 1.0 - tower - 3.0) * 2.0).round() as i64;
        for x in t1 + 1..=t1 + reach {
            dots.set(x, (arm_row * 4.0) as i64 + 1, TOWER);
            dots.set(x, (arm_row * 4.0) as i64 + 2, TOWER);
        }
        // The deck, with the flame trench under the engine.
        for x in 0..w {
            for y in h - 4..h - 2 {
                dots.set(x, y, DECK);
            }
            let c = (x / 2) as f64;
            if (2.0..cols - 2.0).contains(&c) && (c - mid).abs() >= 4.0 {
                dots.set(x, (pad * 4.0) as i64 + 3, DECK);
            }
        }

        // Exhaust, from every puff still alive. Births are on a fixed beat, so a
        // puff's age and where it was born follow from t alone.
        for k in 0..self.puffs.len() {
            let born = k as f64 / RATE;
            let age = t - born;
            if age < 0.0 || age > self.puffs[k].life || !burning(born) {
                continue;
            }
            let Some(hb) = height(born) else { continue };
            // Coming down it flies into its own plume, so only the last rows of the burn leave smoke.
            if pad - 1.0 - hb >= -2.0 {
                let thick = (-hb / if born > LAND - burn() { 4.0 } else { 9.0 }).exp();
                self.puff(k, cx, pad - 0.5 - hb, age, thick, pad);
            }
        }
        let now = height(t);
        let firing = burning(t);
        let base = now.map(|hn| pad - hn); // the rocket's bottom edge, in rows
        let dots = &mut self.dots;
        // Smoke is how much of it there is, not light, so it keeps its sense on
        // paper, and it stops short of solid so the clouds stay soft.
        for (k, &f) in self.fog.iter().enumerate() {
            if f <= 0.06 {
                continue;
            }
            let (x, y) = (k as i64 % w, k as i64 / w);
            let v = 1.0 - (-f64::from(f) / 0.9).exp();
            if dither(x as usize, y as usize, 0.12 + 0.7 * v) {
                let lit = firing && base.is_some_and(|b| (y as f64 / 4.0 - b).abs() < 5.0 && (x as f64 / 2.0 - cx).abs() < 9.0);
                dots.set(x, y, if lit && v > 0.3 { LIT_SMOKE } else { SMOKE.at(v) });
            }
        }
        // The two stands the fins rest on.
        for y in ((pad - 3.0) * 4.0) as i64..(pad * 4.0) as i64 {
            dots.set((left * 2.0) as i64, y, TOWER);
            dots.set(((left + W) * 2.0) as i64 - 1, y, TOWER);
        }

        if let Some(base) = base {
            let top = base - HT;
            for y in (top * 4.0).floor() as i64..(base * 4.0).ceil() as i64 {
                let Some((hw, paint, fin)) = profile((y as f64 + 0.5) / 4.0 - top) else { continue };
                for x in ((cx - fin.max(hw)) * 2.0).floor() as i64..((cx + fin.max(hw)) * 2.0).ceil() as i64 {
                    let u = (x as f64 + 0.5) / 2.0 - cx;
                    let (paint, lit) = if u.abs() < hw {
                        // A cylinder lit from the left, where the tower's floodlights are.
                        let s = u / hw;
                        (paint, (-0.72 * s + 0.69 * (1.0 - s * s).max(0.0).sqrt()).max(0.0))
                    } else if u.abs() < fin {
                        (Paint::Fin, if u < 0.0 { 0.75 } else { 0.3 }) // flat, so each fin takes one tone
                    } else {
                        continue;
                    };
                    let tone = match paint {
                        Paint::White => 1.0,
                        Paint::Interstage => 0.42,
                        Paint::Bell => 0.62,
                        Paint::Fin => 0.6,
                    };
                    let light = tone * (0.12 + 0.88 * lit);
                    let ink = if env.paper { 1.0 - 0.6 * light } else { 0.45 + 0.55 * light };
                    if dither(x as usize, y as usize, ink) {
                        let color = match paint {
                            Paint::White => PAINT.at(0.15 + 0.85 * lit),
                            Paint::Fin => PAINT.at(lit * 0.6),
                            Paint::Interstage => DARK.at(lit),
                            Paint::Bell => METAL.at(lit),
                        };
                        dots.set(x, y, color);
                    }
                }
            }
            // While it waits on the pad, liquid oxygen boils off through a vent
            // on the upper stage in a thin wisp that drifts away on the wind.
            for k in (0..=23).rev() {
                let born = ((time / 0.1).floor() - f64::from(k)) * 0.1;
                let age = time - born;
                if !on_pad(born.rem_euclid(LOOP)) || age > 2.4 {
                    continue;
                }
                let f = age / 2.4;
                let gust = 1.0 + 0.3 * (0.9 * born).sin();
                let y = top + 8.5 - 0.4 * age - 0.2 * age * age + 1.1 * (2.3 * born).sin() * f;
                let x = left + W - 3.5 + gust * (3.0 * age + 0.6 * age * age);
                let len = if f < 0.2 {
                    3
                } else if f < 0.45 {
                    2
                } else {
                    1
                };
                for d in 0..len {
                    dots.set((x * 2.0) as i64 + d, (y * 4.0) as i64, VAPOUR);
                }
            }
            if firing {
                // Short at ignition, long in the climb, shorter braking.
                let len = if t < LIFT {
                    1.0 + 2.5 * (t - IGNITE) / (LIFT - IGNITE)
                } else if t < LAND - burn() {
                    6.0
                } else {
                    4.5
                };
                let f = (time * 20.0).floor() as i64;
                let reach = len + (f * 7).rem_euclid(3) as f64 - 1.0;
                for y in (base * 4.0).ceil() as i64..((base + reach) * 4.0).ceil() as i64 {
                    let k = (y as f64 + 0.5) / 4.0 - base;
                    let heat = 1.0 - k / (len + 1.0);
                    let half = 0.4 + 2.8 * heat.max(0.0).powf(0.6);
                    for x in ((cx - half) * 2.0).floor() as i64..((cx + half) * 2.0).ceil() as i64 {
                        let edge = ((x as f64 + 0.5) / 2.0 - cx).abs() / (half + 0.5);
                        let flicker = ((f + x * 3 + y) * 13).rem_euclid(5) as f64 * 0.03;
                        let v = heat * 0.8 + 0.35 - edge * 0.5 + flicker;
                        if v > 0.05 && dither(x as usize, y as usize, 0.3 + v) {
                            dots.set(x, y, FLAME.at(v.min(1.0)));
                        }
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
