//! gyroscope: a flywheel hung in three gimbal rings on a post and foot. The
//! outer ring turns and the gimbals swing so the rotor's axle keeps its line.
//! After ascii.rest's gyroscope.ts (MIT, @bas3line).
//!
//! In braille the 60×32 cells are a 120×128 picture, z-buffered per dot and
//! shaded by ordered dithering. Each gimbal is its own metal (gold, silver,
//! copper) round a steel rotor, so the eye can follow one ring through its
//! swing, and where one part passes in front of another the farther one
//! breaks off short of it.

use crate::geom::{unit, M3, V3};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The gyroscope's meta.
pub static META: Meta = Meta {
    name: "gyroscope",
    slug: "gyroscope",
    category: Category::Shapes,
    note: "gold, silver and copper gimbals swing round a spinning steel rotor",
    cols: 60,
    rows: 32,
    fps: 30,
    charset: Charset::Extended,
    // Four shades each, dimmest first: the stand and pins, the outer ring
    // (gold), the middle (silver), the inner (copper), the rotor (steel).
    palette: Some(&[
        "#33363c", "#5c6068", "#8c9098", "#c8ccd4", "#4a3a10", "#8a6a1a", "#c89a2a", "#f4d27a", "#3c4048", "#6c727c", "#a4aab4", "#e4e8ee", "#4a2414",
        "#8a4424", "#c46a3c", "#f0a07a", "#1c2c40", "#2c4a6c", "#44709c", "#8cb4dc",
    ]),
    ground: Some("#0b0c10"),
    ..Meta::DEFAULT
};

/// Eye to centre, and columns per unit at the centre.
const K2: f64 = 8.0;
const K1: f64 = 22.0 * K2;
/// Ring radii (outer, middle, inner) and their tube.
const RO: f64 = 1.0;
const RM: f64 = 0.74;
const RI: f64 = 0.5;
const TUBE: f64 = 0.065;
/// Flywheel radius, half thickness, and its rounded rim as a fraction.
const ROTOR: f64 = 0.35;
const HALF: f64 = 0.07;
const RIM: f64 = 0.26;
/// Steps round a tube.
const N: usize = 24;

/// What draws into the picture: the transform, which part it is, a gain for
/// polished pins, the dots, and which part drew each dot.
struct Pen<'a> {
    m: M3,
    id: u8,
    gain: f64,
    w: usize,
    h: usize,
    light: V3,
    half: V3,
    dots: &'a mut ColorDots,
    part: &'a mut [u8],
}

impl Pen<'_> {
    /// A point and its normal in the current part's frame; `k` scales its light.
    fn plot(&mut self, p: V3, n: V3, k: f64) {
        let [x, y, z] = self.m.apply(p);
        let ooz = 1.0 / (K2 + z);
        let col = (self.w as f64 / 2.0 + 2.0 * K1 * ooz * x).floor() as i64;
        let row = (self.h as f64 / 2.0 - 8.0 - 2.0 * K1 * ooz * y).floor() as i64;
        let mut nn = unit(self.m.apply(n));
        if nn[2] > 0.0 {
            nn = nn.map(|c| -c); // light whichever side faces us
        }
        let diff = ((nn[0] * self.light[0] + nn[1] * self.light[1] + nn[2] * self.light[2] + 1.0) / 2.0).powi(2);
        let spec = (nn[0] * self.half[0] + nn[1] * self.half[1] + nn[2] * self.half[2]).max(0.0).powi(24);
        let near = 0.55 + 0.45 * (0.5 - 0.6 * z).clamp(0.0, 1.0);
        let b = ((0.08 + 0.7 * diff + 0.45 * spec) * near * k * self.gain).min(1.0);
        let shade = ((b * 4.0) as u8).min(3);
        if self.dots.plot(col, row, ooz as f32, self.id * 4 + shade) {
            self.part[row as usize * self.w + col as usize] = self.id;
        }
    }

    /// A torus of radius `r` in the local xy plane.
    fn ring(&mut self, r: f64, steps: usize) {
        for i in 0..steps {
            let (su, cu) = (i as f64 / steps as f64 * TAU).sin_cos();
            for j in 0..N {
                let (sv, cv) = (j as f64 / N as f64 * TAU).sin_cos();
                let hh = r + TUBE * cv;
                self.plot([hh * cu, hh * su, TUBE * sv], [cv * cu, cv * su, sv], 1.0);
            }
        }
    }

    /// A pin of radius `r` along the local x (axis 0) or y (axis 1), from a to b.
    fn pin(&mut self, axis: usize, a: f64, b: f64, r: f64) {
        let (id, gain) = (self.id, self.gain);
        self.id = 0;
        self.gain = 1.5;
        let n = ((b - a).abs() / 0.01).ceil() as usize;
        for i in 0..=n {
            let s = a + (b - a) * i as f64 / n as f64;
            for j in 0..N {
                let (sv, cv) = (j as f64 / N as f64 * TAU).sin_cos();
                if axis == 0 {
                    self.plot([s, r * cv, r * sv], [0.0, cv, sv], 1.0);
                } else {
                    self.plot([r * cv, s, r * sv], [cv, 0.0, sv], 1.0);
                }
            }
        }
        (self.id, self.gain) = (id, gain);
    }

    /// A flat disc lying level, centred at height y: top face and rim.
    fn disc(&mut self, y: f64, r: f64, h: f64) {
        for k in 0..240 {
            let (sa, ca) = (k as f64 / 240.0 * TAU).sin_cos();
            let mut p = 0.0;
            while p <= r {
                self.plot([p * ca, y + h, p * sa], [0.12 * ca * (p / r), 1.0, 0.12 * sa * (p / r)], 1.0);
                p += 0.01;
            }
            let mut s = -h;
            while s <= h {
                self.plot([r * ca, y + s, r * sa], [ca, 0.0, sa], 1.0);
                s += 0.01;
            }
        }
    }

    /// The flywheel facing along local x: flat faces rounding off at the rim,
    /// one dark spoke so the spin shows, and the axle through its hub.
    fn rotor(&mut self) {
        for k in 0..320 {
            let th = k as f64 / 320.0 * TAU;
            let (sa, ca) = th.sin_cos();
            let spoke = th.min(TAU - th) < 0.22;
            let mut p = 0.0;
            while p <= ROTOR + 1e-9 {
                let f = p / ROTOR;
                let bev = if f > 1.0 - RIM { 0.9 * (f - 1.0 + RIM) / RIM } else { 0.0 };
                let shade = if spoke && f > 0.3 && f < 0.95 { 0.35 } else { 1.35 };
                self.plot([HALF, p * ca, p * sa], [1.0, bev * ca, bev * sa], shade);
                self.plot([-HALF, p * ca, p * sa], [-1.0, bev * ca, bev * sa], shade);
                p += 0.006;
            }
            let mut x = -HALF;
            while x <= HALF + 1e-9 {
                self.plot([x, ROTOR * ca, ROTOR * sa], [0.0, ca, sa], 1.0);
                x += 0.01;
            }
        }
        self.pin(0, -HALF - 0.05, HALF + 0.05, 0.065); // hub
        self.pin(0, -RI - 0.1, RI + 0.1, 0.05); // axle, through the inner ring's bearings
        self.pin(0, RI - 0.06, RI + 0.04, 0.07);
        self.pin(0, -RI - 0.04, -RI + 0.06, 0.07);
    }
}

