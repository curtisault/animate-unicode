//! sundial: the Samrat Yantra at Jantar Mantar, Jaipur, seen from the
//! south. The gnomon's shadow lies on the quadrants where the sun at Jaipur
//! would put it at this hour of the viewer's own clock (not Jaipur solar
//! time). After ascii.rest's sundial.ts (MIT, @bas3line).
//!
//! The instrument, the sun's hour angle and the rule for day and night are
//! ascii.rest's. It is drawn in octants: the quadrants' face a marble ring
//! dithered by the sunlight, an hour line every fifteen degrees, the
//! shadow's edge a hard dark line with a soft fringe beside it, and the
//! gnomon's stair climbing between its rails to the chhatri on top. In
//! colour the face is sunlit cream, the shadow on it a cool slate blue, the
//! gnomon the ochre stone of Jaipur, the sun gold, and at night the stars come out.

use crate::clock::Ticker;
use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// The sundial's meta.
pub static META: Meta = Meta {
    name: "sundial",
    slug: "sundial",
    category: Category::Objects,
    note: "jaipur's samrat yantra in octants, its slate shadow set by your clock",
    cols: 65,
    rows: 23,
    fps: 10,
    charset: Charset::Extended,
    clock: true,
    // The face lit; its hour lines; the shadow; its edge; the gnomon and its
    // rails; the walls; the sun; a star; the numerals.
    palette: Some(&["#f2e4c4", "#a8906c", "#56607c", "#232838", "#d8985e", "#7a4e2e", "#b89a7a", "#ffd24a", "#8a90a8", "#ece0c4"]),
    ground: Some("#0a0b10"),
    ..Meta::DEFAULT
};

const FACE: u8 = 0;
const HOUR: u8 = 1;
const SHADOW: u8 = 2;
const EDGE: u8 = 3;
const GNOMON: u8 = 4;
const RAIL: u8 = 5;
const WALL: u8 = 6;
const SUN: u8 = 7;
const STAR: u8 = 8;
const NUMERAL: u8 = 9;
/// Jaipur's latitude; the centre of the quadrants' arc (on the gnomon's
/// edge); the arc's radius across and down (a cell twice as tall as wide);
/// the inner edge of the face, as a share of the outer; the gnomon's ground
/// row, top row and half width at the foot.
const LAT: f64 = 26.92 * PI / 180.0;
const CX: f64 = 32.5;
const YC: f64 = 9.5;
const RX: f64 = 26.0;
const RY: f64 = 11.6;
const INNER: f64 = 0.77;
const GROUND: f64 = 22.0;
const APEX: f64 = 4.0;
const FOOT: f64 = 2.9;

/// The gnomon's half width at row r.
fn half(r: f64) -> f64 {
    0.6 + (FOOT - 0.6) * (r - APEX) / (GROUND - 1.0 - APEX)
}

/// The sundial: its ticker, the stars, and the dots.
pub struct Sundial {
    ticker: Ticker,
    stars: Vec<(i64, i64, f64)>,
    dots: ColorOctants,
}

