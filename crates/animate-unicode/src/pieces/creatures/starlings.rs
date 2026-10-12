//! starlings: a murmuration. Each bird matches its neighbours' heading, keeps
//! to their middle and makes room for the one beside it, while a slow drift
//! wheels the cloud about the middle. Every so often a falcon cuts through
//! it. After ascii.rest's starlings.ts (MIT, @bas3line).
//!
//! The flocking and the falcon are ascii.rest's, but every bird is one
//! braille dot, so the same cells hold eight times the birds: 2560 where
//! ascii.rest has 320. The distances they keep are scaled down to match, so
//! the cloud is the same size and each bird still minds about as many
//! neighbours, and the murmuration shows its folds and ripples as a dot
//! texture. In colour the birds are pale on a dusk sky, whitest where they
//! crowd, and the falcon is amber.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// The starlings' meta.
pub static META: Meta = Meta {
    name: "starlings",
    slug: "starlings",
    category: Category::Creatures,
    note: "a murmuration of 2560 braille birds wheeling, split by a falcon",
    cols: 64,
    rows: 16,
    fps: 30,
    charset: Charset::Extended,
    // Birds, sparse to crowded; the falcon.
    palette: Some(&["#8a809c", "#c8c0dc", "#f4f0ff", "#e8a040"]),
    ground: Some("#140f1c"),
    ..Meta::DEFAULT
};

const FALCON: u8 = 3;
/// A column and a row, in em; birds; the step.
const CW: f64 = 0.6;
const RH: f64 = 1.2;
const N: usize = 2560;
const DT: f64 = 1.0 / 30.0;
/// Steps before the first frame, between falcons, and the first falcon's.
const WARM: i64 = 200;
const PASS: i64 = 210;
const FIRST: i64 = WARM - 10;
/// Neighbours are this close, and this close is too close: ascii.rest's
/// distances over the root of eight, for eight times the birds.
const R: f64 = 1.6 / 2.828;
const SEP: f64 = 0.85 / 2.828;
/// How far ahead a bird looks for the edge, how hard it turns, and how close
/// to the falcon a bird scatters.
const LOOK: f64 = 0.6;
const TURN: f64 = 90.0;
const FLEE: f64 = 4.2;

/// A smooth wave in [-1, 1] with period p.
fn osc(t: f64, p: f64) -> f64 {
    let u = 1.0 - (2.0 * (t / p - (t / p).floor()) - 1.0).abs();
    2.0 * u * u * (3.0 - 2.0 * u) - 1.0
}

/// The flock: every bird's place and velocity, the neighbour grid, the step
/// count, the falcon (place and heading), and the dots.
pub struct Starlings {
    x: Vec<f64>,
    y: Vec<f64>,
    vx: Vec<f64>,
    vy: Vec<f64>,
    head: Vec<i32>,
    next: Vec<i32>,
    k: i64,
    falcon: (f64, f64, f64),
    dots: ColorDots,
}

/// A murmuration; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (w, h) = (META.cols as f64 * CW, META.rows as f64 * RH);
    let mut rnd = Mulberry32::new(26);
    let mut s = Starlings {
        x: vec![0.0; N],
        y: vec![0.0; N],
        vx: vec![0.0; N],
        vy: vec![0.0; N],
        head: vec![-1; ((w / R).ceil() * (h / R).ceil()) as usize],
        next: vec![-1; N],
        k: 0,
        falcon: (f64::NAN, 0.0, -1.0),
        dots: ColorDots::new(META.cols, META.rows),
    };
    for i in 0..N {
        s.x[i] = w * (0.3 + 0.3 * rnd.next_f64());
        s.y[i] = h * (0.3 + 0.4 * rnd.next_f64());
        s.vx[i] = 6.0 + 2.0 * rnd.next_f64();
        s.vy[i] = 2.0 * rnd.next_f64() - 1.0;
    }
    while s.k < WARM {
        s.step();
    }
    Box::new(s)
}

