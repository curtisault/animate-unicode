//! train: a steam train crossing the frame on its track, a coach and tender
//! behind the engine. Smoke puffs from the stack in time with the beat of
//! the wheels, whose rods turn as they roll; it leaves on the right and
//! comes back. After ascii.rest's train.ts (MIT, @bas3line).
//!
//! The run, the beat and the smoke are ascii.rest's; the train is redrawn in
//! octants from its shapes: a coach with its windows, the tender heaped with
//! coal, and the engine's cab, boiler, dome, stack and cowcatcher, so it
//! slides a dot at a time. Its wheels are true circles whose spokes turn as
//! far as they roll, the rods driven round by the cranks, and the smoke is
//! braille, billows that swell and thin as they drift back. In colour the
//! engine is green with a black smokebox and stack, a brass dome and red
//! wheels, the coach maroon with pale windows, and the smoke grey.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{FRAC_PI_2, TAU};

/// The train's meta.
pub static META: Meta = Meta {
    name: "train",
    slug: "train",
    category: Category::Objects,
    note: "a green octant steam engine and maroon coach, wheels turning, smoke",
    cols: 76,
    rows: 26,
    fps: 15,
    charset: Charset::Extended,
    // The engine, its black, the brass; red wheels; silver rods; the coach,
    // its windows; coal; rails, sleepers, ballast; smoke, thin to thick.
    palette: Some(&[
        "#2e6a40", "#1a1c20", "#dcac44", "#c63c2a", "#b8bcc8", "#7a2030", "#c8e0f0", "#24242a", "#8a8e9a", "#5a4030", "#4a4a4e", "#5a5e68", "#9a9ea8",
        "#d8dce4",
    ]),
    ground: Some("#0a0c10"),
    ..Meta::DEFAULT
};

const GREEN: u8 = 0;
const BLACK: u8 = 1;
const BRASS: u8 = 2;
const WHEEL: u8 = 3;
const ROD: u8 = 4;
const COACH: u8 = 5;
const WINDOW: u8 = 6;
const COAL: u8 = 7;
const RAILS: u8 = 8;
const SLEEPER: u8 = 9;
const BALLAST: u8 = 10;
const SMOKE: Ramp = Ramp::new(11, 3);
/// The train's length, rear buffer to the cowcatcher's tip; the rail's row;
/// columns a second; the coupled wheels' centre columns; the small wheels'
/// columns; where the main rod meets the crosshead mid stroke; the back of
/// the cylinder; the chimney's column; the driving wheels' radius in rows;
/// seconds the smoke lasts; columns a second it drifts back.
const LEN: f64 = 73.0;
const RAIL: f64 = 23.0;
const SPEED: f64 = 8.0;
const DRIVE: [f64; 2] = [42.0, 52.0];
const SMALL: [f64; 8] = [3.0, 6.0, 12.0, 15.0, 22.0, 25.0, 28.0, 31.0];
const CROSS: f64 = 58.0;
const CYL: f64 = 60.0;
const STACK: f64 = 62.0;
const RIM: f64 = 2.5;
const LIFE: f64 = 5.5;
const WIND: f64 = 2.4;

fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

