//! coffee: a cup of coffee on its saucer, lit from the upper left, with a
//! few thin wisps of steam lifting off the surface and curling as they climb.
//! After ascii.rest's coffee.ts (MIT, @bas3line).
//!
//! The cup, the saucer, the light and the steam are ascii.rest's. The cup
//! and saucer, worked out eight times a cell as there, are drawn a sample to
//! an octant dot, dithered by the light; the steam is braille, fine wisps
//! that curl as they climb and pass in front of the dark coffee but behind
//! the porcelain. In colour the cup is a cobalt glaze with a bright lip, the
//! saucer cream, the coffee a dark brown pool with a ring of tan crema and a
//! glint of the light, and the steam a pale grey.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// The coffee's meta.
pub static META: Meta = Meta {
    name: "coffee",
    slug: "coffee",
    category: Category::Objects,
    note: "a cobalt octant cup of coffee on a saucer, braille steam curling up",
    cols: 52,
    rows: 24,
    fps: 20,
    charset: Charset::Extended,
    // The glaze, shadow to glint; the saucer, shadow to light; coffee,
    // crema, its glint; steam.
    palette: Some(&["#163a66", "#2a62a4", "#4a8ad0", "#b4d4f4", "#6a5e50", "#b8a88c", "#ece2cc", "#2a160a", "#b07a40", "#fff4e0", "#d0d4dc"]),
    ground: Some("#0d0b0a"),
    ..Meta::DEFAULT
};

const GLAZE: Ramp = Ramp::new(0, 4);
const PORCELAIN: Ramp = Ramp::new(4, 3);
const COFFEE: u8 = 7;
const CREMA: u8 = 8;
const SHINE: u8 = 9;
const STEAM: u8 = 10;
/// How steeply we look down at the table; the cup's height and its radius at
/// the rim in rows; the coffee's level; and seconds before the steam repeats.
const TILT: f64 = 0.34;
const H: f64 = 7.6;
const RT: f64 = 6.2;
const HC: f64 = H - 0.55;
const PERIOD: f64 = 6.0;
/// Wisps: x and z where they leave the coffee, height in rows, phase, lean.
const WISPS: [(f64, f64, f64, f64, f64); 3] = [(-2.4, 0.6, 11.0, 0.0, -0.12), (0.3, -1.4, 14.0, 2.2, 0.05), (2.9, 0.5, 10.0, 4.1, 0.15)];
/// The saucer turned on a lathe, radius and height: a flat well, rising to a rounded lip.
const SAUCER: [(f64, f64); 9] = [(0.0, 0.0), (4.4, 0.0), (6.5, 0.12), (8.7, 0.45), (9.9, 0.78), (10.4, 0.92), (10.6, 0.8), (10.55, 0.55), (10.2, 0.4)];

