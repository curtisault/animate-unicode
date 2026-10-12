//! tunnel: the demo-scene tunnel. Each dot casts a ray down a tiled tube
//! that bends away out of sight, so the far end swings about, while the
//! tiles wind round it in a slow spiral and fade into the dark. After
//! ascii.rest's tunnel.ts (MIT, @bas3line).
//!
//! The tube, its bend and its tiling are ascii.rest's; a ray is cast for
//! every octant dot instead of every cell, the pattern averaged over each
//! dot's own footprint so the far tiles fade to an even grey rather than
//! shimmer. In colour the tiles alternate an ice blue and a deep blue, set
//! in dark mortar, and sink into a violet fog with depth.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{SQRT_2, TAU};

/// The tunnel's meta.
pub static META: Meta = Meta {
    name: "tunnel",
    slug: "tunnel",
    category: Category::Effects,
    note: "flying down an octant tube of blue tiles that bends into violet fog",
    cols: 64,
    rows: 24,
    fps: 30,
    charset: Charset::Extended,
    // Light tiles, fog to near; dark tiles, fog to near.
    palette: Some(&["#2a1c4a", "#3a4a9a", "#6aa4e8", "#c8ecff", "#1c143a", "#22307a", "#3a5ac8", "#6a8aff"]),
    ground: Some("#07050f"),
    ..Meta::DEFAULT
};

const LIGHT: Ramp = Ramp::new(0, 4);
const DARK: Ramp = Ramp::new(4, 4);
/// Focal length in columns; tiles round the tube; tiles per unit of depth;
/// the dark tiles' share of the light ones; the mortar's share of a light
/// tile; the most of a tile the mortar takes on each side; units of depth a
/// second; turns of the pattern per unit of depth; the depth over which the
/// light falls by e; the depth over which the tube turns to its far heading;
/// how much a lamp to the upper left favours the far wall; and the share of
/// a dot's footprint the pattern is averaged over.
const F: f64 = 18.0;
const AROUND: f64 = 10.0;
const ALONG: f64 = 0.55;
const DIM: f64 = 0.35;
const MORTAR: f64 = 0.1;
const GAP: f64 = 0.07;
const SPEED: f64 = 2.4;
const TWIST: f64 = 0.06;
const FOG: f64 = 2.8;
const BEND: f64 = 3.0;
const KEY: f64 = 0.12;
const SHARP: f64 = 0.7;

/// A square wave, and pulses of half-width g at the whole numbers, each
/// averaged over a span w, so tiles smaller than a dot fade to an even grey.
fn wrap(x: f64) -> f64 {
    x - x.floor()
}

fn wave(x: f64, w: f64) -> f64 {
    2.0 * ((wrap((x - w / 2.0) / 2.0) - 0.5).abs() - (wrap((x + w / 2.0) / 2.0) - 0.5).abs()) / w
}

fn ridge(x: f64, g: f64) -> f64 {
    (x + g).floor() * 2.0 * g + wrap(x + g).min(2.0 * g)
}

fn pulse(x: f64, w: f64, g: f64) -> f64 {
    (ridge(x + w / 2.0, g) - ridge(x - w / 2.0, g)) / w
}

/// The tunnel: each ray's place on the tube (round, along, depth, light), and the dots.
pub struct Tunnel {
    u: Vec<f64>,
    v: Vec<f64>,
    z: Vec<f64>,
    l: Vec<f64>,
    dots: ColorOctants,
}

/// A tunnel; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorOctants::new(META.cols, META.rows);
    let n = (dots.width() + 1) * (dots.height() + 1);
    Box::new(Tunnel { u: vec![0.0; n], v: vec![0.0; n], z: vec![0.0; n], l: vec![0.0; n], dots })
}

impl Piece for Tunnel {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (w, h) = (self.dots.width(), self.dots.height());
        let ww = w + 1; // one sample more right and below, for each dot's footprint
                        // Where the far end points, as a slope off the view axis, on a slow loop.
        let (ax, ay) = (0.42 * (t * 0.55).sin(), 0.26 * (t * 0.9 + 1.0).sin());
        let spin = t * 0.06;
        for j in 0..=h {
            for i in 0..=w {
                let sx = ((i as f64 + 0.5) / 2.0 - cols / 2.0) / F;
                let sy = -(((j as f64 + 0.5) / 4.0 - rows / 2.0) * 2.0) / F;
                // A ray hits the wall (radius 1) where its offset from the bent
                // axis is 1, the axis leaning toward (ax, ay) more with depth.
                let (mut z, mut dx, mut dy) = (1.0 / sx.hypot(sy).max(1e-3), sx, sy);
                for _ in 0..4 {
                    let lean = 1.0 - (-z / BEND).exp();
                    dx = sx - ax * lean;
                    dy = sy - ay * lean;
                    z = (1.0 / dx.hypot(dy).max(1e-3)).min(60.0);
                }
                let k = j * ww + i;
                self.u[k] = (dy.atan2(dx) / TAU + spin + z * TWIST) * AROUND;
                self.v[k] = z * ALONG + t * SPEED * ALONG;
                self.z[k] = z;
                // Light falls off down the tube, and the wall facing the lamp is brighter.
                self.l[k] = (-z / FOG).exp() * (1.0 - KEY + KEY * (dx - dy) * z / SQRT_2);
            }
        }
        // Going round, the seam is no jump.
        let turn = |d: f64| d - AROUND * (d / AROUND).round();
        self.dots.clear();
        for j in 0..h {
            for i in 0..w {
                let k = j * ww + i;
                let (u, v, z) = (self.u[k], self.v[k], self.z[k]);
                // How far the pattern moves across this dot, in tiles.
                let wu = SHARP * turn(self.u[k + 1] - u).abs().max(turn(self.u[k + ww] - u).abs()).max(0.01);
                let wv = SHARP * (self.v[k + 1] - v).abs().max((self.v[k + ww] - v).abs()).max(0.01);
                // Light tiles and dark ones, set in mortar lines about a dot wide.
                let light = 0.5 - 0.5 * wave(u, wu) * wave(v, wv);
                let gu = GAP.min(z * AROUND / (4.0 * std::f64::consts::PI * F * 2.0));
                let gv = GAP.min(z * z * ALONG / (F * 2.0));
                let mortar = 1.0 - (1.0 - pulse(u, wu, gu)) * (1.0 - pulse(v, wv, gv));
                let b = (((DIM + (1.0 - DIM) * light) * (1.0 - mortar) + MORTAR * mortar) * self.l[k] * 1.25).max(0.0).powf(0.8);
                let ink = if env.paper { 1.0 - b } else { b };
                if dither(i, j, ink) {
                    let fog = (self.l[k] * 1.6).min(1.0);
                    self.dots.set(i as i64, j as i64, if light > 0.5 { LIGHT.at(fog) } else { DARK.at(fog) });
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
