//! reaction diffusion: a Gray-Scott reaction whose spots on the left give way
//! to stripes on the right. Every few seconds the kill rate rises, the
//! pattern dies back to a few survivors, and they grow out again. After
//! ascii.rest's reaction-diffusion.ts (MIT, @bas3line).
//!
//! The reaction is ascii.rest's, step for step, on its 60×48 grid (checked
//! against the TypeScript). It is drawn per octant dot, the field read
//! between grid points, so the 60×24 characters show it at 120×96 with
//! smooth, crisp edges where the TS had a character a cell; and the
//! chemical's strength is coloured, teal at the fringes to cream at the core.

use crate::util::{smoothstep, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// Reaction-diffusion's meta.
pub static META: Meta = Meta {
    name: "reaction diffusion",
    slug: "reaction-diffusion",
    category: Category::Generative,
    note: "gray-scott spots and stripes in octants, teal to cream",
    cols: 60,
    rows: 24,
    fps: 20,
    charset: Charset::Extended,
    // The second chemical, faint to strong.
    palette: Some(&["#0f4a52", "#16707a", "#1f9a9a", "#3cbfa8", "#7ad8b0", "#bfeac0", "#f4f6dc"]),
    ground: Some("#071012"),
    ..Meta::DEFAULT
};

/// The grid, two cells to a character so they are square.
const W: usize = 60;
const H: usize = 48;
/// Feed and kill rates for spots and for stripes.
const SPOTS: (f64, f64) = (0.0367, 0.0649);
const STRIPES: (f64, f64) = (0.03, 0.057);
/// Diffusion.
const DU: f64 = 0.45;
const DV: f64 = 0.225;
/// Reaction steps a second, and steps from one die back to the next.
const RATE: f64 = 1000.0;
const CYCLE: usize = 8000;
/// The kill rate's rise over a cycle, and its height.
const BACK: [f64; 4] = [0.1, 0.25, 0.35, 0.48];
const BUMP: f64 = 0.012;
/// Seeds scattered at the start, and steps run before the first frame.
const SEEDS: usize = 48;
const WARM: usize = 1200;
/// Below the floor is bare ground; the top is full strength.
const FLOOR: f64 = 0.12;
const TOP: f64 = 0.28;
const STRENGTH: Ramp = Ramp::new(0, 7);

/// ascii.rest's smoothstep from a to b.
fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let s = ((x - a) / (b - a)).clamp(0.0, 1.0);
    s * s * (3.0 - 2.0 * s)
}

/// The reaction: both chemicals, the feed and kill by column, the die-back's
/// bite, its generator and step count, the frame's edge fade, and the dots.
pub struct ReactionDiffusion {
    u: Vec<f32>,
    v: Vec<f32>,
    u2: Vec<f32>,
    v2: Vec<f32>,
    feed: Vec<f32>,
    kill: Vec<f32>,
    bite: Vec<f32>,
    rand: Mulberry32,
    n: usize,
    cycle: i64,
    fade: Vec<f32>,
    dots: ColorOctants,
}

impl ReactionDiffusion {
    fn seed(&mut self, cx: usize, cy: usize) {
        for dy in -2i64..=2 {
            for dx in -2i64..=2 {
                let i = (cy as i64 + dy).rem_euclid(H as i64) as usize * W + (cx as i64 + dx).rem_euclid(W as i64) as usize;
                self.u[i] = 0.5;
                self.v[i] = 0.25;
            }
        }
    }

