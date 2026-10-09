//! aquarium: a fish tank. Fish of a few kinds swim its length, drift between
//! depths and turn about, bubbles stream up from a stone, and weed sways.
//! After ascii.rest's aquarium.ts (MIT, @bas3line).
//!
//! The tank, the fishes' courses and turns, the weed and the bubbles are
//! ascii.rest's; everything in the water is redrawn in octants. Each fish is
//! shaped dot by dot, a body, a fanned tail and an eye, a tall-finned angel
//! among them, mirrored when it swims back and seen face on for a moment as
//! it turns. Each kind has its own colour: neon tetras blue, a long fish
//! orange-red, a bottom feeder yellow, the angel silver, a goldfish orange.
//! The weed sways in greens, the bubbles swell as they rise, the gravel is
//! mixed stone colours.

use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The aquarium's meta.
pub static META: Meta = Meta {
    name: "aquarium",
    slug: "aquarium",
    category: Category::Creatures,
    note: "octant fish of five colours turning in a tank, weed and bubbles",
    cols: 60,
    rows: 20,
    fps: 15,
    charset: Charset::Extended,
    // Fish: tetra, long fish, bottom feeder, angel, goldfish. Weed dark and
    // light; bubbles; the surface; gravel light and dark; the stone; the glass.
    palette: Some(&[
        "#46c4ff", "#ff6a3a", "#f2d040", "#e8e4d4", "#ffa22c", "#2c7a3c", "#5cb44a", "#c4ecff", "#78c8f0", "#b49a72", "#6e6050", "#7a7c84", "#5a7a98",
    ]),
    ground: Some("#06101a"),
    ..Meta::DEFAULT
};

const WEED: [u8; 2] = [5, 6];
const BUBBLE: u8 = 7;
const SURFACE_LINE: u8 = 8;
const GRAVEL: [u8; 2] = [9, 10];
const STONE: u8 = 11;
const GLASS: u8 = 12;

/// Seconds a loop; every period below divides it. The surface's row and the floor's.
const LOOP: f64 = 48.0;
const SURFACE: f64 = 1.0;
const FLOOR: f64 = 17.0;

/// A kind of fish: length in columns, height in rows, how much of it is
/// tail, fin height above the body as a share of it, and colour.
#[derive(Clone, Copy)]
struct Kind {
    len: f64,
    tall: f64,
    tail: f64,
    fin: f64,
    color: u8,
}

const TETRA: Kind = Kind { len: 3.2, tall: 0.7, tail: 0.3, fin: 0.0, color: 0 };
const LONG: Kind = Kind { len: 7.0, tall: 0.9, tail: 0.18, fin: 0.0, color: 1 };
const BOTTOM: Kind = Kind { len: 5.0, tall: 0.8, tail: 0.22, fin: 0.0, color: 2 };
const ANGEL: Kind = Kind { len: 6.0, tall: 1.4, tail: 0.25, fin: 0.9, color: 3 };
const GOLD: Kind = Kind { len: 9.0, tall: 2.4, tail: 0.25, fin: 0.25, color: 4 };

