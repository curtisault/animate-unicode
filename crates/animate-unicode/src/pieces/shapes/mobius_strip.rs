//! mobius strip: a band with one half twist, spinning about its axis while it
//! slowly nods. Z-buffered, lit from above. After ascii.rest's
//! mobius-strip.ts (MIT, @bas3line).
//!
//! In braille the 58×20 cells are a 116×80 picture, shaded by ordered
//! dithering. The band's one face is painted with a rainbow that runs the
//! whole length of that face, which is twice round the loop: follow a colour
//! and it passes through the twist onto the "other" side and comes back to
//! itself only after the second lap, so the strip shows it has one side.

use crate::geom::{dot, unit};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The Möbius strip's meta.
pub static META: Meta = Meta {
    name: "mobius strip",
    slug: "mobius-strip",
    category: Category::Shapes,
    note: "a mobius strip whose rainbow runs twice round its one face",
    cols: 58,
    rows: 20,
    fps: 30,
    charset: Charset::Extended,
    // Six hues round the colour wheel, each in four shades, dimmest first.
    palette: Some(&[
        "#5a1a22", "#8f2a34", "#c8404a", "#f0727a", "#5a3a12", "#8f5e1c", "#c98a2a", "#f2b85a", "#4a5212", "#76821c", "#a8b82c", "#d8e670", "#124a2c",
        "#1c7646", "#2ca866", "#68dc9a", "#123a5a", "#1c5c8f", "#2c84c8", "#6ab4f0", "#3a1a5a", "#5c2a8f", "#8440c8", "#b27af0",
    ]),
    ground: Some("#0a0a10"),
    ..Meta::DEFAULT
};

/// Half the band's width, the ring's radius being 1.
const W: f64 = 0.46;
/// Samples round the loop and across the band: enough to cover every dot.
const NU: usize = 1800;
const NV: usize = 72;
/// Eye to centre.
const D: f64 = 6.0;
const HUES: usize = 6;
const SHADES: usize = 4;
/// A depth step this big, between parts far round the loop, is one layer over another.
const GAP: f32 = 0.3;

/// The strip: its points and normals (built once), how far round each is,
/// the screen scale, and the dots.
pub struct MobiusStrip {
    p: Vec<[f32; 3]>,
    n: Vec<[f32; 3]>,
    k: f64,
    along: Vec<u16>,
    cut: Vec<bool>,
    dots: ColorDots,
}

/// Spin about the ring's axis, which nods between 0.8 and 1.2 rad from the line of sight.
fn view(t: f64) -> (f64, f64) {
    (2.6 + t * 0.35, 1.0 - 0.2 * (t * 0.3).cos())
}

/// The strip's projected bounds as `[centre x, centre y, half width, half height]`.
fn bounds(p: &[[f32; 3]], s: f64, tilt: f64) -> [f64; 4] {
    let ((ss, cs), (st, ct)) = (s.sin_cos(), tilt.sin_cos());
    let (mut x0, mut x1, mut y0, mut y1) = (9.0f64, -9.0f64, 9.0f64, -9.0f64);
    for q in p.iter().step_by(11) {
        let (px, py, pz) = (f64::from(q[0]), f64::from(q[1]), f64::from(q[2]));
        let (x, w) = (cs * px - ss * py, ss * px + cs * py);
        let (y, z) = (ct * w + st * pz, -st * w + ct * pz);
        let (sx, sy) = (x / (D - z), y / (D - z));
        x0 = x0.min(sx);
        x1 = x1.max(sx);
        y0 = y0.min(sy);
        y1 = y1.max(sy);
    }
    [(x0 + x1) / 2.0, (y0 + y1) / 2.0, (x1 - x0) / 2.0, (y1 - y0) / 2.0]
}