/// The gyroscope: the rotor's fixed direction, which part drew each dot, the
/// dots, and the gaps to cut.
pub struct Gyroscope {
    axle: V3,
    part: Vec<u8>,
    gap: Vec<usize>,
    dots: ColorDots,
}

/// A gyroscope; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    Box::new(Gyroscope { axle: unit([-0.55, 0.42, -0.72]), part: vec![0; n], gap: vec![], dots })
}

impl Piece for Gyroscope {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        // The outer ring turns about the stand; the middle and inner rings take
        // whatever angles keep the axle on its fixed line.
        let a = 0.9 * ((t / 8.0 - 1.0 / 12.0) * TAU).sin();
        let (sa, ca) = a.sin_cos();
        let ax = self.axle;
        let (wx, wy, wz) = (ca * ax[0] - sa * ax[2], ax[1], sa * ax[0] + ca * ax[2]);
        let c = wx.clamp(-1.0, 1.0).acos();
        let b = wy.atan2(-wz);
        let view = M3::rx(-0.3); // looking down a little
        let mo = view.mul(&M3::ry(a));
        let mm = mo.mul(&M3::rx(b));
        let mi = mm.mul(&M3::ry(c));
        let mr = mi.mul(&M3::rx(t * 1.25 * TAU));
        let light = unit([-0.5, 0.8, -0.6]);
        let half = unit([light[0], light[1], light[2] - 1.0]);

        self.dots.clear();
        self.part.fill(0);
        let mut pen = Pen { m: view, id: 0, gain: 1.0, w, h, light, half, dots: &mut self.dots, part: &mut self.part };
        // The stand: a knob on top, a post and a round foot below.
        pen.pin(1, RO, RO + 0.13, 0.04);
        pen.pin(1, RO + 0.13, RO + 0.2, 0.07);
        pen.pin(1, -RO - 0.13, -RO, 0.04);
        pen.pin(1, -1.4, -RO - 0.1, 0.055);
        pen.disc(-1.42, 0.12, 0.02);
        pen.disc(-1.46, 0.38, 0.025);
        pen.m = mo;
        pen.pin(0, RM, RO, 0.032);
        pen.pin(0, -RO, -RM, 0.032);
        pen.id = 1;
        pen.ring(RO, 900);
        pen.m = mm;
        pen.pin(1, RI, RM, 0.032);
        pen.pin(1, -RM, -RI, 0.032);
        pen.id = 2;
        pen.ring(RM, 700);
        pen.m = mi;
        pen.id = 3;
        pen.ring(RI, 520);
        pen.m = mr;
        pen.id = 4;
        pen.rotor();

        // Where one part passes in front of another, leave two blank dots
        // beside the nearer one so each ring keeps its own edge.
        self.gap.clear();
        for k in 0..w * h {
            let (x, y) = ((k % w) as i64, (k / w) as i64);
            if self.dots.get(x, y).is_none() || self.part[k] == 0 {
                continue;
            }
            let here = self.dots.depth(x, y);
            for dx in [-2i64, -1, 1, 2] {
                let nx = x + dx;
                if nx < 0 || nx >= w as i64 {
                    continue;
                }
                let n = k as i64 + dx;
                let p = self.part[n as usize];
                if p != 0 && p != self.part[k] && self.dots.get(nx, y).is_some() && self.dots.depth(nx, y) > here + 0.0008 {
                    self.gap.push(k);
                    break;
                }
            }
        }
        for &k in &self.gap {
            self.dots.unset((k % w) as i64, (k / w) as i64);
        }
        let paper = env.paper;
        self.dots.dither_by(|c| {
            let b = f64::from(c % 4) / 3.0;
            if paper {
                0.95 - 0.6 * b
            } else {
                0.35 + 0.6 * b
            }
        });
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
