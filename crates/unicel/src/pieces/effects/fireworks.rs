//! fireworks: shells climb on a sparkling trail and burst. Every spark
//! follows a closed-form path under air drag and gravity, so any moment can
//! be drawn straight from the clock, and each shell is one of several kinds
//! of burst. After ascii.rest's fireworks.ts (MIT, @bas3line).
//!
//! The shells, their bursts and every spark's flight are ascii.rest's,
//! drawn in braille: a spark's head is a dot and its trail a fine curve of
//! dots back along its own path, so the peonies open round, the willows hang
//! in threads and the palms droop in long comets. In colour each shell takes
//! its own hue, red, green, blue, gold, purple or white, its heads bright and
//! its trails cooling to an ember; the willows fall gold.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::collections::HashMap;
use std::f64::consts::FRAC_PI_2;

/// The fireworks' meta.
pub static META: Meta = Meta {
    name: "fireworks",
    slug: "fireworks",
    category: Category::Effects,
    note: "braille shells bursting as peonies, willows and palms, each its hue",
    cols: 64,
    rows: 24,
    fps: 30,
    charset: Charset::Extended,
    // Six hues, each bright, cooling and an ember: red, green, blue, gold,
    // purple, white.
    palette: Some(&[
        "#ff5a5a", "#c82a3a", "#5a1418", "#6aff8a", "#2aa84a", "#14401e", "#7ab0ff", "#3a5ac8", "#18224a", "#ffd25a", "#e08a2a", "#5a3410", "#d88aff",
        "#8a3ad0", "#341650", "#ffffff", "#c8c8d8", "#4a4a58",
    ]),
    ground: Some("#04050b"),
    ..Meta::DEFAULT
};

const GOLD: usize = 3;
/// Gravity in column widths a second squared (a row is two), and seconds between launches.
const G: f64 = 13.0;
const EVERY: f64 = 1.25;
/// Shells take their kind and their spot on the skyline in turn.
const KINDS: [Kind; 9] = [Kind::Peony, Kind::Willow, Kind::Ring, Kind::Palm, Kind::Saturn, Kind::Peony, Kind::Willow, Kind::Ring, Kind::Palm];
const SPOTS: [f64; 7] = [0.3, 0.68, 0.45, 0.25, 0.62, 0.36, 0.72];
/// Play opens this long after the first shell, a peony, bursts.
const OPEN: f64 = 0.55;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Peony,
    Willow,
    Ring,
    Palm,
    Saturn,
}

/// How a spark is drawn.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    Peony,
    Willow,
    Ring,
    Core,
    Palm,
}

/// A spark: its throw, its drag, how long it lives, its own seed, how it
/// looks, and whether it was thrown nearer the eye.
struct Spark {
    vx: f64,
    vy: f64,
    k: f64,
    life: f64,
    seed: u32,
    look: Look,
    inner: bool,
}

/// A shell: when it launches, where, its drift, how long it climbs, how high
/// it bursts, its hue, and its sparks.
struct Shell {
    at: f64,
    x0: f64,
    drift: f64,
    rise: f64,
    top: f64,
    hue: usize,
    sparks: Vec<Spark>,
}

/// Adds a spark to a shell: throw across and up, look, drag, life, and whether it is inner.
type Add<'a> = &'a mut dyn FnMut(&mut Mulberry32, f64, f64, Look, f64, f64, bool);

