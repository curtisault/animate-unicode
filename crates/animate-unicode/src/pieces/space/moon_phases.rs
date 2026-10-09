//! moon phases: the moon through one lunar month. The sun swings round
//! behind it, so the terminator sweeps across the sphere and back again.
//! After ascii.rest's moon-phases.ts (MIT, @bas3line).
//!
//! The seas, the craters, Tycho's rays and the light are ascii.rest's,
//! sampled once per octant dot and shaded by ordered dithering, so the
//! terminator is a fine curve and the maria have edges. In colour each dot is
//! grey by the ground under it, the seas darker than the highlands and the
//! young craters and their rays nearly white; near new moon the earthshine
//! shows the night side's highlands in a faint blue, and the rim of the dark
//! side is a thin grey line.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The moon's meta.
pub static META: Meta = Meta {
    name: "moon phases",
    slug: "moon-phases",
    category: Category::Space,
    note: "the moon waxing and waning in octants, grey maria, faint earthshine",
    cols: 46,
    rows: 23,
    fps: 15,
    charset: Charset::Extended,
    // The ground in light, darkest sea to brightest ray; earthshine; the dark
    // side's rim; a star; a bright star.
    palette: Some(&["#4a4c54", "#76787e", "#a4a39e", "#cfcdc4", "#f0eee4", "#ffffff", "#34466c", "#3c3e46", "#7a7e94", "#c8ccdc"]),
    ground: Some("#05060a"),
    ..Meta::DEFAULT
};

const GROUND: Ramp = Ramp::new(0, 6);
const EARTHSHINE: u8 = 6;
const RIM: u8 = 7;
const STAR: u8 = 8;
const BRIGHT_STAR: u8 = 9;
/// Seconds for new moon to new moon, and the phase at t = 0 in radians past
/// new moon: a young crescent.
const MONTH: f64 = 24.0;
const START: f64 = 0.95;

/// The near side's seas as ellipses on the disk, x east and y north, radius
/// 1: centre, radii, how much darker, and a tilt in radians.
const MARIA: [[f64; 6]; 14] = [
    [-0.62, 0.08, 0.28, 0.5, 0.62, 0.0],  // Oceanus Procellarum
    [-0.3, 0.45, 0.27, 0.21, 0.7, 0.0],   // Imbrium
    [-0.3, 0.71, 0.25, 0.1, 0.55, 0.25],  // Frigoris, west
    [0.14, 0.79, 0.22, 0.085, 0.5, -0.2], // Frigoris, east
    [0.19, 0.42, 0.15, 0.14, 0.7, 0.0],   // Serenitatis
    [0.38, 0.12, 0.21, 0.16, 0.7, 0.0],   // Tranquillitatis
    [0.05, 0.25, 0.09, 0.06, 0.5, 0.0],   // Vaporum
    [-0.28, 0.08, 0.12, 0.09, 0.4, 0.0],  // Insularum
    [0.75, 0.3, 0.1, 0.14, 0.72, 0.0],    // Crisium
    [0.62, -0.12, 0.12, 0.2, 0.62, 0.0],  // Fecunditatis
    [0.42, -0.28, 0.09, 0.09, 0.62, 0.0], // Nectaris
    [-0.2, -0.34, 0.19, 0.14, 0.6, 0.0],  // Nubium
    [-0.52, -0.43, 0.09, 0.1, 0.65, 0.0], // Humorum
    [-0.88, -0.06, 0.05, 0.08, 0.5, 0.0], // Grimaldi
];
/// Young bright craters: Tycho, Copernicus, Kepler, Aristarchus.
const CRATERS: [[f64; 4]; 4] = [[-0.15, -0.72, 0.09, 0.4], [-0.33, 0.17, 0.05, 0.2], [-0.6, 0.14, 0.035, 0.15], [-0.75, 0.38, 0.03, 0.3]];
/// Tycho's rays: direction and length.
const RAYS: [[f64; 2]; 7] = [[1.75, 0.55], [1.3, 0.4], [2.3, 0.35], [0.4, 0.3], [3.1, 0.3], [-0.6, 0.22], [4.1, 0.2]];

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// How bright the ground is at a point of the disk, before any light falls on it.
fn albedo(x: f64, y: f64) -> f64 {
    let mut alb = 1.0;
    for [mx, my, rx, ry, k, rot] in MARIA {
        let (s, c) = rot.sin_cos();
        let (dx, dy) = (x - mx, y - my);
        alb -= k * (1.0 - smooth(0.55, 1.0, ((dx * c + dy * s) / rx).hypot((dy * c - dx * s) / ry)));
    }
    alb = alb.max(0.3);
    for [mx, my, rr, k] in CRATERS {
        alb += k * (1.0 - smooth(0.3, 1.0, (x - mx).hypot(y - my) / rr));
    }
    let [tx, ty, tr, _] = CRATERS[0];
    let (dx, dy) = (x - tx, y - ty);
    let (d, a) = (dx.hypot(dy), dy.atan2(dx));
    for [at, len] in RAYS {
        let off = (a - at).sin().atan2((a - at).cos()).abs();
        if d > tr && d < len && off < 1.0 {
            alb += 0.3 * (1.0 - d / len) * (-((off * d) / 0.03).powi(2)).exp();
        }
    }
    alb.min(1.35)
}