/// A sundial; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut rand = Mulberry32::new(5);
    let stars = (0..22).map(|_| ((rand.next_f64() * META.cols as f64) as i64 * 2, (rand.next_f64() * 8.0) as i64 * 4 + 1, rand.next_f64() * 6.0)).collect();
    Box::new(Sundial { ticker: Ticker::default(), stars, dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Sundial {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        let now = self.ticker.now(t, env.now).local();
        let hours = f64::from(now.hour) + f64::from(now.minute) / 60.0;
        let day = f64::from(now.yday - 1) + hours / 24.0;
        // The hour angle, 15 degrees an hour from noon, and whether the sun is
        // up at Jaipur's latitude on this day of the year.
        let ha = (hours - 12.0) * 15.0 * PI / 180.0;
        let dec = -23.44 * PI * (TAU * (day + 10.0) / 365.0).cos() / 180.0;
        let up = dec.cos() * ha.cos() * LAT.cos() + dec.sin() * LAT.sin() > 0.0;
        let hh = up.then_some(ha.clamp(-FRAC_PI_2, FRAC_PI_2));
        // Near noon the edge would fall behind the gnomon, so it is drawn against its foot.
        let hidden = (half(GROUND - 1.0) / RX).asin();
        let edge = hh.map(|hh| (if hh < 0.0 { -1.0 } else { 1.0 }) * hh.abs().max(hidden));
        self.dots.clear();
        // The face: lit marble with an hour line every fifteen degrees, and in
        // daylight the shadow from the gnomon's foot out to the hour angle on
        // the side away from the sun, its edge hard and fringed.
        for j in (YC * 4.0) as i64..h {
            for i in 0..w {
                let (x, y) = ((i as f64 + 0.5) / 2.0, (j as f64 + 0.5) / 4.0);
                let (u, v) = ((x - CX) / RX, (y - YC) / RY);
                let e = u.hypot(v);
                if e >= 1.0 || e <= INNER {
                    continue;
                }
                let psi = u.atan2(v); // 0 straight down, east positive
                let hour = ((psi / (PI / 12.0)).round() - psi / (PI / 12.0)).abs() * (PI / 12.0) * RX * e < 0.25;
                let rim = e > 0.975;
                let (paint, ink) = match (hh, edge) {
                    (Some(_), Some(edge)) if (psi - edge).abs() * RX * e < 0.45 => (EDGE, 1.0),
                    (Some(_), Some(edge)) if (psi - edge).abs() * RX * e < 1.2 => (SHADOW, 0.6),
                    (Some(hh), _) if psi * hh >= 0.0 && psi.abs() <= hh.abs() => (SHADOW, 0.3),
                    _ if hour || rim => (HOUR, 1.0),
                    (None, _) => (SHADOW, 0.35),
                    _ => (FACE, 0.75),
                };
                if dither(i as usize, j as usize, ink) {
                    self.dots.set(i, j, paint);
                }
            }
        }
        // The end walls the quadrants' tips stand on.
        for s in [-1.0, 1.0] {
            let wall = ((CX + s * RX) * 2.0) as i64;
            for j in (YC * 4.0) as i64..(GROUND * 4.0) as i64 {
                self.dots.set(wall, j, WALL);
                self.dots.set(wall + 1, j, WALL);
            }
            for i in 0..((RX * (1.0 - INNER)) * 2.0) as i64 {
                self.dots.set(wall - (s * i as f64) as i64, (YC * 4.0) as i64 - 1, WALL);
            }
        }
        // The gnomon in front, narrowing as it climbs: its stair up the middle
        // between two rails.
        for j in (APEX * 4.0) as i64..(GROUND * 4.0) as i64 {
            let r = j as f64 / 4.0;
            let hw = half(r);
            for i in ((CX - hw) * 2.0).floor() as i64..((CX + hw) * 2.0).ceil() as i64 {
                let x = (i as f64 + 0.5) / 2.0 - CX;
                let rail = (x.abs() - hw).abs() < 0.5 || (hw > 2.0 && (x.abs() - (hw - 1.0)).abs() < 0.3);
                let stair = j % 2 == 0;
                if rail || stair || dither(i as usize, j as usize, 0.5) {
                    self.dots.set(i, j, if rail { RAIL } else { GNOMON });
                }
            }
        }
        // The chhatri on top.
        for j in ((APEX - 2.0) * 4.0) as i64..(APEX * 4.0) as i64 {
            for i in ((CX - 1.2) * 2.0) as i64..((CX + 1.2) * 2.0) as i64 {
                let x = (i as f64 + 0.5) / 2.0 - CX;
                let roof = j < ((APEX - 1.0) * 4.0) as i64;
                if roof && x.abs() < 1.2 - (((APEX - 1.0) * 4.0) as i64 - j) as f64 * 0.12 || (!roof && (x.abs() - 0.9).abs() < 0.3) {
                    self.dots.set(i, j, GNOMON);
                }
            }
        }
        for i in 0..w {
            for j in 0..2 {
                self.dots.set(i, (GROUND * 4.0) as i64 + j, WALL);
            }
        }
        // The sky: the sun on its arc, east to the right, its rays flickering;
        // at night, the stars.
        match hh {
            None => {
                for &(x, y, ph) in &self.stars {
                    if (t * 1.4 + ph).sin() > -0.3 && self.dots.get(x, y).is_none() {
                        self.dots.set(x, y, STAR);
                    }
                }
            }
            Some(hh) => {
                let (sx, sy) = ((CX - 24.0 * hh.sin()) * 2.0, (1.5 + 4.0 * (1.0 - hh.cos())) * 4.0 + 2.0);
                let step = (t / 0.45).floor() as i64 % 4;
                for j in (sy - 9.0) as i64..(sy + 9.0) as i64 {
                    for i in (sx - 9.0) as i64..(sx + 9.0) as i64 {
                        let (dx, dy) = (i as f64 + 0.5 - sx, j as f64 + 0.5 - sy);
                        let d = dx.hypot(dy);
                        let a = dy.atan2(dx).rem_euclid(PI / 4.0) - PI / 8.0;
                        // Upright and level rays, then slanted ones, then all of them.
                        let octant = (dy.atan2(dx) / (PI / 4.0)).round() as i64;
                        let shown = match step {
                            0 => octant % 2 == 0,
                            2 => octant % 2 != 0,
                            _ => true,
                        };
                        let ray = d > 4.5 && d < 8.5 && a.abs() < 0.12 && shown;
                        if (d < 3.6 || ray) && j >= 0 && self.dots.get(i, j).is_none() {
                            self.dots.set(i, j, SUN);
                        }
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // Hour numbers just inside the face, morning on the west quadrant; and
        // arches in the masonry.
        for hr in -6..=6 {
            if hr == 0 {
                continue;
            }
            let p = f64::from(hr) * PI / 12.0;
            let s = (if hr < 0 { 12 + hr } else { hr }).to_string();
            let c = (CX + RX * (INNER - 0.09) * p.sin() - s.len() as f64 / 2.0).round() as i64;
            grid.text_at(c, (YC + RY * (INNER - 0.12) * p.cos()) as i64, &s, NUMERAL);
        }
        for s in [-1.0, 1.0] {
            for dx in [16.0, 21.0] {
                let c = (CX + s * dx).floor() as i64 - 1;
                grid.text_at(c, GROUND as i64 - 2, "╭─╮", WALL);
                grid.text_at(c, GROUND as i64 - 1, "│ │", WALL);
            }
        }
    }
}
