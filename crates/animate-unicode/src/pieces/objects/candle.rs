//! candle: a candle in a brass candlestick. The flame flickers and sways,
//! its glow lighting the wax and the metal, and the candle burns slowly down;
//! when it gutters out the stub is lifted away and a fresh one is set in and
//! lit. After ascii.rest's candle.ts (MIT, @bas3line).
//!
//! The candlestick turned on a lathe, the wax, its drips, the flame and its
//! light are ascii.rest's, worked out eight times a cell as there; where
//! ascii.rest averages each cell into a character, this draws every sample as
//! an octant dot, dithered by its light. In colour the stick is brass, lit
//! gold by the flame and glinting, the wax is cream glowing warm near the
//! top, the flame is a teardrop with a blue root and a white-gold heart, its
//! soft glow is a scatter of warm dots, and the smoke is grey.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{PI, TAU};

/// The candle's meta.
pub static META: Meta = Meta {
    name: "candle",
    slug: "candle",
    category: Category::Objects,
    note: "an octant candle in brass, its flame lighting the wax, burning down",
    cols: 40,
    rows: 30,
    fps: 15,
    charset: Charset::Extended,
    // Brass, dark to glinting; wax, shadow to glowing; the flame's root, its
    // body, its heart; the glow; the wick; smoke.
    palette: Some(&[
        "#4a3414", "#7a5a22", "#b08a3a", "#e0bc62", "#fff0b4", "#5c4a3c", "#a48a70", "#e6d2b0", "#fff2d6", "#3a5ad0", "#f2902a", "#ffd25a", "#fffbe8",
        "#c8742a", "#1a1410", "#6a6a72",
    ]),
    ground: Some("#0c0a08"),
    ..Meta::DEFAULT
};

const BRASS: Ramp = Ramp::new(0, 5);
const WAX: Ramp = Ramp::new(5, 4);
const ROOT: u8 = 9;
const FLAME: Ramp = Ramp::new(10, 3);
const GLOW: u8 = 13;
const WICK: u8 = 14;
const SMOKE: u8 = 15;
/// How steeply we look down on it, the candle's radius in rows, its height
/// new and spent above the socket, and where it stands in the holder.
const SN: f64 = 0.4;
const R: f64 = 2.1;
const FULL: f64 = 9.6;
const STUB: f64 = 1.2;
const SOCKET: f64 = 11.0;
/// The cycle in seconds: lit, burning down, guttering, smoke, the stub out,
/// a new one in, a breath; and where frame 0 falls, a little way down.
const STEPS: [f64; 7] = [1.2, 26.0, 1.4, 2.4, 1.6, 2.2, 0.5];
const T0: f64 = 4.6;
/// The candlestick, foot to socket, as radius and height round its axis.
const STICK: [(f64, f64); 28] = [
    (4.4, 0.0),
    (4.6, 0.3),
    (4.4, 0.65),
    (3.6, 0.95),
    (2.4, 1.25),
    (1.35, 1.6),
    (0.82, 2.1),
    (0.66, 3.0),
    (0.7, 4.1),
    (1.05, 4.45),
    (1.5, 4.9),
    (1.6, 5.4),
    (1.45, 5.9),
    (1.0, 6.2),
    (0.66, 6.5),
    (0.6, 7.6),
    (0.6, 8.7),
    (1.0, 9.1),
    (2.4, 9.55),
    (3.8, 9.75),
    (4.3, 10.0),
    (4.35, 10.3),
    (4.1, 10.35),
    (3.5, 10.1),
    (2.7, 10.05),
    (2.45, 10.1),
    (2.45, 11.15),
    (2.25, 11.3),
];
/// Drips: angle round the front, longest run in rows, seconds into the burn
/// they start, and fatness.
const DRIPS: [(f64, f64, f64, f64); 8] = [
    (-1.3, 4.5, 0.5, 0.42),
    (0.45, 3.0, 2.0, 0.34),
    (1.32, 5.0, 3.5, 0.42),
    (-0.5, 2.2, 6.0, 0.32),
    (0.95, 3.6, 10.0, 0.36),
    (-1.05, 4.0, 14.0, 0.38),
    (0.15, 2.6, 18.0, 0.3),
    (1.25, 3.0, 21.0, 0.38),
];