fn cup_r(h: f64) -> f64 {
    RT * (0.62 + 0.38 * (FRAC_PI_2 * (h.max(0.0) / H).min(1.0)).sin())
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

fn norm(x: f64, y: f64, z: f64) -> [f64; 3] {
    let m = (x * x + y * y + z * z).sqrt();
    [x / m, y / m, z / m]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// What a dot of the still picture is: its light and its paint.
#[derive(Clone, Copy, PartialEq)]
enum Part {
    Saucer,
    Cup,
    Coffee(u8),
}

/// The cup: its still picture by dot (light and part), which cells the
/// steam may cross, where each wisp leaves the coffee, and the canvases.
pub struct Coffee {
    still: Vec<Option<(f64, Part)>>,
    free: Vec<bool>,
    starts: Vec<(f64, f64)>,
    porcelain: ColorOctants,
    steam: ColorDots,
}

/// A cup of coffee; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as f64, META.rows as f64);
    let porcelain = ColorOctants::new(META.cols, META.rows);
    let (w, hh) = (porcelain.width(), porcelain.height());
    let (cs, sn) = ((1.0 - TILT * TILT).sqrt(), TILT);
    let (cx, base) = (cols / 2.0 - 1.5, rows - 4.4); // the saucer's centre, a little left so the handle fits
    let l = norm(-0.62, 0.62, 0.48);
    let to_cell = |x: f64, y: f64, z: f64| (cx + 2.0 * x, base - (y * cs - z * sn));
    let mut depth = vec![-1e9; w * hh];
    let mut still: Vec<Option<(f64, Part)>> = vec![None; w * hh];
    let mut lip = vec![false; w * hh];
    let mut plot = |x: f64, y: f64, z: f64, n: [f64; 3], albedo: f64, extra: f64, part: Part, is_lip: bool| {
        let (c, r) = to_cell(x, y, z);
        let (px, py) = ((c * 2.0).floor(), (r * 4.0).floor());
        if px < 0.0 || py < 0.0 || px >= w as f64 || py >= hh as f64 {
            return;
        }
        let k = py as usize * w + px as usize;
        let d = z * cs + y * sn;
        if d <= depth[k] {
            return;
        }
        depth[k] = d;
        let diff = dot3(n, l).max(0.0);
        still[k] = Some((albedo * (0.1 + 0.9 * diff) + extra, part));
        lip[k] = is_lip;
    };
    // A glaze's glint, where the light bounces toward us.
    let hv = norm(l[0], l[1] + sn, l[2] + cs);
    let glint = |n: [f64; 3], p: i32| dot3(n, hv).max(0.0).powi(p);
    // Is the cup in the way between this point and the light?
    let shadowed = |x: f64, y: f64, z: f64| {
        let mut s = 0.3;
        while s < 16.0 {
            let yy = y + l[1] * s;
            if yy > H {
                return false;
            }
            if yy > 0.0 && (x + l[0] * s).hypot(z + l[2] * s) < cup_r(yy) {
                return true;
            }
            s += 0.25;
        }
        false
    };
    // The saucer: lit more on the left as it turns from the light, shaded by
    // the cup and darkened close round its foot.
    for seg in SAUCER.windows(2) {
        let ((r1, y1), (r2, y2)) = (seg[0], seg[1]);
        let len = (r2 - r1).hypot(y2 - y1);
        let (nr, ny) = (-(y2 - y1) / len, (r2 - r1) / len);
        let mut f = 0.0;
        while f <= 1.0 {
            let (p, y) = (r1 + (r2 - r1) * f, y1 + (y2 - y1) * f);
            let mut a = 0.0;
            while a < TAU {
                let (x, z) = (p * a.cos(), p * a.sin());
                let n = norm(nr * a.cos(), ny, nr * a.sin());
                let side = 0.78 - 0.22 * (x / 10.6);
                let ao = 0.35 + 0.65 * smooth(cup_r(0.0) - 0.1, cup_r(0.0) + 1.4, p);
                plot(x, y, z, n, 0.78 * side * ao * if shadowed(x, y, z) { 0.35 } else { 1.0 }, 0.25 * glint(n, 12), Part::Saucer, false);
                a += 0.05 / p.max(0.5);
            }
            f += 0.02 / len;
        }
    }
    // The cup, outside and in.
    let mut hgt = 0.0;
    while hgt <= H {
        let r = cup_r(hgt);
        let dr = (cup_r(hgt + 0.05) - cup_r(hgt - 0.05)) / 0.1;
        let mut a = 0.0;
        while a < TAU {
            let (ca, sa) = (a.cos(), a.sin());
            if sa > -0.3 {
                let n = norm(ca, -dr, sa);
                plot(r * ca, hgt, r * sa, n, 0.95, 0.35 * glint(n, 10), Part::Cup, false);
            }
            if hgt > HC && sa < 0.3 {
                plot((r - 0.4) * ca, hgt, (r - 0.4) * sa, norm(-ca, dr, -sa), 0.75, 0.0, Part::Cup, false);
            }
            a += 0.06 / r;
        }
        hgt += 0.03;
    }
    // The lip, lit as one rounded edge facing out and up.
    let rr = cup_r(H);
    let mut v = 0.0;
    while v <= PI {
        let mut a = 0.0;
        while a < TAU {
            let (p, y) = (rr - 0.2 - 0.2 * v.cos(), H + 0.12 * v.sin());
            plot(p * a.cos(), y, p * a.sin(), norm(0.6 * a.cos(), 0.8, 0.6 * a.sin()), 0.95, 0.12, Part::Cup, true);
            a += 0.04 / rr;
        }
        v += 0.12;
    }
    // The coffee: a dark pool, a paler ring of crema at the wall, one glint of the light.
    let rc = cup_r(HC) - 0.4;
    let mut p = 0.0;
    while p < rc {
        let mut a = 0.0;
        while a < TAU {
            let (x, z) = (p * a.cos(), p * a.sin());
            let shine = (-(((x + 2.4) / 1.1).powi(2) + ((z + 1.4) / 0.8).powi(2))).exp();
            let crema = smooth(0.82 * rc, rc, p);
            let paint = if shine > 0.5 {
                SHINE
            } else if crema > 0.5 {
                CREMA
            } else {
                COFFEE
            };
            plot(x, HC, z, [0.0, 1.0, 0.0], 0.04 + 0.16 * crema, 0.85 * shine, Part::Coffee(paint), false);
            a += 0.05 / p.max(0.5);
        }
        p += 0.05;
    }
    // One thin dark line where the cup stands in front of the saucer.
    let foot = to_cell(0.0, 0.0, 0.0).1 * 4.0;
    let was = still.clone();
    for k in w..w * hh - 1 {
        let Some((v, Part::Saucer)) = was[k] else { continue };
        let cup = |n: usize| matches!(was[n], Some((_, Part::Cup)));
        if cup(k - w) && (k / w) as f64 > foot - 2.0 {
            still[k] = Some((v.min(0.06), Part::Saucer));
        } else if cup(k - 1) || cup(k + 1) {
            still[k] = Some((v * 0.4, Part::Saucer));
        }
    }
    // The handle: a thin loop on the right, standing out from the wall, each
    // dot near its line lit by the tube's own normal. Through its loop the
    // saucer is in shade, so the hole reads open.
    let (hy, ax, ay, tube) = (4.1, 3.9, 2.6, 0.42);
    let x0 = cup_r(hy) - 1.3;
    let ring: Vec<(f64, f64)> = (0..=210).map(|i| -2.1 + f64::from(i) * 0.02).map(|u| (x0 + ax * u.cos(), hy + ay * u.sin())).collect();
    #[allow(clippy::needless_range_loop)] // the index reads several arrays at once
    for k in 0..w * hh {
        let (c, r) = ((k % w) as f64 / 2.0 + 0.25, (k / w) as f64 / 4.0 + 0.125);
        let (x, y) = ((c - cx) / 2.0, (base - r) / cs);
        if x < cup_r(y) - 0.1 || x > x0 + ax + 1.0 {
            continue;
        }
        let (mut best, mut ox, mut oy) = (9.0, 0.0, 0.0);
        for &(lx, ly) in &ring {
            let d = (x - lx).hypot(y - ly);
            if d < best {
                (best, ox, oy) = (d, (x - lx) / tube, (y - ly) / tube);
            }
        }
        if best < tube + 0.06 {
            let n = norm(ox, oy, (1.0 - ox * ox - oy * oy).max(0.05).sqrt());
            still[k] = Some(((0.1 + 0.85 * dot3(n, l).max(0.0) + 0.3 * glint(n, 10)).min(1.0), Part::Cup));
        } else if (x - x0).powi(2) / (ax * ax) + (y - hy).powi(2) / (ay * ay) < 1.0 && !matches!(still[k], Some((_, Part::Cup))) {
            still[k] = None;
        }
    }
    // The lip reads as one bright line.
    #[allow(clippy::needless_range_loop)] // the index reads several arrays at once
    for k in 0..w * hh {
        if lip[k] {
            if let Some((v, part)) = still[k] {
                still[k] = Some((v.max(0.85), part));
            }
        }
    }
    // The steam may cross a cell with nothing in it, or only coffee.
    let free = (0..META.cols * META.rows)
        .map(|cell| {
            let (c, r) = (cell % META.cols, cell / META.cols);
            (0..8).all(|q| matches!(still[(r * 4 + q / 2) * w + c * 2 + q % 2], None | Some((_, Part::Coffee(_)))))
        })
        .collect();
    let starts = WISPS.iter().map(|&(x, z, ..)| to_cell(x, HC, z)).collect();
    Box::new(Coffee { still, free, starts, porcelain, steam: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Coffee {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let w = self.porcelain.width();
        self.porcelain.clear();
        for (k, s) in self.still.iter().enumerate() {
            let Some((v, part)) = *s else { continue };
            let v = v.clamp(0.0, 1.0);
            let (color, ink) = match part {
                Part::Cup => (GLAZE.at(v), 0.35 + 0.65 * v),
                Part::Saucer => (PORCELAIN.at(v), 0.3 + 0.7 * v),
                Part::Coffee(c) => (c, if c == COFFEE { 0.18 } else { 0.7 }),
            };
            if dither(k % w, k / w, if env.paper { 1.05 - ink } else { ink }) {
                self.porcelain.set((k % w) as i64, (k / w) as i64, color);
            }
        }
        // Where a wisp is at height s rows: a bend that rides up it and grows as it climbs.
        self.steam.clear();
        for (i, &(_, _, len, ph, lean)) in WISPS.iter().enumerate() {
            let (c0, r0) = self.starts[i];
            let mut s = 0.0;
            while s < len {
                let f = s / len;
                let wv = TAU * t / PERIOD;
                let c = c0 + 2.0 * (lean * s + (0.2 + 1.7 * f.powf(1.5)) * (0.75 * s - 2.0 * wv + ph).sin() + 0.3 * f * (wv + ph).sin());
                let r = r0 - s;
                // Faint off the surface and at the top, with breaks that rise with it.
                let thin = 0.5 + 0.5 * (0.55 * s - wv + ph * 2.0).sin();
                let a = smooth(0.3, 1.8, s) * (1.0 - smooth(0.6 * len, len, s)) * (0.3 + 0.7 * thin);
                let (x, y) = ((c * 2.0).floor() as i64, (r * 4.0).floor() as i64);
                let cell = (y.max(0) / 4) as usize * META.cols + (x / 2).max(0) as usize;
                if a > 0.2
                    && y >= 0
                    && x >= 0
                    && (x / 2) < META.cols as i64
                    && self.free.get(cell).copied().unwrap_or(false)
                    && dither(x as usize, y as usize, a * 1.3)
                {
                    self.steam.set(x, y, STEAM);
                }
                s += 0.06;
            }
        }
        grid.clear();
        self.porcelain.draw(grid, 0, 0);
        self.steam.draw(grid, 0, 0);
    }
}