impl Starlings {
    fn step(&mut self) {
        let (w, h) = (META.cols as f64 * CW, META.rows as f64 * RH);
        let (gx_n, gy_n) = ((w / R).ceil() as i64, (h / R).ceil() as i64);
        let (mx, my) = (1.5, 1.2); // the open sky's oval, in from each side
        let t = self.k as f64 * DT;
        let ph = (self.k - FIRST + PASS * 9).rem_euclid(PASS);
        let hunting = self.k >= FIRST && ph < 90;
        self.k += 1;
        let (tx, ty) = (w * (0.5 + 0.12 * osc(t, 23.0)), h * (0.5 + 0.1 * osc(t + 4.0, 11.0)));
        let cy = self.y.iter().sum::<f64>() / N as f64;
        // Each falcon comes in from the side the last one left by, level with
        // the flock, and bends toward the middle of it as it goes.
        let (mut fx, mut fy, mut fd) = self.falcon;
        if hunting && ph == 0 {
            fd = -fd;
            fx = if fd < 0.0 { -2.0 } else { w + 2.0 };
            fy = cy;
        }
        fx = if hunting { fx + fd * 20.0 * DT } else { f64::NAN };
        if hunting {
            fy += (cy - fy) * 3.0 * DT;
        }
        self.falcon = (fx, fy, fd);
        let cell =
            |x: f64, y: f64| ((y / R).floor().clamp(0.0, (gy_n - 1) as f64) as i64 * gx_n + (x / R).floor().clamp(0.0, (gx_n - 1) as f64) as i64) as usize;
        self.head.fill(-1);
        for i in 0..N {
            let c = cell(self.x[i], self.y[i]);
            self.next[i] = self.head[c];
            self.head[c] = i as i32;
        }
        for i in 0..N {
            let (xi, yi) = (self.x[i], self.y[i]);
            let (gx0, gy0) = ((xi / R).floor() as i64, (yi / R).floor() as i64);
            let (mut n, mut sx, mut sy, mut svx, mut svy, mut px, mut py) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
            for gy in (gy0 - 1).max(0)..=(gy0 + 1).min(gy_n - 1) {
                for gx in (gx0 - 1).max(0)..=(gx0 + 1).min(gx_n - 1) {
                    let mut j = self.head[(gy * gx_n + gx) as usize];
                    while j >= 0 {
                        let ju = j as usize;
                        let (dx, dy) = (self.x[ju] - xi, self.y[ju] - yi);
                        let d2 = dx * dx + dy * dy;
                        if ju != i && d2 <= R * R {
                            n += 1.0;
                            sx += dx;
                            sy += dy;
                            svx += self.vx[ju];
                            svy += self.vy[ju];
                            if d2 < SEP * SEP {
                                px -= dx / (d2 + 0.02 / 8.0);
                                py -= dy / (d2 + 0.02 / 8.0);
                            }
                        }
                        j = self.next[ju];
                    }
                }
            }
            // Drift; align, cohere and separate. Separation pushes harder as
            // the birds are closer, so it is eased by the root of eight too.
            let (mut ax, mut ay) = ((tx - xi) * 0.25, (ty - yi) * 0.5);
            if n > 0.0 {
                ax += (svx / n - self.vx[i]) * 3.0 + sx / n * 0.5 * 2.828 + px * 4.0 / 2.828;
                ay += (svy / n - self.vy[i]) * 3.0 + sy / n * 0.5 * 2.828 + py * 4.0 / 2.828;
            }
            // A bird heading out of the open sky, an oval in the frame, turns
            // back toward the middle, harder the further out it is headed.
            let lx = (xi + self.vx[i] * LOOK - w / 2.0) / (w / 2.0 - mx);
            let ly = (yi + self.vy[i] * LOOK - h / 2.0) / (h / 2.0 - my);
            let out = lx.hypot(ly) - 0.6;
            if out > 0.0 {
                let s = self.vx[i].hypot(self.vy[i]);
                let side = if self.vx[i] * (h / 2.0 - yi) - self.vy[i] * (w / 2.0 - xi) > 0.0 { 1.0 } else { -1.0 };
                ax += -self.vy[i] / s * side * TURN * out;
                ay += self.vx[i] / s * side * TURN * out;
            }
            // The falcon: birds near it break hard away and fly fast.
            let mut top = 10.0;
            let (dx, dy) = (xi - fx, yi - fy);
            let d2 = dx * dx + dy * dy;
            if d2 < FLEE * FLEE {
                ax += dx / (d2 + 0.4) * 90.0;
                ay += dy / (d2 + 0.4) * 90.0;
                top = 15.0;
            }
            let (ux, uy) = (self.vx[i] + ax * DT, self.vy[i] + ay * DT);
            let s = ux.hypot(uy);
            let m = if s > top {
                top / s
            } else if s < 5.0 {
                5.0 / s
            } else {
                1.0
            };
            self.vx[i] = ux * m;
            self.vy[i] = uy * m;
        }
        for i in 0..N {
            self.x[i] += self.vx[i] * DT;
            self.y[i] += self.vy[i] * DT;
        }
    }
}

impl Piece for Starlings {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let want = WARM + (t / DT + 1e-6).floor() as i64;
        for _ in 0..12 {
            if self.k >= want {
                break;
            }
            self.step();
        }
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        // Count the birds on each dot, so crowded places draw whitest.
        let mut count = vec![0u8; (w * h) as usize];
        for i in 0..N {
            let (dx, dy) = ((self.x[i] / CW * 2.0).floor() as i64, (self.y[i] / RH * 4.0).floor() as i64);
            if (0..w).contains(&dx) && (0..h).contains(&dy) {
                let c = &mut count[(dy * w + dx) as usize];
                *c = c.saturating_add(1);
            }
        }
        self.dots.clear();
        for (k, &c) in count.iter().enumerate() {
            if c > 0 {
                self.dots.set(k as i64 % w, k as i64 / w, (c.min(3) - 1).min(2));
            }
        }
        // The falcon, beating its wings as it goes.
        let (fx, fy, _) = self.falcon;
        if fx.is_finite() {
            let (cx, cy) = ((fx / CW * 2.0) as i64, (fy / RH * 4.0) as i64);
            let up = (self.k >> 2) & 1 == 1;
            let shape: &[(i64, i64)] = if up {
                &[(-3, -2), (-2, -1), (-1, 0), (0, 0), (1, 0), (2, -1), (3, -2), (0, 1)]
            } else {
                &[(-3, 0), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (3, 0), (0, 1)]
            };
            for &(dx, dy) in shape {
                self.dots.set(cx + dx, cy + dy, FALCON);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
