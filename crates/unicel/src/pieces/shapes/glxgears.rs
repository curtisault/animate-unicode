//! glxgears: the three gears of gears.c, with their sizes, places and view.
//! Each gear's face keeps one shade, as each has one colour there; the walls
//! of the teeth and the bores are bright where they catch the light. After
//! ascii.rest's glxgears.ts (MIT, @bas3line).
//!
//! In octants the 56×31 cells are a 112×124 picture of solid square dots, in
//! glxgears' own red, green and blue. A cell keeps only the dots of the part
//! that covers most of it, so the teeth mesh with sharp edges.

use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// The gears' meta.
pub static META: Meta = Meta {
    name: "glxgears",
    slug: "glxgears",
    category: Category::Shapes,
    note: "glxgears' red, green and blue gears meshing, in octants",
    cols: 56,
    rows: 31,
    fps: 24,
    charset: Charset::Extended,
    // Each gear: a wall in shadow, its face, a face a shade lighter, a lit wall.
    palette: Some(&["#4a0c0c", "#a61c1c", "#c92a2a", "#ff7a6a", "#0c3a12", "#1c8c2a", "#2aae3c", "#86f08e", "#0c1a4a", "#1c3ca6", "#2a54c9", "#7a9cff"]),
    ground: Some("#0a0a0a"),
    ..Meta::DEFAULT
};

const RAD: f64 = PI / 180.0;
/// Columns per unit.
const SCALE: f64 = 4.2;
/// Spacing of sampled points, in units.
const DS: f64 = 0.035;
/// Degrees a second, for the big gear.
const SPEED: f64 = 17.0;
/// Tooth depth; gears.c has 0.7.
const DEPTH: f64 = 0.8;

/// A gear: inner and outer radius, width, teeth, place, turn as `k * a + off`,
/// and the shade of its face (0–3).
#[derive(Clone, Copy)]
struct Gear {
    r0: f64,
    w: f64,
    n: f64,
    x: f64,
    y: f64,
    k: f64,
    off: f64,
    face: u8,
    r1: f64,
    r2: f64,
}

impl Gear {
    /// The tooth outline as radius by angle: up a flank, across, down, the root.
    fn radius(&self, th: f64) -> f64 {
        let u = (th * self.n / TAU).rem_euclid(1.0);
        let rise = if u < 0.25 {
            u * 4.0
        } else if u < 0.5 {
            1.0
        } else if u < 0.75 {
            (0.75 - u) * 4.0
        } else {
            0.0
        };
        self.r1 + (self.r2 - self.r1) * rise
    }
}

/// glRotatef(view_rotx = 20) then glRotatef(view_roty = 30), and the light at (5, 5, 10).
struct View {
    cy: f64,
    sy: f64,
    cx: f64,
    sx: f64,
    light: [f64; 3],
}

impl View {
    fn across(&self, x: f64, z: f64) -> f64 {
        x * self.cy + z * self.sy
    }
    fn up(&self, x: f64, y: f64, z: f64) -> f64 {
        y * self.cx + (x * self.sy - z * self.cy) * self.sx
    }
}

/// The gears, the view, the picture's top left in units, and the dots.
pub struct Glxgears {
    gears: [Gear; 3],
    view: View,
    left: f64,
    high: f64,
    dots: ColorOctants,
}