fn hash(n: i64) -> f64 {
    let mut h = ((n as u32) ^ 0x5bd1_e995).wrapping_mul(0x27d4_eb2d);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

fn noise(x: f64) -> f64 {
    let i = x.floor();
    let u = x - i;
    let e = u * u * (3.0 - 2.0 * u);
    hash(i as i64) + (hash(i as i64 + 1) - hash(i as i64)) * e
}

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

fn ease(u: f64) -> f64 {
    let u = clamp(u);
    u * u * (3.0 - 2.0 * u)
}

/// The candle: the candlestick's nearest surface at each dot (depth, then
/// position and normal), and the dots.
pub struct Candle {
    depth: Vec<f32>,
    geo: Vec<[f32; 6]>,
    dots: ColorOctants,
}

/// A candle; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    let cs = (1.0 - SN * SN).sqrt();
    let (cx, base) = (META.cols as f64 / 2.0, META.rows as f64 - 2.4);
    let mut depth = vec![-1e9f32; w * h];
    let mut geo = vec![[0f32; 6]; w * h];
    // The candlestick, turned on a lathe: the nearest surface behind each
    // dot, with its position and normal, so the flame can light it each frame.
    for s in STICK.windows(2) {
        let ((r1, y1), (r2, y2)) = (s[0], s[1]);
        let len = (r2 - r1).hypot(y2 - y1);
        let (nr, ny) = ((y2 - y1) / len, -(r2 - r1) / len);
        let mut f = 0.0;
        while f <= 1.0 {
            let (r, y) = (r1 + (r2 - r1) * f, y1 + (y2 - y1) * f);
            let mut a = 0.0;
            while a < TAU {
                let (x, z) = (r * a.cos(), r * a.sin());
                let (px, py) = (((cx + 2.0 * x) * 2.0).floor(), ((base - (y * cs - z * SN)) * 4.0).floor());
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let k = py as usize * w + px as usize;
                    let d = (z * cs + y * SN) as f32;
                    if d > depth[k] {
                        depth[k] = d;
                        geo[k] = [x as f32, y as f32, z as f32, (nr * a.cos()) as f32, ny as f32, (nr * a.sin()) as f32];
                    }
                }
                a += 0.08 / r.max(0.5);
            }
            f += 0.03 / len;
        }
    }
    Box::new(Candle { depth, geo, dots })
}