/// A fish's course.
type Swim = (Kind, f64, f64, f64, f64, f64, f64, f64);
/// Each fish: its kind, the row it swims at, from and to which columns,
/// seconds there and back, its phase, rows it drifts up and down over the
/// loop, and the drift's phase.
const FISH: [Swim; 7] = [
    (TETRA, 5.2, 6.0, 54.0, 12.0, 0.8, 1.5, 1.0),
    (TETRA, 6.6, 3.0, 51.0, 12.0, 0.775, 1.5, 1.1),
    (TETRA, 8.0, 8.0, 56.0, 12.0, 0.79, 1.5, 0.9),
    (LONG, 3.3, 3.0, 50.0, 24.0, 0.1, 1.2, 1.0),
    (BOTTOM, 15.6, 4.0, 40.0, 48.0, 0.55, 0.4, 3.0),
    (ANGEL, 9.8, 10.0, 51.0, 24.0, 0.35, 0.9, 4.0),
    (GOLD, 12.8, 2.0, 46.0, 16.0, 0.6, 0.9, 4.0),
];
/// A clump of weed.
type Clump = (f64, &'static [(f64, f64)], f64);
/// Clumps of weed: root column, each blade's height and lean, and phase.
const WEEDS: [Clump; 5] = [
    (6.0, &[(11.0, 0.6), (8.0, -1.6), (6.0, 2.0)], 0.0),
    (13.0, &[(6.0, 1.2), (4.0, -1.2)], 1.7),
    (25.0, &[(4.0, -1.0), (3.0, 1.0)], 0.9),
    (37.0, &[(9.0, 1.2), (6.0, -1.4), (4.0, 2.2)], 2.4),
    (55.0, &[(7.0, -1.2), (5.0, 0.8)], 0.4),
];
/// Where the bubbles leave the stone.
const NOZZLE: f64 = 46.5;

fn hash(a: i64, b: i64) -> f64 {
    let mut h = (a as u32).wrapping_mul(0x27d4_eb2d) ^ (b as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

/// Whether a point of a fish is part of it: u from 0 at the tail's end to 1
/// at the nose, v from -1 at the top of the body to 1 at the bottom. The eye
/// is left out, so it shows as a hole.
fn fish_at(k: Kind, u: f64, v: f64) -> bool {
    let body = (1.0 + k.tail) / 2.0;
    let half = (1.0 - k.tail) / 2.0;
    let in_body = ((u - body) / half).powi(2) + v * v < 1.0;
    // The tail fans out from the body's root toward its end.
    let in_tail = u < k.tail + 0.08 && v.abs() < 0.25 + 0.85 * (k.tail + 0.08 - u) / (k.tail + 0.08);
    // A fin over the back and under the belly, tallest at the middle.
    let in_fin = k.fin > 0.0 && (u - body).abs() < half * 0.6 && v.abs() < 1.0 + k.fin * (1.0 - ((u - body) / (half * 0.6)).abs());
    let eye = (u - (1.0 - half * 0.45)).abs() < 0.06 && (v + 0.25).abs() < 0.3;
    (in_body || in_tail || in_fin) && !eye
}

/// The tank: gravel by dot, and the dots.
pub struct Aquarium {
    gravel: Vec<(i64, i64, u8)>,
    dots: ColorOctants,
}

/// An aquarium; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorOctants::new(META.cols, META.rows);
    let w = dots.width() as i64;
    // Gravel: a fine bed with coarser stones lying in it here and there.
    let mut gravel = vec![];
    for j in (FLOOR * 4.0) as i64..((FLOOR + 2.0) * 4.0) as i64 {
        for i in 2..w - 2 {
            let n = hash(i, j);
            let deep = (j as f64 / 4.0 - FLOOR) / 2.0;
            if n < 0.25 + 0.5 * deep {
                gravel.push((i, j, GRAVEL[(hash(i / 3, j / 2 + 7) * 2.0) as usize % 2]));
            }
        }
    }
    // The stone the bubbles leave from.
    for j in 60..68 {
        for i in 84..100 {
            let (dx, dy) = ((i as f64 + 0.5 - 92.0) / 8.0, (j as f64 + 0.5 - 67.0) / 7.0);
            if dx * dx + dy * dy < 1.0 {
                gravel.push((i, j, STONE));
            }
        }
    }
    Box::new(Aquarium { gravel, dots })
}

impl Piece for Aquarium {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        let w = self.dots.width() as i64;
        let u = t.rem_euclid(LOOP);
        let inside = |x: i64, y: i64| x > 1 && x < w - 2 && y > (SURFACE * 4.0) as i64 + 3 && y < (rows - 1) * 4;
        self.dots.clear();
        // The water's surface rippling under the rim.
        for x in 2..w - 2 {
            let c = x as f64 / 2.0;
            let ripple = (c * 0.55 - u * TAU / 4.0).sin() + 0.5 * (c * 0.23 + u * TAU / 6.0).sin();
            self.dots.set(x, (SURFACE * 4.0) as i64 + 2 + (ripple > 0.3) as i64, SURFACE_LINE);
        }
        for &(x, y, c) in &self.gravel {
            self.dots.set(x, y, c);
        }
        // Weed: blades that rise straight from the root and bend further
        // toward the tip, the tip swinging two or three columns.
        let weed = |dots: &mut ColorOctants, (c0, blades, ph): Clump, k: usize| {
            for &(h, lean) in blades {
                let steps = (h * 4.0) as i64;
                for s in 0..=steps {
                    let f = s as f64 / steps as f64;
                    let sway = 1.4 * (u * TAU / 6.0 + ph - f * 0.7 + lean * 0.2).sin();
                    let x = c0 + 0.5 + lean * f.powf(1.5) + sway * f * f;
                    let (dx, dy) = ((x * 2.0).floor() as i64, ((FLOOR - f * h) * 4.0).floor() as i64);
                    if inside(dx, dy) {
                        dots.set(dx, dy, WEED[k % 2]);
                    }
                }
            }
        };
        for (k, &clump) in WEEDS[..WEEDS.len() - 1].iter().enumerate() {
            weed(&mut self.dots, clump, k);
        }
        // Each fish swims there and back, easing into each turn, and drifts up
        // and down over the loop; for a moment mid-turn it faces out of the glass.
        for &(k, row, from, to, period, phase, drift, dp) in &FISH {
            let th = TAU * (u / period + phase);
            let p = (0.995 * th.sin()).asin() / (TAU / 4.0);
            let x = from + (p + 1.0) / 2.0 * (to - from) + k.len / 2.0;
            let y = row + drift * (TAU * u / LOOP + dp).sin() + 0.35 * (TAU * u / 6.0 + dp * 3.0).sin();
            let facing = th.cos();
            let (half_w, half_h) = if facing.abs() < 0.09 { (k.tall * 0.8, k.tall / 2.0) } else { (k.len / 2.0, k.tall / 2.0) };
            let fin = 1.0 + k.fin;
            for j in ((y - half_h * fin) * 4.0).floor() as i64..=((y + half_h * fin) * 4.0).ceil() as i64 {
                for i in ((x - half_w) * 2.0).floor() as i64..=((x + half_w) * 2.0).ceil() as i64 {
                    let (dx, dy) = ((i as f64 + 0.5) / 2.0 - x, ((j as f64 + 0.5) / 4.0 - y) / half_h);
                    let on = if facing.abs() < 0.09 {
                        // Face on: a round face, both eyes showing.
                        let r = (dx / half_w).hypot(dy);
                        r < 1.0 && !((dx.abs() - half_w * 0.45).abs() < 0.3 && (dy + 0.25).abs() < 0.35)
                    } else {
                        let along = dx / k.len + 0.5;
                        fish_at(k, if facing > 0.0 { along } else { 1.0 - along }, dy)
                    };
                    if on && inside(i, j) {
                        self.dots.set(i, j, k.color);
                    }
                }
            }
        }
        // Bubbles leave the stone in a stream, wobbling and swelling as they
        // rise, and break at the surface.
        for k in 0..80 {
            let age = (u - k as f64 * (LOOP / 80.0) + LOOP).rem_euclid(LOOP);
            let y = 14.6 - age * (2.6 + hash(k, 1));
            if y < SURFACE + 1.2 {
                continue;
            }
            let x = NOZZLE + 0.7 * (age * 3.5 + k as f64).sin() + (hash(k, 2) - 0.5);
            let (bx, by) = ((x * 2.0).floor() as i64, (y * 4.0).floor() as i64);
            let shape: &[(i64, i64)] = if y > 11.0 {
                &[(0, 0)]
            } else if y > 6.0 {
                &[(0, 0), (1, 0), (0, 1), (1, 1)]
            } else {
                &[(0, -1), (1, -1), (-1, 0), (2, 0), (-1, 1), (2, 1), (0, 2), (1, 2)]
            };
            for &(dx, dy) in shape {
                if inside(bx + dx, by + dy) && self.dots.get(bx + dx, by + dy).is_none() {
                    self.dots.set(bx + dx, by + dy, BUBBLE);
                }
            }
        }
        weed(&mut self.dots, WEEDS[WEEDS.len() - 1], 1); // one in front of the fish
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // The glass.
        grid.text_at(0, 0, &format!("┌{}┐", "─".repeat(META.cols - 2)), GLASS);
        grid.text_at(0, rows - 1, &format!("└{}┘", "─".repeat(META.cols - 2)), GLASS);
        for r in 1..rows - 1 {
            grid.set(0, r, '│', GLASS);
            grid.set(cols - 1, r, '│', GLASS);
        }
    }
}