/// A Möbius strip; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (mut p, mut n) = (Vec::with_capacity(NU * NV), Vec::with_capacity(NU * NV));
    for i in 0..NU {
        let u = i as f64 / NU as f64 * TAU;
        let ((su, cu), (sh, ch)) = (u.sin_cos(), (u / 2.0).sin_cos());
        for j in 0..NV {
            let v = -W + 2.0 * W * j as f64 / (NV - 1) as f64;
            let h = 1.0 + v * ch;
            p.push([h * cu, h * su, v * sh].map(|x| x as f32));
            // The normal from the two tangents, d/du and d/dv.
            let du = [-(v / 2.0) * sh * cu - h * su, -(v / 2.0) * sh * su + h * cu, (v / 2.0) * ch];
            let dv = [ch * cu, ch * su, sh];
            n.push(unit([du[1] * dv[2] - du[2] * dv[1], du[2] * dv[0] - du[0] * dv[2], du[0] * dv[1] - du[1] * dv[0]]).map(|x| x as f32));
        }
    }
    // The strip is wider lying flat than standing up: size it from the widest it gets.
    let (mut ex, mut ey) = (0.0f64, 0.0f64);
    let mut s: f64 = 0.0;
    while s < TAU {
        let mut tilt: f64 = 0.8;
        while tilt <= 1.21 {
            let b = bounds(&p, s, tilt);
            ex = ex.max(b[2]);
            ey = ey.max(b[3]);
            tilt += 0.05;
        }
        s += 0.05;
    }
    let dots = ColorDots::new(META.cols, META.rows);
    let k = ((dots.width() as f64 - 6.0) / 2.0 / ex).min((dots.height() as f64 - 4.0) / 2.0 / ey);
    let size = dots.width() * dots.height();
    Box::new(MobiusStrip { p, n, k, along: vec![0; size], cut: vec![false; size], dots })
}

impl Piece for MobiusStrip {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let (s, tilt) = view(t);
        let [mx, my, _, _] = bounds(&self.p, s, tilt);
        let ((ss, cs), (st, ct)) = (s.sin_cos(), tilt.sin_cos());
        // A key light from above and a weak fill.
        let (key, fill) = (unit([-0.35, 0.8, 0.5]), unit([0.3, -0.3, 0.9]));
        let k = self.k;
        self.dots.clear();
        for (q, (p, n)) in self.p.iter().zip(&self.n).enumerate() {
            let (px, py, pz) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
            let (x, y0) = (cs * px - ss * py, ss * px + cs * py);
            let (y, z) = (ct * y0 + st * pz, -st * y0 + ct * pz);
            let ooz = 1.0 / (D - z);
            let col = (w as f64 / 2.0 + k * (ooz * x - mx)).floor() as i64;
            let row = (h as f64 / 2.0 - k * (ooz * y - my)).floor() as i64;
            let (nx, ny, nz) = (f64::from(n[0]), f64::from(n[1]), f64::from(n[2]));
            let (wx, w0) = (cs * nx - ss * ny, ss * nx + cs * ny);
            let mut nn = [wx, ct * w0 + st * nz, -st * w0 + ct * nz];
            // A band has no inside: light whichever side faces the eye, and
            // note which side that is.
            let back = nn[2] < 0.0;
            if back {
                nn = nn.map(|c| -c);
            }
            let lit = (0.06 + dot(nn, key).max(0.0) * 0.85 + dot(nn, fill).max(0.0) * 0.22).clamp(0.0, 1.0);
            // How far along the one face: round the loop once on one side,
            // then once more on the other, where the twist hands it over.
            let i = q / NV;
            let along_face = (i + if back { NU } else { 0 }) as f64 / (2 * NU) as f64;
            let hue = (along_face * HUES as f64) as usize % HUES;
            let shade = ((lit * SHADES as f64) as usize).min(SHADES - 1);
            if self.dots.plot(col, row, z as f32, (hue * SHADES + shade) as u8) {
                self.along[row as usize * w + col as usize] = i as u16;
            }
        }
        // Clear the dots of band just behind a part further round the loop
        // that passes in front, so the layers stay apart.
        for i in 0..w * h {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            let here = self.dots.depth(x, y);
            self.cut[i] = self.dots.get(x, y).is_some()
                && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                    let (nx, ny) = (x + dx, y + dy);
                    if self.dots.get(nx, ny).is_none() || self.dots.depth(nx, ny) <= here + GAP {
                        return false;
                    }
                    let s = self.along[ny as usize * w + nx as usize].abs_diff(self.along[i]) as usize;
                    s.min(NU - s) > NU / 12
                });
        }
        for i in 0..w * h {
            if self.cut[i] {
                self.dots.unset((i % w) as i64, (i / w) as i64);
            }
        }
        let paper = env.paper;
        self.dots.dither_by(|c| {
            let b = (c as usize % SHADES) as f64 / (SHADES - 1) as f64;
            if paper {
                0.9 - 0.65 * b
            } else {
                0.22 + 0.62 * b
            }
        });
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