/// glxgears; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let gear = |r0: f64, r: f64, w: f64, n: f64, x: f64, y: f64, k: f64, face: u8| Gear {
        r0,
        w,
        n,
        x,
        y,
        k,
        off: 0.0,
        face,
        r1: r - DEPTH / 2.0,
        r2: r + DEPTH / 2.0,
    };
    // Twelve and six teeth for twenty and ten, so each tooth is big enough to read.
    let mut gears = [gear(1.0, 4.0, 1.0, 12.0, -3.0, -2.0, 1.0, 1), gear(0.5, 2.0, 2.0, 6.0, 3.1, -2.0, -2.0, 1), gear(1.3, 2.0, 0.5, 6.0, -3.1, 4.2, -2.0, 1)];
    // Offsets that set each small gear's gap on a tooth of the big one.
    let big = gears[0];
    for g in gears.iter_mut().skip(1) {
        let dir = (g.y - big.y).atan2(g.x - big.x);
        g.off = (dir + PI - ((0.875 - dir * big.n / TAU + 0.375) * TAU) / g.n) / RAD;
    }
    let l = (25.0f64 + 25.0 + 100.0).sqrt();
    let view = View { cy: (30.0 * RAD).cos(), sy: (30.0 * RAD).sin(), cx: (20.0 * RAD).cos(), sx: (20.0 * RAD).sin(), light: [5.0 / l, 5.0 / l, 10.0 / l] };
    // Centre the gears' outer circles in the frame.
    let (mut left, mut right, mut high, mut low) = (f64::MAX, f64::MIN, f64::MIN, f64::MAX);
    for g in &gears {
        let mut th = 0.0;
        while th < TAU {
            for z in [-g.w / 2.0, g.w / 2.0] {
                let (x, y) = (g.x + g.r2 * th.cos(), g.y + g.r2 * th.sin());
                left = left.min(view.across(x, z));
                right = right.max(view.across(x, z));
                high = high.max(view.up(x, y, z));
                low = low.min(view.up(x, y, z));
            }
            th += 0.02;
        }
    }
    let (cols, rows) = (META.cols as f64, META.rows as f64);
    let left = left - (cols / SCALE - (right - left)) / 2.0;
    let high = high + (2.0 * rows / SCALE - (high - low)) / 2.0;
    Box::new(Glxgears { gears, view, left, high, dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Glxgears {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let a = SPEED * t;
        let (v, left, high) = (&self.view, self.left, self.high);
        let dots = &mut self.dots;
        dots.clear();
        // A point and its normal in gear space; a wall's normal has no z.
        let mut plot = |i: usize, x: f64, y: f64, z: f64, nx: f64, ny: f64, face: Option<u8>| {
            let col = ((v.across(x, z) - left) * SCALE * 2.0).floor() as i64;
            let row = ((high - v.up(x, y, z)) * SCALE * 2.0).floor() as i64;
            let d = y * v.sx + (z * v.cy - x * v.sy) * v.cx;
            let (vx, vy, vz) = (nx * v.cy, ny * v.cx + nx * v.sy * v.sx, ny * v.sx - nx * v.sy * v.cx);
            if face.is_none() && vz < 0.0 {
                return;
            }
            // A wall is lit where it faces the light, dark where it does not.
            let shade = face.unwrap_or(if vx * v.light[0] + vy * v.light[1] + vz * v.light[2] > 0.42 { 3 } else { 0 });
            dots.plot(col, row, d as f32, i as u8 * 4 + shade);
        };
        for (i, g) in self.gears.iter().enumerate() {
            let (spin, h) = ((g.k * a + g.off) * RAD, g.w / 2.0);
            // The front face, ring by ring. The back face never turns to the eye.
            let mut r = g.r0;
            while r < g.r2 {
                let mut th = 0.0;
                while th < TAU {
                    if r <= g.radius(th) {
                        // The face shades a little from its hub to its rim.
                        let face = g.face + u8::from(r > (g.r0 + g.r1) / 2.0);
                        plot(i, g.x + r * (th + spin).cos(), g.y + r * (th + spin).sin(), h, 0.0, 0.0, Some(face));
                    }
                    th += DS / r;
                }
                r += DS;
            }
            // The outer wall, walked by arc length so the flanks are as dense as the tips.
            let mut th = 0.0;
            while th < TAU {
                let r = g.radius(th);
                let dr = (g.radius(th + 1e-4) - g.radius(th - 1e-4)) / 2e-4;
                let l = (r * r + dr * dr).sqrt();
                let (s, c) = (th + spin).sin_cos();
                let mut z = -h;
                while z <= h {
                    plot(i, g.x + r * c, g.y + r * s, z, (r * c + dr * s) / l, (r * s - dr * c) / l, None);
                    z += DS;
                }
                th += DS / l;
            }
            // The bore, facing in.
            let mut th = 0.0;
            while th < TAU {
                let (s, c) = th.sin_cos();
                let mut z = -h;
                while z <= h {
                    plot(i, g.x + g.r0 * c, g.y + g.r0 * s, z, -c, -s, None);
                    z += DS;
                }
                th += DS / g.r0;
            }
        }
        // In one ink the colours are gone, so the dark walls are dithered
        // (on paper, the faces), and a gear still shows its depth.
        let paper = env.paper;
        dots.dither_by(|c| match (c % 4, paper) {
            (0, false) | (1 | 2, true) => 0.5,
            _ => 1.0,
        });
        grid.clear();
        dots.draw_split(grid, 0, 0);
    }
}