    /// Each die back is uneven: a smooth random field, new every cycle, sets
    /// how hard it bites where, so the survivors fall differently each time.
    /// The phase's 6.283 is ascii.rest's, kept so the port draws the same field.
    #[allow(clippy::approx_constant)]
    fn reshape(&mut self) {
        let mut waves = [(0.0, 0.0, 0.0); 3];
        for wave in &mut waves {
            let (mut kx, mut ky) = (0.0, 0.0);
            while kx == 0.0 && ky == 0.0 {
                kx = (self.rand.next_f64() * 5.0).floor() - 2.0;
                ky = (self.rand.next_f64() * 5.0).floor() - 2.0;
            }
            *wave = (kx, ky, self.rand.next_f64() * 6.283);
        }
        for y in 0..H {
            for x in 0..W {
                let s: f64 = waves.iter().map(|&(kx, ky, p)| (TAU * (kx * x as f64 / W as f64 + ky * y as f64 / H as f64) + p).cos()).sum();
                self.bite[y * W + x] = (1.0 + s / 6.0) as f32;
            }
        }
    }

    fn step(&mut self) {
        // The cycle starts on the first frame.
        let at = (self.n as f64 - WARM as f64) / CYCLE as f64;
        let k = at.floor() as i64;
        if k != self.cycle {
            self.cycle = k;
            self.reshape();
        }
        let p = at - k as f64;
        let b = BUMP * smooth(BACK[0], BACK[1], p) * (1.0 - smooth(BACK[2], BACK[3], p));
        self.n += 1;
        let (u, v) = (&self.u, &self.v);
        for y in 0..H {
            let (ym, y0, yp) = (((y + H - 1) % H) * W, y * W, ((y + 1) % H) * W);
            for x in 0..W {
                let (xm, xp) = ((x + W - 1) % W, (x + 1) % W);
                let i = y0 + x;
                let f = |a: &[f32], j: usize| f64::from(a[j]);
                let lu = 0.2 * (f(u, ym + x) + f(u, yp + x) + f(u, y0 + xm) + f(u, y0 + xp))
                    + 0.05 * (f(u, ym + xm) + f(u, ym + xp) + f(u, yp + xm) + f(u, yp + xp))
                    - f(u, i);
                let lv = 0.2 * (f(v, ym + x) + f(v, yp + x) + f(v, y0 + xm) + f(v, y0 + xp))
                    + 0.05 * (f(v, ym + xm) + f(v, ym + xp) + f(v, yp + xm) + f(v, yp + xp))
                    - f(v, i);
                let uvv = f(u, i) * f(v, i) * f(v, i);
                let (fe, ki) = (f64::from(self.feed[x]), f64::from(self.kill[x]));
                self.u2[i] = (f(u, i) + DU * lu - uvv + fe * (1.0 - f(u, i))) as f32;
                self.v2[i] = (f(v, i) + DV * lv + uvv - (fe + ki + b * f64::from(self.bite[i])) * f(v, i)) as f32;
            }
        }
        std::mem::swap(&mut self.u, &mut self.u2);
        std::mem::swap(&mut self.v, &mut self.v2);
        // Should a die back ever take everything, start again from a few seeds.
        if p < BACK[3] && p + 1.0 / CYCLE as f64 >= BACK[3] && self.v.iter().map(|&x| f64::from(x)).sum::<f64>() < 8.0 {
            for _ in 0..4 {
                let (x, y) = ((self.rand.next_f64() * W as f64) as usize, (self.rand.next_f64() * H as f64) as usize);
                self.seed(x, y);
            }
        }
    }

    /// The second chemical between grid points: (x, y) in grid units, wrapping.
    fn v_at(&self, x: f64, y: f64) -> f64 {
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let at = |dx: i64, dy: i64| {
            let (xi, yi) = ((x0 as i64 + dx).rem_euclid(W as i64) as usize, (y0 as i64 + dy).rem_euclid(H as i64) as usize);
            f64::from(self.v[yi * W + xi])
        };
        let top = at(0, 0) + (at(1, 0) - at(0, 0)) * fx;
        let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * fx;
        top + (bottom - top) * fy
    }
}