/// One shell, everything about it drawn from its own seed.
fn shell(i: i64, cols: f64, h: f64) -> Shell {
    let mut r = Mulberry32::new((i * 7919 + 17) as u32);
    let pick = |n: usize| i.rem_euclid(n as i64) as usize;
    let kind = KINDS[pick(KINDS.len())];
    let mut s = Shell {
        at: i as f64 * EVERY + r.next_f64() * 0.25,
        x0: cols * (SPOTS[pick(SPOTS.len())] + (r.next_f64() - 0.5) * 0.05),
        drift: (r.next_f64() - 0.5) * 2.0,
        rise: 1.0 + r.next_f64() * 0.2,
        top: h * (0.62 + r.next_f64() * 0.08),
        hue: if kind == Kind::Willow { GOLD } else { (i * 5).rem_euclid(6) as usize },
        sparks: vec![],
    };
    let mut add = |r: &mut Mulberry32, vx: f64, vy: f64, look: Look, k: f64, life: f64, inner: bool| {
        s.sparks.push(Spark { vx, vy, k, life, seed: (r.next_f64() * 1e9) as u32, look, inner });
    };
    #[allow(clippy::approx_constant)] // ascii.rest's 6.283, so the bursts are theirs
    let tau = 6.283;
    // A sphere seen from outside: headings evenly round the sky, and most
    // sparks out at the rim. With `deep`, every third one is thrown nearer the
    // eye and shows further in.
    let sphere = |r: &mut Mulberry32, add: Add<'_>, count: usize, speed: f64, look: Look, k: f64, life: f64, deep: bool| {
        let turn = r.next_f64() * tau;
        for j in 0..count {
            let a = turn + (j as f64 + 0.5) / count as f64 * tau;
            let inner = deep && j % 3 == 1;
            let reach = if inner { 0.5 + 0.25 * (((j * 7) % 5) as f64 / 4.0) } else { 0.94 + r.next_f64() * 0.06 };
            add(r, a.cos() * speed * reach, a.sin() * speed * reach, look, k, life, inner);
        }
    };
    // A flat ring tipped toward the viewer, so it shows as an ellipse.
    let ring = |r: &mut Mulberry32, add: Add<'_>, count: usize, speed: f64, open: f64| {
        let (tilt, roll) = (open + r.next_f64() * 0.15, (r.next_f64() - 0.5) * 0.5);
        for j in 0..count {
            let a = j as f64 / count as f64 * tau;
            let (x, y) = (a.cos(), a.sin() * tilt);
            add(r, (x * roll.cos() - y * roll.sin()) * speed, (x * roll.sin() + y * roll.cos()) * speed, Look::Ring, 2.2, 1.5, false);
        }
    };
    match kind {
        Kind::Peony => sphere(&mut r, &mut add, 27, 33.0, Look::Peony, 2.2, 1.7, true),
        Kind::Willow => sphere(&mut r, &mut add, 15, 30.0, Look::Willow, 2.1, 3.4, false),
        Kind::Ring => ring(&mut r, &mut add, 24, 28.0, 0.3),
        Kind::Saturn => {
            ring(&mut r, &mut add, 24, 30.0, 0.5);
            sphere(&mut r, &mut add, 9, 10.0, Look::Core, 2.2, 1.1, true);
        }
        Kind::Palm => {
            // A palm: a few heavy comets fanned up and out, drooping as they slow.
            for j in 0..7 {
                let a = FRAC_PI_2 + (j as f64 - 3.0) / 3.0 * 1.15 + (r.next_f64() - 0.5) * 0.08;
                add(&mut r, a.cos() * 36.0, a.sin() * 36.0, Look::Palm, 2.2, 1.9, false);
            }
        }
    }
    s
}

/// Where a spark is b seconds after the burst, in column widths up from the
/// bottom. Drag spends its throw exponentially, so its reach so far is
/// (1 - e^-kb) / k of its speed.
fn at(s: &Shell, p: &Spark, b: f64) -> (f64, f64) {
    let e = (1.0 - (-p.k * b).exp()) / p.k;
    (s.x0 + s.drift * s.rise + p.vx * e, s.top + p.vy * e - G / p.k * (b - e))
}

/// The age at which a spark has gone a share q of the way out it has gone by age a.
fn back(p: &Spark, a: f64, q: f64) -> f64 {
    -(1.0 - q * (1.0 - (-p.k * a).exp())).ln() / p.k
}

/// The fireworks: the shells in the sky, and the dots.
pub struct Fireworks {
    cache: HashMap<i64, Shell>,
    dots: ColorDots,
}