impl Piece for Candle {
    fn frame(&mut self, time: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let cs = (1.0 - SN * SN).sqrt();
        let (cx, base) = (META.cols as f64 / 2.0, META.rows as f64 - 2.4);
        let row_of = |y: f64, z: f64| base - (y * cs - z * SN);
        let key = [-0.45, 0.35, 0.82]; // the room's light, from the front left
        let km = (key[0] * key[0] + (key[1] + SN).powi(2) + (key[2] + cs).powi(2)).sqrt();
        let kh = [key[0] / km, (key[1] + SN) / km, (key[2] + cs) / km];
        // Where in the cycle we are: how far burnt, the flame's strength, the
        // candle's lift and fade.
        let cycle: f64 = STEPS.iter().sum();
        let mut k0 = (T0 + time).rem_euclid(cycle);
        let mut step = 0;
        while k0 > STEPS[step] {
            k0 -= STEPS[step];
            step += 1;
        }
        let u = k0 / STEPS[step];
        let (mut tb, mut flame, mut flare, mut smoke, mut lift, mut fade) = (0.0, 1.0, 0.0, -1.0, 0.0, 1.0);
        match step {
            0 => {
                flame = ease(u * 4.0);
                flare = (PI * clamp(u * 1.6)).sin() * (1.0 - u);
            }
            1 => tb = k0,
            _ => {
                tb = STEPS[1];
                flame = if step == 2 { (1.0 - u) * (0.6 + 0.4 * noise(time * 14.0)) } else { 0.0 };
                if step == 3 {
                    smoke = k0;
                }
                if step == 4 {
                    lift = 4.5 * ease(u);
                    fade = 1.0 - ease(u * 1.15);
                    smoke = STEPS[3] + k0;
                }
                if step == 5 {
                    lift = 13.0 * (1.0 - (PI / 2.0 * u).sin());
                }
                if step >= 5 {
                    tb = 0.0;
                }
            }
        }
        let hgt = FULL - (FULL - STUB) * (tb / STEPS[1]);
        let (y_top, ys) = (SOCKET + hgt + lift, SOCKET - 0.6 + lift);
        // The flame: its height, lean and brightness wander, with a gust now and then.
        let gust = clamp((noise(time * 0.45 + 3.0) - 0.62) * 3.0);
        let hf = flame * (4.3 + 1.1 * noise(time * 2.3) - 1.3 * gust * noise(time * 9.0 + 1.0)) * (1.0 + 0.7 * flare);
        let sway = 0.4 * (noise(time * 1.2 + 7.0) - 0.5) + 0.25 * gust * (time * 11.0).sin();
        let bright = (flame * (0.75 + 0.25 * noise(time * 3.1 + 11.0)) + 0.4 * flare).min(1.3);
        let (fx, fy) = (sway * 0.3, y_top + 0.4 + hf * 0.32); // its heart, where the light comes from
                                                              // The wax: lit by the room, glowing through near the top; drips are raised runs.
        let wax = |nx: f64, nz: f64, below: f64| {
            let k = (nx * key[0] + nz * key[2]).max(0.0);
            let glow = (0.55 * (-below / 2.2).exp() + 0.25 * (-below / 0.6).exp()) * bright;
            (0.05 + 0.7 * k * (0.55 + 0.45 * bright.min(1.0)) + glow * (0.5 + 0.5 * nz), glow)
        };
        let drips: Vec<(f64, f64, f64, f64, f64)> = DRIPS
            .iter()
            .filter_map(|&(da, dl, start, fat)| {
                let y0 = SOCKET + FULL - (FULL - STUB) * (start / STEPS[1]) + lift - 0.3;
                let run = dl.min((tb - start) * 0.8);
                (run > 0.0 && y0 - run <= y_top - 0.2).then(|| ((R + 0.05) * da.sin(), (R + 0.05) * da.cos(), ys.max(y0 - run), y0.min(y_top - 0.15), fat))
            })
            .collect();
        self.dots.clear();
        // The light at every dot, and whether it is brass or wax.
        let mut solid = vec![false; w * h];
        #[allow(clippy::needless_range_loop)] // the index reads several arrays at once
        for k in 0..w * h {
            let (sx, sy) = ((k % w) as f64 / 2.0 + 0.25, (k / w) as f64 / 4.0 + 0.125);
            let x = (sx - cx) / 2.0;
            let mut d = f64::from(self.depth[k]);
            let mut paint: Option<(f64, u8)> = None;
            if d > -1e8 {
                let [gx, gy, gz, nx, ny, nz] = self.geo[k].map(f64::from);
                let (lx, ly, lz) = (fx - gx, fy - gy, -gz);
                let ld = (lx * lx + ly * ly + lz * lz).sqrt();
                let diff = ((nx * lx + ny * ly + nz * lz) / ld).max(0.0);
                // Brass: the flame's light and its glint, and the room's from the left, glinting too.
                let (hx, hy, hz) = (lx / ld, ly / ld + SN, lz / ld + cs);
                let hm = (hx * hx + hy * hy + hz * hz).sqrt();
                let spec = ((nx * hx + ny * hy + nz * hz) / hm).max(0.0).powi(6);
                let kd = (nx * key[0] + ny * key[1] + nz * key[2]).max(0.0);
                let kspec = (nx * kh[0] + ny * kh[1] + nz * kh[2]).max(0.0).powi(6);
                let v = 0.08 + (0.4 * diff + 0.7 * spec) * bright / (1.0 + (ld / 14.0).powi(2)) + 0.16 * kd + 0.85 * kspec;
                paint = Some((v, BRASS.at(v)));
            }
            if fade > 0.02 && x.abs() < R + 0.4 {
                let z = (R * R - x * x).max(0.0).sqrt();
                let yb = (base - sy + z * SN) / cs; // the body's front, here
                let zt = (sy - base + y_top * cs) / SN; // the top's plane, here
                if x * x + zt * zt < R * R && zt * cs + y_top * SN > d {
                    // The top: a pool of melted wax lit by the flame, its rim brightest.
                    let rim = x.hypot(zt) / R;
                    let v = (0.3 + 0.35 * bright + 0.25 * bright * rim.powi(4)) * fade;
                    paint = Some((v, WAX.at(0.5 + 0.5 * bright.min(1.0))));
                    d = zt * cs + y_top * SN;
                } else if x.abs() < R && yb < y_top && yb > ys && z * cs + yb * SN > d {
                    let (mut v, glow) = wax(x / R, z / R, y_top - yb);
                    // A drip throws a thin shadow on the wax to its right.
                    if drips.iter().any(|&(dx, _, y1, y0, fat)| x > dx + fat * 0.9 && x < dx + fat + 0.32 && yb < y0 && yb > y1 - 0.2) {
                        v *= 0.55;
                    }
                    paint = Some((v * fade, WAX.at(0.15 + 0.5 * v + glow)));
                    d = z * cs + yb * SN;
                }
                // The drips, each a rounded run down the side, standing out from it.
                for &(dcx, dcz, y1, y0, fat) in &drips {
                    let dx = x - dcx;
                    if dx.abs() >= fat {
                        continue;
                    }
                    let dz = (fat * fat - dx * dx).sqrt();
                    let zz = dcz + dz;
                    let yy = (base - sy + zz * SN) / cs;
                    if (yy < y1 + fat && dx * dx + (y1 + fat - yy).powi(2) > fat * fat) || yy > y0 || zz * cs + yy * SN <= d - 0.02 {
                        continue;
                    }
                    let (v, glow) = wax(dx / fat, dz / fat, y_top - yy);
                    paint = Some(((v + 0.1) * fade, WAX.at(0.25 + 0.5 * v + glow)));
                    d = zz * cs + yy * SN;
                }
            }
            if let Some((v, color)) = paint {
                solid[k] = true;
                let v = clamp(v);
                if dither(k % w, k / w, if env.paper { 1.05 - v } else { 0.15 + 0.9 * v }) {
                    self.dots.set((k % w) as i64, (k / w) as i64, color);
                }
            }
        }
        let (fc, fr) = (cx + 2.0 * fx, row_of(fy, 0.0));
        // A soft glow round the flame, a thin random scatter of warm dots kept
        // clear of the solid.
        if bright > 0.05 {
            #[allow(clippy::needless_range_loop)] // the index reads several arrays at once
            for k in 0..w * h {
                let (sx, sy) = ((k % w) as f64 / 2.0 + 0.25, (k / w) as f64 / 4.0 + 0.125);
                let g = ((sx - fc) / 2.0 / 1.05).hypot((sy - fr) / 1.35) / (2.4 + 1.4 * bright.min(1.0));
                if g < 1.0 && !solid[k] && hash(k as i64 * 7 + 3) < 0.2 * (1.0 - g) * bright.min(1.0) {
                    self.dots.set((k % w) as i64, (k / w) as i64, GLOW);
                }
            }
        }
        // A teardrop on the wick, swaying at the tip, with a blue root and the
        // wick dark in its heart.
        if hf > 0.2 {
            let y0 = y_top + 0.1;
            for j in ((row_of(y0 + hf + 0.5, 0.0)) * 4.0).floor().max(0.0) as i64..((row_of(y0 - 0.5, 0.0)) * 4.0).ceil().min(h as f64) as i64 {
                for i in ((fc - 5.0) * 2.0).max(0.0) as i64..((fc + 5.0) * 2.0).min(w as f64) as i64 {
                    let y = (base - (j as f64 + 0.5) / 4.0) / cs - y0 + 0.35;
                    let uu = y / (hf + 0.35);
                    if uu <= 0.0 || uu >= 1.0 {
                        continue;
                    }
                    let wd = (1.2 + 0.25 * flare) * (uu.sqrt() * (1.0 - uu).powf(1.2)) / 0.36;
                    let q = (((i as f64 + 0.5) / 2.0 - cx) / 2.0 - fx - sway * uu * uu) / wd;
                    if q.abs() >= 1.0 {
                        continue;
                    }
                    let color = if uu < 0.18 {
                        ROOT
                    } else if uu < 0.32 && q.abs() < 0.35 {
                        WICK
                    } else {
                        FLAME.at((1.0 - q * q) * (0.5 + 0.6 * (uu * 2.5).min(1.0)))
                    };
                    self.dots.set(i, j, color);
                }
            }
        }
        // The wick, standing bare once the flame is out.
        let (wr, wc) = ((row_of(y_top + 0.35, 0.0) * 4.0) as i64, (cx * 2.0) as i64);
        if hf <= 0.2 && fade > 0.5 {
            self.dots.set(wc, wr, WICK);
            self.dots.set(wc, wr - 1, WICK);
        }
        // Once it is out, a thread of smoke from the wick, thinning as it climbs.
        if smoke >= 0.0 {
            let mut s = 1.5;
            while s < 11.0 {
                let r = row_of(y_top - lift + 0.4, 0.0) - s;
                let age = smoke - s / 4.0;
                if r >= 0.0 && (0.0..=2.0).contains(&age) {
                    let x = cx + 0.4 + 0.5 * (s / 3.0) * (0.9 * s - smoke * 3.0).sin();
                    if age < 1.4 || dither((x * 2.0) as usize, (r * 4.0) as usize, 0.5) {
                        self.dots.set((x * 2.0) as i64, (r * 4.0) as i64, SMOKE);
                    }
                }
                s += 0.25;
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
