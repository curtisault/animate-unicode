//! flow field: particles carried left to right by a smooth noise field whose
//! currents pull them together, so their fading trails gather into a few
//! winding streams that merge and part as the field slowly changes. After
//! ascii.rest's flow-field.ts (MIT, @bas3line).
//!
//! The field and the particles are the TypeScript's, from its own seeded
//! generator; the trails are laid down per braille dot, eight to a cell, so
//! each one is a line of its own, and each dot is coloured by its heading,
//! violet climbing through green to orange falling, so the streams read as
//! currents.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// The flow field's meta.
pub static META: Meta = Meta {
    name: "flow field",
    slug: "flow-field",
    category: Category::Generative,
    note: "braille trails gathering into streams, coloured by their heading",
    cols: 64,
    rows: 22,
    fps: 20,
    charset: Charset::Extended,
    // Six headings from steep up to steep down, each bright then dim.
    palette: Some(&["#a77bff", "#5d3fa0", "#5f9bff", "#34558f", "#4fe0c0", "#2a7a68", "#9cf06a", "#557f38", "#ffd35a", "#8a7130", "#ff8a4a", "#8a4a2a"]),
    ground: Some("#08090d"),
    ..Meta::DEFAULT
};

/// Currents across the field.
const STREAMS: usize = 4;
/// Particles let go from the left edge a second.
const RATE: f64 = 16.0;
/// Cell widths a second.
const SPEED: f64 = 9.0;
/// Seconds for a trail to fade to about a third.
const FADE: f64 = 1.6;
/// How far a current wanders up and down, in cell widths.
const BEND: f64 = 12.0;
/// Noise cycles per cell width along a current.
const SCALE: f64 = 0.085;
/// How hard a current draws particles in, per cell width, and how far it reaches.
const PULL: f64 = 0.22;
const REACH: f64 = 7.0;
/// How fast the currents change.
const DRIFT: f64 = 0.04;
/// The steepest heading off the horizontal, in radians.
const TURN: f64 = 1.2;
/// Seconds simulated before the first frame.
const WARM: f64 = 9.0;
const HEADINGS: usize = 6;

/// A fixed number in [0, 1) for each lattice point, as the TypeScript's hash.
fn hash(i: i32, j: i32, k: i32) -> f64 {
    let mut h = i.wrapping_mul(0x27d4_eb2d) ^ j.wrapping_mul(0x1656_67b1) ^ k.wrapping_mul(0x9e37_79b1_u32 as i32);
    h = (h ^ ((h as u32) >> 15) as i32).wrapping_mul(0x85eb_ca77_u32 as i32);
    f64::from((h ^ ((h as u32) >> 13) as i32) as u32) / 4_294_967_296.0
}

/// Smooth value noise along a line, changing with time z, in [0, 1).
fn noise(x: f64, z: f64, k: i32) -> f64 {
    let (i, j) = (x.floor(), z.floor());
    let s = |f: f64| f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let (u, v) = (s(x - i), s(z - j));
    let (i, j) = (i as i32, j as i32);
    let a = hash(i, j, k) + (hash(i + 1, j, k) - hash(i, j, k)) * u;
    let b = hash(i, j + 1, k) + (hash(i + 1, j + 1, k) - hash(i, j + 1, k)) * u;
    a + (b - a) * v
}

/// A particle: where it is, in cell widths, and its lane off the current.
struct Particle {
    x: f64,
    y: f64,
    o: f64,
}

/// The field: its generator, the particles, the ink and heading of every dot,
/// and its clock.
pub struct FlowField {
    rand: Mulberry32,
    ps: Vec<Particle>,
    ink: Vec<f32>,
    heading: Vec<u8>,
    clock: f64,
    due: f64,
    last: f64,
    dots: ColorDots,
}

/// The field in cell widths: a row is two of them tall.
const W: f64 = META.cols as f64;
const H: f64 = (META.rows * 2) as f64;

/// Where each current runs at column x, and its slope there.
fn currents(x: f64, z: f64) -> ([f64; STREAMS], [f64; STREAMS]) {
    let (mut ys, mut ss) = ([0.0; STREAMS], [0.0; STREAMS]);
    for k in 0..STREAMS {
        // Wandering freely mid-frame, eased back in near the top and bottom.
        let at = |xx: f64| {
            let y = ((k as f64 + 0.5) / STREAMS as f64 - 0.5) * H + BEND * (2.0 * noise(xx * SCALE + k as f64 * 7.3, z + k as f64 * 3.1, k as i32) - 1.0);
            H / 2.0 + (H / 2.0 - 3.0) * (y / (H / 2.0 - 3.0)).tanh()
        };
        ys[k] = at(x);
        ss[k] = at(x + 0.5) - at(x - 0.5);
    }
    (ys, ss)
}