/// Reaction-diffusion after its warm-up.
fn warm() -> ReactionDiffusion {
    let n = W * H;
    let (mut feed, mut kill) = (vec![0.0; W], vec![0.0; W]);
    for x in 0..W {
        let s = smooth(0.1, 0.9, (1.0 + (TAU * (x as f64 + 0.5) / W as f64).sin()) / 2.0);
        feed[x] = (STRIPES.0 + (SPOTS.0 - STRIPES.0) * s) as f32;
        kill[x] = (STRIPES.1 + (SPOTS.1 - STRIPES.1) * s) as f32;
    }
    let dots = ColorOctants::new(META.cols, META.rows);
    // The frame fades toward its edges, so the pattern thins out there.
    let (dw, dh) = (dots.width(), dots.height());
    let fade = (0..dw * dh)
        .map(|k| {
            let (ax, ay) = ((((k % dw) as f64 + 0.5) / dw as f64 - 0.5).abs() * 2.0, (((k / dw) as f64 + 0.5) / dh as f64 - 0.5).abs() * 2.0);
            (1.0 - 0.6 * smooth(0.55, 1.0, (ax.powi(3) + ay.powi(3)).cbrt())) as f32
        })
        .collect();
    let mut rd = ReactionDiffusion {
        u: vec![1.0; n],
        v: vec![0.0; n],
        u2: vec![0.0; n],
        v2: vec![0.0; n],
        feed,
        kill,
        bite: vec![0.0; n],
        rand: Mulberry32::new(5),
        n: 0,
        cycle: -1,
        fade,
        dots,
    };
    for _ in 0..SEEDS {
        let (x, y) = ((rd.rand.next_f64() * W as f64) as usize, (rd.rand.next_f64() * H as f64) as usize);
        rd.seed(x, y);
    }
    while rd.n < WARM {
        rd.step();
    }
    rd
}

/// Reaction-diffusion; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(warm())
}

impl Piece for ReactionDiffusion {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let want = WARM + (t * RATE).floor() as usize;
        // A long pause skips ahead rather than stalls.
        if want > self.n + (RATE / 5.0) as usize {
            self.n = want - (RATE / 5.0).ceil() as usize;
        }
        while self.n < want {
            self.step();
        }
        let (dw, dh) = (self.dots.width(), self.dots.height());
        self.dots.clear();
        for py in 0..dh {
            for px in 0..dw {
                // Two dots to a grid cell each way.
                let val = self.v_at((px as f64 + 0.5) / 2.0 - 0.5, (py as f64 + 0.5) / 2.0 - 0.5) * f64::from(self.fade[py * dw + px]);
                if val < FLOOR {
                    continue;
                }
                let q = ((val - FLOOR) / (TOP - FLOOR)).min(1.0).sqrt();
                // The pattern is the ink, on paper too; its fringes are thinner.
                if crate::util::dither(px, py, 0.45 + 0.75 * smoothstep(q)) {
                    self.dots.set(px as i64, py as i64, STRENGTH.at(q));
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v at a few grid cells and its sum, from ascii.rest's TypeScript (node),
    /// after the warm-up and 200 steps on.
    #[test]
    fn the_reaction_matches_the_typescript() {
        let picks = [0, 100, 1000, 1500, 2000, 2500, 2879];
        let mut rd = warm();
        let want =
            [0.28331834077835083, 0.2930520474910736, 0.27528420090675354, 0.04408588260412216, 0.1742764115333557, 0.032488398253917694, 0.22573964297771454];
        for (k, &i) in picks.iter().enumerate() {
            assert!((f64::from(rd.v[i]) - want[k]).abs() < 1e-5, "after warm-up, v[{i}] = {} not {}", rd.v[i], want[k]);
        }
        assert!((rd.v.iter().map(|&x| f64::from(x)).sum::<f64>() - 374.7578755354725).abs() < 1e-2);
        for _ in 0..200 {
            rd.step();
        }
        let want =
            [0.1987147033214569, 0.29249846935272217, 0.2829141318798065, 0.043199148029088974, 0.19430100917816162, 0.03368274122476578, 0.13328497111797333];
        for (k, &i) in picks.iter().enumerate() {
            assert!((f64::from(rd.v[i]) - want[k]).abs() < 1e-5, "200 steps on, v[{i}] = {} not {}", rd.v[i], want[k]);
        }
    }
}
