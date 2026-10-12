//! whale: a whale swimming left comes up, blows, rolls its back over and
//! dives, lifting its tail fluke clear of the water before it slips under.
//! After ascii.rest's whale.ts (MIT, @bas3line).
//!
//! The dive's timing, the swim and the blow are ascii.rest's. The whale is
//! redrawn in octants from smooth outlines, its back, then its rolling arch,
//! then its fluke, each sinking smoothly through the surface; the blow and
//! the water it throws are braille, a fine spray that opens into a bushy
//! head and drifts away downwind. In colour the whale is slate blue, darker
//! along its back and paler under its fluke, the sea blue, and the spray
//! white thinning to a pale blue mist.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// The whale's meta.
pub static META: Meta = Meta {
    name: "whale",
    slug: "whale",
    category: Category::Creatures,
    note: "an octant whale blowing a braille spout, arching, diving fluke last",
    cols: 72,
    rows: 18,
    fps: 15,
    charset: Charset::Extended,
    // The back, the flank, the fluke's underside, the eye; the sea's surface
    // and its swells; spray, and mist.
    palette: Some(&["#2c3a4c", "#4e6076", "#8a9cb2", "#0e1218", "#4a8cbc", "#1e4a6a", "#eef6ff", "#9cc0dc"]),
    ground: Some("#060c14"),
    ..Meta::DEFAULT
};

const BACK: u8 = 0;
const FLANK: u8 = 1;
const PALE: u8 = 2;
const EYE: u8 = 3;
const SURFACE: u8 = 4;
const SWELL: u8 = 5;
const SPRAY: u8 = 6;
const MIST: u8 = 7;
/// The water line's row; seconds a loop; seconds in at the first frame (the
/// blow at its height); columns a second it swims; the snout's column as it
/// first breaks the water; when the blow starts; columns a second the blow
/// drifts downwind.
const SEA: f64 = 13.0;
const LOOP: f64 = 10.0;
const T0: f64 = 1.5;
const SWIM: f64 = 2.6;
const X0: f64 = 13.0;
const BLOW: f64 = 0.7;
const WIND: f64 = 1.5;
/// When each shape comes up, is fully up, starts down and is under.
const SURF_UP: [f64; 4] = [0.0, 1.0, 3.0, 3.8];
const ARCH_UP: [f64; 4] = [3.1, 3.8, 4.6, 5.2];
const FLUKE_UP: [f64; 4] = [5.0, 5.9, 7.5, 8.5];
/// The blowhole and the back's fin, in columns behind the snout.
const HOLE: f64 = 18.5;
const FIN: f64 = 45.0;