/// The body's paint at (x, y): x in columns from the train's rear, y the
/// row. Windows are cut out, and the boiler's bands are gaps.
fn body(x: f64, y: f64) -> Option<u8> {
    let inside = |x0: f64, x1: f64, y0: f64, y1: f64| x >= x0 && x < x1 && y >= y0 && y < y1;
    // The coach: an arched roof, four windows, a frame under it.
    if inside(0.5, 17.5, 14.4, 20.3) {
        let arch = y < 15.2 && ((x - 9.0) / 8.8).powi(2) + ((y - 15.2) / 0.9).powi(2) > 1.0;
        if arch {
            return None;
        }
        let window = (16.2..18.3).contains(&y) && [2.0, 6.0, 10.0, 14.0].iter().any(|&w| x >= w && x < w + 2.5);
        return Some(if window {
            WINDOW
        } else if !(15.0..19.6).contains(&y) {
            BLACK
        } else {
            COACH
        });
    }
    // The tender, heaped with coal.
    if inside(20.0, 31.5, 15.4, 20.3) {
        return Some(if y < 16.0 {
            COAL
        } else if y >= 19.6 {
            BLACK
        } else {
            GREEN
        });
    }
    if inside(21.0, 30.5, 14.6, 15.4) && ((x - 25.7) / 4.8).powi(2) + ((y - 15.4) / 0.8).powi(2) < 1.0 {
        return Some(COAL);
    }
    // The engine: the cab with its roof and window.
    if inside(34.0, 43.5, 11.4, 12.1) {
        return Some(BLACK);
    }
    if inside(34.5, 42.5, 12.1, 19.6) {
        return if inside(36.0, 41.0, 13.0, 15.6) { Some(WINDOW) } else { Some(GREEN) };
    }
    // The boiler, banded in brass, a dome on top, the stack at the front.
    if inside(42.5, 64.5, 12.6, 17.2) {
        let band = [48.5, 55.0, 61.0].iter().any(|&b| (x - b).abs() < 0.3);
        return Some(if band { BRASS } else { GREEN });
    }
    if ((x - 49.0) / 2.2).powi(2) + ((y - 12.6) / 1.3).powi(2) < 1.0 && y < 12.6 {
        return Some(BRASS);
    }
    if inside(60.0, 63.0, 10.0, 12.6) || inside(59.4, 63.6, 9.2, 10.0) {
        return Some(BLACK);
    }
    // The smokebox's front, the running board, the cylinder, the buffer beam.
    if inside(64.5, 66.5, 12.2, 17.6) || inside(42.0, 66.5, 17.2, 17.8) || inside(56.0, 62.0, 17.8, 19.4) || inside(64.0, 67.5, 17.8, 18.6) {
        return Some(BLACK);
    }
    // The cowcatcher, slanting down and forward to the rail.
    if (18.6..22.6).contains(&y) && x >= 64.5 && x < 66.5 + (y - 18.6) * 0.9 && ((x - 64.5) * 2.0).floor() as i64 % 2 == 0 {
        return Some(WHEEL);
    }
    None
}

/// The train: its smoke's density by dot, and the two canvases.
pub struct Train {
    dens: Vec<f32>,
    smoke: ColorDots,
    engine: ColorOctants,
}