/// The heading of a particle in lane `o` at (x, y): along the nearby
/// currents, and in towards them.
fn heading(x: f64, y: f64, o: f64, z: f64) -> f64 {
    let (ys, ss) = currents(x, z);
    let (mut num, mut den, mut best, mut near) = (0.0, 1e-9, 0.0, 1e9);
    for k in 0..STREAMS {
        let d = ys[k] + o - y;
        let w = (-(d / REACH).powi(2)).exp();
        num += w * (ss[k] + PULL * d);
        den += w;
        if d.abs() < near {
            near = d.abs();
            best = ss[k] + PULL * d;
        }
    }
    let slope = if den > 1e-4 { num / den } else { best };
    slope.atan().clamp(-TURN, TURN)
}

impl FlowField {
    fn step(&mut self, dt: f64) {
        self.clock += dt;
        let keep = (-dt / FADE).exp() as f32;
        self.ink.iter_mut().for_each(|v| *v *= keep);
        let z = self.clock * DRIFT;
        // New particles come in at the left edge: most near a current, some
        // anywhere, to show the field drawing them in.
        self.due -= dt;
        while self.due <= 0.0 {
            let (ys, _) = currents(0.0, z);
            let y = if self.rand.next_f64() < 0.75 {
                ys[(self.rand.next_f64() * STREAMS as f64) as usize] + (self.rand.next_f64() - 0.5) * 8.0
            } else {
                2.0 + self.rand.next_f64() * (H - 4.0)
            };
            let o = (self.rand.next_f64() - 0.5) * 3.2;
            self.ps.push(Particle { x: 0.0, y, o });
            self.due += 1.0 / RATE;
        }
        let (dw, dh) = (self.dots.width(), self.dots.height());
        let mut i = self.ps.len();
        while i > 0 {
            i -= 1;
            let p = &mut self.ps[i];
            let a = heading(p.x, p.y, p.o, z);
            let (dx, dy) = (a.cos() * SPEED * dt, a.sin() * SPEED * dt);
            // Marked dot by dot: two dots a cell width.
            let n = ((dx * dx + dy * dy).sqrt() * 3.0).ceil().max(1.0) as usize;
            let dir = ((a + TURN) / (2.0 * TURN) * HEADINGS as f64).min(HEADINGS as f64 - 1.0) as u8;
            for s in 1..=n {
                let f = s as f64 / n as f64;
                let (x, y) = (((p.x + dx * f) * 2.0).floor(), ((p.y + dy * f) * 2.0).floor());
                if x >= 0.0 && y >= 0.0 && (x as usize) < dw && (y as usize) < dh {
                    let k = y as usize * dw + x as usize;
                    self.ink[k] = 1.0;
                    self.heading[k] = dir;
                }
            }
            p.x += dx;
            p.y += dy;
            if p.x >= W || p.y < 0.0 || p.y >= H {
                self.ps.remove(i);
            }
        }
    }
}

/// A flow field; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    let mut f = FlowField { rand: Mulberry32::new(0x5eed), ps: vec![], ink: vec![0.0; n], heading: vec![0; n], clock: 0.0, due: 0.0, last: 0.0, dots };
    for _ in 0..(WARM * META.fps as f64) as usize {
        f.step(1.0 / META.fps as f64);
    }
    Box::new(f)
}

impl Piece for FlowField {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let dt = (t - self.last).clamp(0.0, 0.1);
        self.last = t;
        if dt > 0.0 {
            self.step(dt);
        }
        // Fresh ink is solid and bright; as it fades it thins and dims.
        let w = self.dots.width();
        self.dots.clear();
        for (k, &v) in self.ink.iter().enumerate() {
            let (x, y) = (k % w, k / w);
            let v = f64::from(v);
            if v > 0.5 || (v > 0.12 && dither(x, y, v * 1.4)) {
                self.dots.set(x as i64, y as i64, self.heading[k] * 2 + u8::from(v <= 0.5));
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
