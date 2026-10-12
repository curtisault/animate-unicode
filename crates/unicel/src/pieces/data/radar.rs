//! radar: a round scope with range rings and a sweep turning clockwise. Its
//! afterglow trails behind; contacts light as the beam crosses them, then
//! fade. After ascii.rest's radar.ts (MIT, @bas3line).
//!
//! The scope, its coast and its contacts' tracks are ascii.rest's. In
//! braille, four dots to each of its half-block pixels, the afterglow is
//! dithered in greens that fade with the beam's angle behind it, the coast
//! flares as the beam passes, the range rings are dotted circles and the
//! contacts bright blips that dim until the beam comes round again.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{PI, TAU};

/// The radar's meta.
pub static META: Meta = Meta {
    name: "radar",
    slug: "radar",
    category: Category::Data,
    note: "a braille radar scope, its sweep fading green, contacts blipping",
    cols: 53,
    rows: 25,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"range": 24, "unit": "nm"}"#),
    // The glow, faint to the beam; land; the coast lit; rings and bezel; contacts; text.
    palette: Some(&["#0e3a1c", "#16602c", "#24903e", "#3ac85a", "#9af0a8", "#1a3a24", "#4a8a5a", "#3a5a44", "#e8fff0", "#7ac88a"]),
    ground: Some("#040a06"),
    ..Meta::DEFAULT
};

const GLOW: Ramp = Ramp::new(0, 5);
const LAND: u8 = 5;
const COAST: u8 = 6;
const RING: u8 = 7;
const BLIP: u8 = 8;
const TEXT: u8 = 9;
/// Scope radius in ascii.rest's pixels (a cell wide, half a cell tall), two dots each.
const R: f64 = 19.0;
/// Seconds per revolution, radians of afterglow behind the beam, seconds
/// for a return to fade to a third.
const TURN: f64 = 4.0;
const TRAIL: f64 = 0.8;
const FADE: f64 = 2.4;
/// The centre, in pixels.
const CX: f64 = 26.0;
const CY: f64 = 24.5;

/// A contact's straight track across the scope.
struct Track {
    h: f64,
    off: f64,
    v: f64,
    l: f64,
    u0: f64,
    size: i64,
}

/// The radar: its range and unit, its contacts, and the dots.
pub struct Radar {
    range: f64,
    unit: String,
    tracks: Vec<Track>,
    dots: ColorDots,
}

/// Bearing of a point in pixels, clockwise from north.
fn bearing(x: f64, y: f64) -> f64 {
    (x - CX).atan2(CY - y).rem_euclid(TAU)
}

/// A coast along the south-west edge: land lies beyond this range at each bearing.
fn shore(b: f64) -> f64 {
    R * (0.66 + 0.09 * (b * 5.0 + 0.6).sin() + 0.05 * (b * 13.0).sin() + 2.2 * ((b - 4.15).abs() - 0.62).max(0.0))
}

fn is_land(x: f64, y: f64) -> bool {
    let b = bearing(x, y);
    b > 3.3 && b < 5.0 && (x - CX).hypot(y - CY) > shore(b)
}

impl Radar {
    fn at(&self, k: usize, t: f64) -> (f64, f64) {
        let tr = &self.tracks[k];
        let u = (tr.u0 + tr.v * t).rem_euclid(tr.l) - tr.l / 2.0;
        (CX + u * tr.h.sin() + tr.off * tr.h.cos(), CY - (u * tr.h.cos() - tr.off * tr.h.sin()))
    }
}