/// A dot of the disk: its normal, its ground, and whether it is on the rim.
struct Spot {
    k: usize,
    n: [f32; 3],
    albedo: f32,
    rim: bool,
}

/// The moon: every dot of its disk, the stars, and the dots.
pub struct MoonPhases {
    spots: Vec<Spot>,
    stars: Vec<(usize, u8)>,
    dots: ColorOctants,
}

/// The moon; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cx, cy) = (META.cols as f64 / 2.0, META.rows as f64 / 2.0);
    let r0 = META.rows as f64 / 2.0 - 1.0; // radius in rows; it spans twice as many columns
    let dots = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    let mut rand = Mulberry32::new(7);
    let (mut spots, mut stars) = (vec![], vec![]);
    for j in 0..h {
        let y = (cy - (j as f64 + 0.5) / 4.0) / r0;
        for i in 0..w {
            let x = ((i as f64 + 0.5) / 2.0 - cx) / (2.0 * r0);
            let d = x * x + y * y;
            if d < 1.0 {
                let n = [x as f32, y as f32, (1.0 - d).sqrt() as f32];
                spots.push(Spot { k: j * w + i, n, albedo: albedo(x, y) as f32, rim: d > 0.955 });
            } else if d.sqrt() > 1.12 && rand.next_f64() < 0.003 {
                stars.push((j * w + i, if rand.next_f64() < 0.25 { BRIGHT_STAR } else { STAR }));
            }
        }
    }
    Box::new(MoonPhases { spots, stars, dots })
}

impl Piece for MoonPhases {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let ph = START + TAU * t.rem_euclid(MONTH) / MONTH;
        let m = (ph.sin().powi(2) + 0.12f64.powi(2) + ph.cos().powi(2)).sqrt();
        let (lx, ly, lz) = (ph.sin() / m, 0.12 / m, -ph.cos() / m);
        // Near new moon the earth is nearly full in the moon's sky, and its
        // light shows the night side's highlands faintly.
        let earthshine = smooth(0.0, 0.5, ph.cos());
        let w = self.dots.width();
        self.dots.clear();
        for &(k, color) in &self.stars {
            self.dots.set((k % w) as i64, (k / w) as i64, color);
        }
        for s in &self.spots {
            let (x, y) = (s.k % w, s.k / w);
            let [nx, ny, nz] = s.n.map(f64::from);
            let albedo = f64::from(s.albedo);
            let mu0 = nx * lx + ny * ly + nz * lz;
            // Mostly Lommel-Seeliger, which keeps the full moon nearly flat,
            // with a little Lambert so the ball still reads as round.
            let v = if mu0 > 0.0 { (albedo * (0.3 * mu0 + 0.7 * 2.0 * mu0 / (mu0 + nz))).min(1.0) } else { 0.0 };
            let pick = if v >= 0.015 {
                // On paper the light is the paper, and the shadow is ink.
                let ink = if env.paper { 1.0 - v } else { 0.1 + 0.9 * v };
                dither(x, y, ink).then(|| GROUND.at((albedo - 0.3) / 1.05 * (0.55 + 0.45 * v.sqrt())))
            } else if earthshine * albedo > 0.72 {
                dither(x, y, if env.paper { 0.8 } else { 0.14 }).then_some(EARTHSHINE)
            } else if s.rim {
                dither(x, y, 0.5).then_some(RIM)
            } else {
                env.paper.then_some(RIM).filter(|_| dither(x, y, 0.9))
            };
            if let Some(color) = pick {
                self.dots.set(x as i64, y as i64, color);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