/// Fireworks; they take no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Fireworks { cache: HashMap::new(), dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Fireworks {
    fn frame(&mut self, time: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let sky = rows * 2.0; // sky height in column widths
        let first = shell(0, cols, sky);
        let t = time + first.at + first.rise + OPEN;
        let tick = (t * 30.0).floor() as i64;
        let last = (t / EVERY).floor() as i64;
        // In dots: two a column across, and a column width up is two dots.
        let dot = |(x, y): (f64, f64)| ((x * 2.0).floor() as i64, ((rows - y / 2.0) * 4.0).floor() as i64);
        self.dots.clear();
        for i in last - 4..=last {
            let s = self.cache.entry(i).or_insert_with(|| shell(i, cols, sky));
            let age = t - s.at;
            if age < 0.0 {
                continue;
            }
            let hue = s.hue as u8 * 3;
            if age < s.rise {
                // Climbing: slowing toward the top, with sparks dropping off behind.
                let g = 2.0 * s.top / (s.rise * s.rise);
                let v0 = g * s.rise;
                let pos = |a: f64| (s.x0 + s.drift * a, v0 * a - g * a * a / 2.0);
                for j in 1..=8 {
                    let a = age - f64::from(j) * 0.03;
                    if a < 0.0 || (tick + i64::from(j) + i) % 3 == 0 {
                        continue;
                    }
                    let (x, y) = pos(a);
                    let (dx, dy) = dot((x + ((tick * 7 + i64::from(j)) % 3 - 1) as f64 * 0.3, y - f64::from(j) * 0.2));
                    self.dots.set(dx, dy, GOLD as u8 * 3 + 1);
                }
                let (dx, dy) = dot(pos(age));
                self.dots.set(dx, dy, GOLD as u8 * 3);
                self.dots.set(dx, dy + 1, GOLD as u8 * 3);
                continue;
            }
            let a = age - s.rise;
            // The flash at the moment of the burst.
            if a < 0.12 {
                let (fx, fy) = dot((s.x0 + s.drift * s.rise, s.top));
                for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1), (-2, 0), (2, 0)] {
                    self.dots.set(fx + dx, fy + dy, 15);
                }
            }
            for p in &s.sparks {
                if a > p.life || a < 0.1 {
                    continue;
                }
                let f = a / p.life; // how spent the spark is
                let mut hh = (p.seed ^ (tick as u32).wrapping_mul(0x9e37_79b1)).wrapping_mul(0x85eb_ca6b);
                hh ^= hh >> 13;
                // Near the end a spark flickers before it goes out.
                if f > 0.7 && f64::from(hh % 1000) / 1000.0 < (f - 0.7) * 3.0 {
                    continue;
                }
                // Its trail: a curve back along its own path, cooling as it goes back.
                let from = match p.look {
                    Look::Peony if !p.inner && a < 0.75 => Some(back(p, a, 0.6).max(a - 0.12)),
                    Look::Willow => Some(back(p, a, 0.45).max(a - 1.6)),
                    Look::Palm => Some(back(p, a, 0.45).max(a - 1.0)),
                    _ => None,
                };
                if let Some(b0) = from {
                    let n = 40;
                    for j in 1..=n {
                        let u = f64::from(j) / f64::from(n);
                        let (dx, dy) = dot(at(s, p, a - (a - b0) * u));
                        // A willow's thread breaks into dots toward its top.
                        if p.look == Look::Willow && u > 0.6 && j % 2 == 0 {
                            continue;
                        }
                        self.dots.set(dx, dy, hue + if u < 0.4 { 1 } else { 2 });
                    }
                }
                let (dx, dy) = dot(at(s, p, a));
                let bright = match p.look {
                    Look::Ring => a > 0.15,
                    Look::Palm => true,
                    _ => f < 0.6,
                };
                if p.look == Look::Ring && a < 0.15 {
                    continue;
                }
                self.dots.set(dx, dy, hue + u8::from(!bright));
                // A heavy head early on: a palm's comet, a fresh peony star.
                if (p.look == Look::Palm && a < 0.45) || (p.look == Look::Peony && a < 0.7) {
                    self.dots.set(dx + 1, dy, hue);
                    self.dots.set(dx, dy + 1, hue);
                }
            }
        }
        self.cache.retain(|&i, _| i >= last - 5);
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