fn hash(i: i64, s: i64) -> f64 {
    let mut h = ((i as u32) ^ 0x2c1b_3c6d).wrapping_mul(0x297a_2d39) ^ ((s + 7) as u32).wrapping_mul(0x9e37_79b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca77);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

fn ease(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// How far a shape is sunk, in its own heights: 1 under, 0 fully up.
fn depth(u: f64, [up, top, down, under]: [f64; 4]) -> f64 {
    if u < up || u > under {
        1.0
    } else if u < top {
        1.0 - ease((u - up) / (top - up))
    } else if u < down {
        0.0
    } else {
        ease((u - down) / (under - down))
    }
}

/// The surfacing back's height above the water, in rows, at columns behind
/// the snout: the head rising, the long back, the small fin, the tail stock.
fn back(d: f64) -> Option<f64> {
    if !(0.0..55.0).contains(&d) {
        return None;
    }
    let body = if d < 10.0 { 2.4 * (d / 10.0 * PI / 2.0).sin() + 0.4 } else { 2.8 - 1.9 * ((d - 10.0) / 45.0).powf(1.4) };
    let fin = (1.6 - (d - FIN).abs() * 0.5 - f64::from(u8::from(d > FIN)) * (d - FIN) * 0.4).max(0.0);
    Some(body.max(0.0) + fin)
}

/// The arch, the back rolling over: its height at columns from its front.
fn arch(d: f64) -> Option<f64> {
    if !(0.0..42.0).contains(&d) {
        return None;
    }
    let f = d / 42.0;
    let fin = (1.4 - (d - 27.0).abs() * 0.45).max(0.0);
    Some(5.8 * (f * PI).sin().powf(0.7) + fin)
}

/// The fluke above the water at (columns from its middle, rows from the
/// water up): two curved wings meeting at a notch over the tail stock.
fn fluke(dx: f64, up: f64) -> bool {
    let stock = dx.abs() < 1.1 && up < 4.5;
    let ax = dx.abs();
    let wing = (4.5..8.0).contains(&up) && ax < 15.0 && {
        // The wing's lower edge sweeps up from the notch to the tips, its upper edge flatter.
        let low = 4.5 + 2.6 * (ax / 15.0).powi(2);
        let high = 6.2 + 1.4 * (ax / 15.0).powf(0.8) - f64::from(u8::from(ax < 1.2)) * 0.8;
        up >= low && up <= high
    };
    stock || wing
}

/// The whale: its octants, and the braille for the spray.
pub struct Whale {
    body: ColorOctants,
    spray: ColorDots,
}

/// A whale; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Whale { body: ColorOctants::new(META.cols, META.rows), spray: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Whale {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let w = self.body.width() as i64;
        let u = (t + T0).rem_euclid(LOOP);
        let dive = ARCH_UP[3];
        let hx = X0 - SWIM * u.min(dive); // the snout
        let sea = (SEA * 4.0) as i64;
        self.body.clear();
        self.spray.clear();
        // A calm surface with the odd crest, and faint swells beneath it.
        for i in 0..w {
            let c = i as f64 / 2.0;
            let crest = (c * 0.61 - t * 1.4).sin() + 0.7 * (c * 0.27 + t * 0.8).sin();
            self.body.set(i, sea - i64::from(crest > 1.45), SURFACE);
            for (row, k) in [(SEA + 2.0, 0.6), (SEA + 4.0, -0.4)] {
                let v = (c * 0.45 + t * k * 2.0 + row).sin() * (c * 0.06 - t * k * 0.3 + row).sin();
                if v > 0.6 && i % 2 == 0 {
                    self.body.set(i, (row * 4.0) as i64 + 1, SWELL);
                }
            }
        }
        // Each shape stands on the water, sunk by a share of its height; only
        // what is above the water shows. Darker along the top.
        let (s_surf, s_arch, s_fluke) = (depth(u, SURF_UP), depth(u, ARCH_UP), depth(u, FLUKE_UP));
        for j in 0..sea {
            let y = SEA - (j as f64 + 0.5) / 4.0; // rows above the water
            for i in 0..w {
                let x = (i as f64 + 0.5) / 2.0;
                let mut paint = None;
                if let Some(hgt) = back(x - (hx - 2.0)) {
                    let top = hgt - s_surf * 3.6;
                    if y < top {
                        let d = x - (hx - 2.0);
                        let eye = (d - 3.4).abs() < 0.5 && (y - (top - 1.2)).abs() < 0.3;
                        paint = Some(if eye {
                            EYE
                        } else if top - y < 0.9 {
                            BACK
                        } else {
                            FLANK
                        });
                    }
                }
                if let Some(hgt) = arch(x - (hx - 2.0 + 14.0)) {
                    let top = hgt - s_arch * 7.2;
                    if y < top {
                        paint = Some(if top - y < 1.2 { BACK } else { FLANK });
                    }
                }
                let fluke_x = X0 - SWIM * dive + 14.0 + 27.0;
                if s_fluke < 1.0 && fluke(x - fluke_x, y + s_fluke * 8.0) {
                    paint = Some(if y + s_fluke * 8.0 < 6.4 { FLANK } else { PALE });
                }
                if let Some(c) = paint {
                    self.body.set(i, j, c);
                }
            }
        }
        // The blow: a column from the blowhole that opens into a bushy head,
        // then hangs, thins and drifts off downwind.
        let s = u - BLOW;
        if (0.0..3.0).contains(&s) {
            let bc = hx - 2.0 + HOLE;
            let from = X0 - SWIM * BLOW - 2.0 + HOLE;
            let br = SEA - (back(HOLE + 2.0).unwrap_or(2.0) - depth(u, SURF_UP) * 3.6);
            let top = 6.6 * (s / 0.5).min(1.0).sqrt();
            let cut = if s < 0.8 { 0.0 } else { (s - 0.8) * 7.0 };
            let hw = 1.0 + 3.6 * (s / 0.8).min(1.0) + 2.4 * (s - 0.8).max(0.0);
            let hh = 1.0 + 0.6 * (s / 0.8).min(1.0) + 0.5 * (s - 0.8).max(0.0);
            let hy0 = top - 0.6 - 0.5 * (s - 1.0).max(0.0);
            let amp = (s / 0.2).min(1.0) * if s < 1.0 { 1.0 } else { (-(s - 1.0) / 1.2).exp() };
            let drift = WIND * s;
            for j in ((br - 10.0) * 4.0).max(0.0) as i64..(br * 4.0) as i64 {
                let yy = br - (j as f64 + 0.5) / 4.0; // rows above the blowhole
                let lean = (yy / top.max(1.0)).min(1.0);
                let mid = bc + (from - bc) * lean + drift * lean * lean;
                for i in ((mid - 12.0) * 2.0) as i64..((mid + 12.0) * 2.0) as i64 {
                    if !(0..w).contains(&i) {
                        continue;
                    }
                    let dx = (i as f64 + 0.5) / 2.0 - mid;
                    let dy = (yy - hy0) / hh;
                    let mut d = if yy > cut && yy <= top { (-(dx / (0.3 + 0.17 * yy)).powi(2)).exp() } else { 0.0 };
                    d = d.max(amp * (-(dx / hw).powi(2) - dy * dy).exp());
                    d *= 0.92 + 0.16 * hash(i * 31 + j, 5);
                    if d > 0.12 && dither(i as usize, j as usize, d) {
                        self.spray.set(i, j, if d > 0.45 { SPRAY } else { MIST });
                    }
                }
            }
        }
        // White water thrown off ahead of the head as the arch goes under.
        if s_arch < 0.7 {
            let lip = hx - 2.0 + 14.0;
            for k in 0..10 {
                let p = (u * 1.4 + hash(k, 3)).rem_euclid(1.0);
                let a = hash(k, 4);
                let (x, y) = (lip - 1.0 - p * (1.0 + 2.0 * a), SEA - (0.6 + 1.6 * a) * (PI * p).sin());
                self.spray.set((x * 2.0) as i64, (y * 4.0) as i64, SPRAY);
            }
        }
        // The fluke streams water from its trailing edge as it rises.
        if s_fluke < 1.0 && u > FLUKE_UP[1] - 0.2 {
            let fx = X0 - SWIM * dive + 14.0 + 27.0;
            for k in 0..10 {
                let dx = -13.0 + f64::from(k) * 2.9;
                let edge = SEA - (4.5 + 2.6 * (dx.abs() / 15.0).powi(2) - s_fluke * 8.0);
                for lag in [0.0, 0.5] {
                    let p = (u * 1.6 + hash(k.into(), 9) + lag).rem_euclid(1.0);
                    let y = edge + p * p * (SEA - edge);
                    self.spray.set(((fx + dx) * 2.0) as i64, (y * 4.0) as i64, MIST);
                }
            }
        }
        grid.clear();
        self.spray.draw(grid, 0, 0);
        self.body.draw(grid, 0, 0);
    }
}