/// A radar; reads `range` (printed in the corner, the rings a third of it apart) and `unit`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    // Heading, offset from the centre, speed in pixels a second, start along
    // the track, size, and the gap it stays out for before coming back.
    let tracks = [
        (1.05, -5.0, 1.1, 15.0, 2, 8.0),
        (3.45, 6.0, 0.9, -1.0, 2, 10.0),
        (5.3, 9.0, 1.2, 4.0, 1, 6.0),
        (2.1, -11.0, 0.8, -9.0, 1, 8.0),
        (0.4, 3.0, 1.3, -21.0, 1, 8.0),
        (4.75, 13.0, 0.7, 2.0, 1, 6.0),
    ]
    .iter()
    .map(|&(h, off, v, u, size, gap): &(f64, f64, f64, f64, i64, f64)| {
        let l = 2.0 * (R * R - off * off).sqrt() + gap;
        Track { h, off, v, l, u0: u + l / 2.0, size }
    })
    .collect();
    Box::new(Radar { range: options.f64("range", 24.0), unit: options.string("unit", "nm"), tracks, dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Radar {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let sweep = (t / TURN * TAU).rem_euclid(TAU);
        // Seconds since the beam passed a bearing.
        let ago = |b: f64| (sweep - b).rem_euclid(TAU) / TAU * TURN;
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        // Dots to pixels: two to a pixel each way.
        let px = |d: i64| (d as f64 + 0.5) / 2.0;
        self.dots.clear();
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (px(x), px(y));
                let d = (fx - CX).hypot(fy - CY);
                if d >= R - 0.3 {
                    continue;
                }
                let b = bearing(fx, fy);
                let a = ago(b) / TURN * TAU;
                // Afterglow: the beam itself, then a fading wedge behind it.
                if a < TRAIL {
                    let glow = 1.0 - a / TRAIL;
                    if dither(x as usize, y as usize, 0.15 + 0.85 * glow * glow) {
                        self.dots.set(x, y, GLOW.at(glow));
                        continue;
                    }
                }
                // Land is a faint fill; its coast flares for a moment after each pass.
                if is_land(fx, fy) {
                    let edge = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)].iter().any(|&(dx, dy)| !is_land(fx + dx, fy + dy));
                    if edge && ago(b) < 1.2 {
                        self.dots.set(x, y, COAST);
                    } else if dither(x as usize, y as usize, if edge { 0.5 } else { 0.2 }) {
                        self.dots.set(x, y, LAND);
                    }
                }
            }
        }
        // The bezel, and two range rings dotted every few degrees.
        let (cx, cy) = (CX * 2.0 + 0.5, CY * 2.0 + 0.5);
        let mut a: f64 = 0.0;
        while a < TAU {
            let rb = (R + 1.5) * 2.0;
            self.dots.set((cx + rb * a.sin()).round() as i64, (cy - rb * a.cos()).round() as i64, RING);
            a += 0.004;
        }
        for rr in [R / 3.0, 2.0 * R / 3.0] {
            let steps = (TAU * rr * 2.0 / 3.0) as usize;
            for k in 0..steps {
                let a = k as f64 / steps as f64 * TAU;
                self.dots.set((cx + rr * 2.0 * a.sin()).round() as i64, (cy - rr * 2.0 * a.cos()).round() as i64, RING);
            }
        }
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            self.dots.set(cx as i64 + dx, cy as i64 + dy, RING);
        }
        // A contact shows where it was when the beam last crossed it, fading.
        let mut seen = 0;
        for k in 0..self.tracks.len() {
            let (x0, y0) = self.at(k, t);
            let s = ago(bearing(x0, y0));
            let (x, y) = self.at(k, t - s);
            if (x - CX).hypot(y - CY) > R - 1.2 || is_land(x, y) {
                continue;
            }
            seen += 1;
            let g = (-s / FADE).exp();
            let size = self.tracks[k].size + 1;
            for dy in 0..size {
                for dx in 0..size {
                    if dither((x * 2.0) as usize + dx as usize, (y * 2.0) as usize + dy as usize, 0.4 + 0.6 * g) {
                        self.dots.set((x * 2.0) as i64 + dx, (y * 2.0) as i64 + dy, if g > 0.42 { BLIP } else { GLOW.at(0.75) });
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        grid.text_at(CX as i64 - 1, 1, "000", TEXT);
        grid.text_at(CX as i64 - 1, rows - 2, "180", TEXT);
        grid.text_at(1, 12, "270", TEXT);
        grid.text_at(cols - 4, 12, "090", TEXT);
        let deg = format!("{:03}", (sweep * 180.0 / PI).round() as i64 % 360);
        let corner = |grid: &mut Grid, r: i64, l: &str, rt: &str| {
            grid.text_at(0, r, l, TEXT);
            grid.text_at(cols - rt.chars().count() as i64, r, rt, TEXT);
        };
        corner(grid, 1, &format!(" brg {deg}°"), &format!("rng {} {} ", self.range, self.unit));
        corner(grid, rows - 2, &format!(" contacts {seen}"), &format!("rings {} {} ", (self.range / 3.0).round(), self.unit));
    }
}