/// A train; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let smoke = ColorDots::new(META.cols, META.rows);
    let dens = vec![0.0; smoke.width() * smoke.height()];
    Box::new(Train { dens, smoke, engine: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Train {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let cols = META.cols as f64;
        let span = cols + LEN + 1.0; // a lap: in from the left, across and out
        let x0 = LEN - 16.0; // at t = 0 the engine and tender are in, the coach coming
        let left = |t: f64| (x0 + SPEED * t).rem_euclid(span) - LEN; // the train's left edge
        let omega = SPEED / (2.0 * RIM); // radians a second the wheels turn
        let beat = FRAC_PI_2 / omega; // four beats to a turn
        let ox = left(t);
        let (w, h) = (self.engine.width() as i64, self.engine.height() as i64);
        // Smoke: let go from the stack, left where it was let go, rising,
        // swelling and thinning as it drifts back on the wind.
        self.dens.fill(0.0);
        let mut k = ((t - LIFE) / beat).ceil() as i64;
        while k as f64 * beat <= t {
            let age = t - k as f64 * beat;
            let u = age / LIFE;
            let x = left(k as f64 * beat) + STACK - WIND * age + 0.8 * (k as f64 * 1.7).sin();
            let y = 8.4 - 7.2 * (1.0 - (-age / 2.0).exp()) + 0.5 * (k as f64 * 2.9).sin();
            let rad = 1.5 + 0.75 * age.powf(0.8);
            let wt = (1.0 - u).powf(1.2);
            for j in ((y - rad) * 4.0).floor().max(0.0) as i64..((y + rad) * 4.0).ceil().min(h as f64) as i64 {
                for i in ((x - 2.0 * rad) * 2.0).floor().max(0.0) as i64..((x + 2.0 * rad) * 2.0).ceil().min(w as f64) as i64 {
                    let (dx, dy) = (((i as f64 + 0.5) / 2.0 - x) / 2.0 / rad, ((j as f64 + 0.5) / 4.0 - y) / rad);
                    let q = dx * dx + dy * dy;
                    if q < 1.0 {
                        // Lit from above, so each billow keeps its round top where it overlaps the last.
                        let v = wt * (1.0 - q).sqrt() * (0.6 - 0.4 * dy - 0.1 * dx);
                        let d = &mut self.dens[(j * w + i) as usize];
                        *d = d.max(v as f32);
                    }
                }
            }
            k += 1;
        }
        self.smoke.clear();
        for (k, &v) in self.dens.iter().enumerate() {
            let v = f64::from(v);
            if v > 0.08 && dither(k % w as usize, k / w as usize, v * 1.2) {
                self.smoke.set(k as i64 % w, k as i64 / w, SMOKE.at(v * 1.4));
            }
        }
        self.engine.clear();
        // The track: the rail, the sleepers, the ballast under it.
        for i in 0..w {
            self.engine.set(i, (RAIL * 4.0) as i64 + 1, RAILS);
            self.engine.set(i, (RAIL * 4.0) as i64 + 2, RAILS);
            if (i / 2) % 3 == 0 {
                for j in 3..6 {
                    self.engine.set(i, (RAIL * 4.0) as i64 + j, SLEEPER);
                }
            }
            for j in ((RAIL + 2.0) * 4.0) as i64..h {
                if hash(i, j) < 0.25 {
                    self.engine.set(i, j, BALLAST);
                }
            }
        }
        for j in 0..h {
            for i in 0..w {
                // Windows, coal and the brass are sparser than the paintwork,
                // so they show in one ink too.
                if let Some(c) = body((i as f64 + 0.5) / 2.0 - ox, (j as f64 + 0.5) / 4.0) {
                    let ink = match c {
                        WINDOW => 0.3,
                        COAL => 0.5,
                        BRASS => 0.6,
                        _ => 1.0,
                    };
                    if dither(i as usize, j as usize, ink) {
                        self.engine.set(i, j, c);
                    }
                }
            }
        }
        // The wheels turn as far as they roll: rims, spokes and hubs.
        let a = omega * t;
        let wheel = |engine: &mut ColorOctants, cx: f64, cy: f64, r: f64, spokes: i64| {
            for j in ((cy - r) * 4.0).floor() as i64..=((cy + r) * 4.0).ceil() as i64 {
                for i in ((cx - 2.0 * r) * 2.0).floor() as i64..=((cx + 2.0 * r) * 2.0).ceil() as i64 {
                    let (dx, dy) = (((i as f64 + 0.5) / 2.0 - cx) / 2.0, (j as f64 + 0.5) / 4.0 - cy);
                    let d = dx.hypot(dy);
                    if d > r {
                        continue;
                    }
                    let ang = dy.atan2(dx) + a;
                    let spoke = ((ang * spokes as f64 / TAU).round() - ang * spokes as f64 / TAU).abs() * TAU / spokes as f64 * d < 0.18;
                    if d > r - 0.35 || d < 0.45 || spoke {
                        engine.set(i, j, WHEEL);
                    } else {
                        engine.unset(i, j);
                    }
                }
            }
        };
        for &c in &DRIVE {
            wheel(&mut self.engine, ox + c + 0.5, RAIL - 2.5, RIM, 8);
        }
        for &c in &SMALL {
            wheel(&mut self.engine, ox + c + 0.5, RAIL - 0.9, 0.9, 4);
        }
        // The crank pins go round together with the rod between them, and the
        // main rod runs on to the crosshead.
        let (pr, pc) = (RAIL - 2.5 - 1.2 * a.sin(), 1.2 * 2.0 * a.cos());
        let rod = |engine: &mut ColorOctants, (x0, y0): (f64, f64), (x1, y1): (f64, f64)| {
            let n = ((x1 - x0).abs() * 2.0).max((y1 - y0).abs() * 4.0).ceil().max(1.0) as i64;
            for s in 0..=n {
                let f = s as f64 / n as f64;
                engine.set(((x0 + (x1 - x0) * f) * 2.0) as i64, ((y0 + (y1 - y0) * f) * 4.0) as i64, ROD);
            }
        };
        rod(&mut self.engine, (ox + DRIVE[0] + 0.5 + pc, pr), (ox + DRIVE[1] + 0.5 + pc, pr));
        let xh = ox + CROSS + a.cos();
        rod(&mut self.engine, (ox + DRIVE[1] + 0.5 + pc, pr), (xh, RAIL - 2.5));
        rod(&mut self.engine, (xh, RAIL - 2.5), (ox + CYL, RAIL - 2.5));
        grid.clear();
        self.smoke.draw(grid, 0, 0);
        self.engine.draw(grid, 0, 0);
    }
}
