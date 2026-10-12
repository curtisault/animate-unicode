//! deep reef: looking along a coral reef from a few metres down. The sun is a
//! bright blaze in the rippled surface, shafts of light fan down from it,
//! kelp sways in the swell, a school of fish wheels through the dark water,
//! bubbles rise and caustics crawl over the sand. After ascii.rest's
//! deep-reef.ts (MIT, @bas3line).
//!
//! The water, the sand, the reefs, the sea fan and the corals, the shafts,
//! the caustics, the kelp, the school, the bubbles and the drifting specks,
//! and how each is shaded, are ascii.rest's, shaded here per pixel instead of
//! per cell. Where ascii.rest draws each of 200 × 100 square cells as a dot as
//! big as it is bright, this draws 320 × 80 cells of two square pixels each,
//! an upper half block over a back colour ([`crate::pixels`]): 1.6 times the
//! detail each way, solid colour, and gradients dithered in colour rather than
//! in dot size. ascii.rest's palette starts at mid tones, since a dot's size
//! carried its darkness; this one adds the deep water, the rock, the kelp and
//! the shadowed corals a solid pixel needs. In one ink each pixel is octant
//! dots, as many as it is bright.
//!
//! Added: each fish is a little sprite of a few pixels, a tapering body and a
//! forked tail, turned along its heading, where ascii.rest's is two dots; and
//! the caustic web is worked out at twice ascii.rest's resolution, so its
//! lines stay sharp at the finer pixel.

use crate::noise::{clamp, fbm, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::util::Mulberry32;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// Deep reef's meta.
pub static META: Meta = Meta {
    name: "deep reef",
    slug: "deep-reef",
    category: Category::Wilds,
    note: "light shafts, swaying kelp and a turning school of fish over a reef",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's (from mid tones up): the water's teals, sand, kelp olives
    // and golds, the corals' pinks and oranges, purples, the haze's greys.
    // Then the dark water, the rock, the kelp and the corals in shadow.
    palette: Some(&[
        "#114a66", "#165c78", "#1d708a", "#27869b", "#3a9fae", "#58b9c0", "#80d2d2", "#b0e8e2", "#e2fbf5", "#5e6b62", "#87927f", "#aab39a", "#cfd2b4",
        "#38461a", "#5a6420", "#857f2a", "#b0a03c", "#d6c25a", "#efe08e", "#8e3355", "#c4506a", "#e0786e", "#f0a070", "#6a3a78", "#9a5aa8", "#c88ad0",
        "#6f8a98", "#9fb8c6", "#cfe3ea", "#03101a", "#041620", "#061d29", "#082433", "#0b2c3d", "#0d3447", "#0f3d52", "#0a2a2e", "#0e3a3c", "#020608",
        "#050b0e", "#081014", "#1e211d", "#2c2f29", "#3b3e35", "#4a4d42", "#3a1a2a", "#5e2840", "#4a2a1a", "#6e3e24", "#2e1a3a", "#45285a", "#3a3618",
        "#5a5222", "#141408", "#1e1e0c", "#2a2a12",
    ]),
    ground: Some("#03101a"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame, in its cells.
const TW: f64 = 200.0;
const TH: f64 = 100.0;
/// The surface band.
const SURF: f64 = 15.0;
/// Where the sea floor would meet the haze.
const HZ: f64 = 61.0;
/// Where the sun shows through the surface.
const SUNX: f64 = 136.0;
/// The sun, above the frame, that the shafts fan out from, and how many shafts.
const SUNY: f64 = -12.0;
const RAYS: usize = 320;
/// The shaft straight down from the sun.
const RAY0: f64 = 160.0;
/// The caustic tile: its size in columns, its cells a side, and its samples a column.
const CT: f64 = 64.0;
const CC: usize = 6;
const CQ: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    None,
    Sand,
    Reef,
    Far,
    Fan,
}

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn row_y(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

/// The scene's x, in ascii.rest's columns (a cell's left edge, as its `x`),
/// at pixel column `px`.
fn col_x(px: usize) -> f64 {
    (px as f64 + 0.5) / S - 0.5
}

/// The pixel nearest a point given as ascii.rest's cell coordinates (`x`, `r`).
fn to_pixel(x: f64, r: f64) -> (i64, i64) {
    (((x + 0.5) * S - 0.5).round() as i64, ((r + 0.5) * S - 0.5).round() as i64)
}

/// The reefs are rounded masses, shouldering down toward the open sand.
fn dome(u: f64) -> f64 {
    1.0 - (1.0 - u * u).max(0.0).sqrt()
}

fn left_top(x: f64) -> f64 {
    47.0 + 56.0 * dome((x / 86.0).min(1.0)) - 8.0 * fbm(x * 0.06, 3.1, 4) + 3.0 * ((10.0 - x) / 10.0).max(0.0)
}

fn right_top(x: f64) -> f64 {
    69.0 + 34.0 * dome(((196.0 - x) / 48.0).min(1.0)) - 5.0 * fbm(x * 0.08, 8.3, 3) - 3.0 * (-((x - 191.0) / 6.0).powi(2)).exp()
}

fn far_top(x: f64) -> f64 {
    HZ - 1.0 - 6.0 * fbm(x * 0.035 + 2.0, 1.7, 3) - 3.0 * (-((x - 120.0) / 18.0).powi(2)).exp()
}

/// The corals' colours.
const CORAL: [[f64; 3]; 4] = [[0.86, 0.36, 0.44], [0.93, 0.55, 0.3], [0.6, 0.36, 0.72], [0.78, 0.7, 0.35]];
const ROCK: [f64; 3] = [0.016, 0.03, 0.042];

/// A fish in the school: its place in it (along, across), a phase and a speed.
struct Fish {
    a: f64,
    b: f64,
    ph: f64,
    sp: f64,
}

/// A bubble: its stream's foot, a phase, a speed and a wobble.
struct Bubble {
    x0: f64,
    y0: f64,
    ph: f64,
    sp: f64,
    w: f64,
}

/// A speck drifting in the water.
struct Speck {
    x: f64,
    y: f64,
    sp: f64,
    ph: f64,
    b: f64,
}

/// The kelp: x, base row, length, width, phase, and how much water hides it.
const KELP: [[f64; 6]; 2] = [[14.0, 100.0, 96.0, 3.5, 0.0, 0.0], [189.0, 100.0, 96.0, 3.5, 5.2, 0.0]];

/// The reef: the water's colour, which shaft each pixel sits in, the scene
/// behind the water, the caustic tile and where each pixel reads it, the
/// moving things, and the picture.
pub struct DeepReef {
    caus: Vec<f32>,
    water: Vec<[f32; 3]>,
    ray_at: Vec<u16>,
    mat: Vec<Mat>,
    back: Vec<[f32; 3]>,
    fog: Vec<f32>,
    /// Caustic coordinates and weight.
    cuvw: Vec<[f32; 3]>,
    fish: Vec<Fish>,
    bubbles: Vec<Bubble>,
    specks: Vec<Speck>,
    rays: Vec<f64>,
    pixels: Pixels,
}

/// The caustic tile: a web of bright lines along the edges of a wrapping
/// Voronoi, `CT · CQ` samples a side.
fn caustics() -> Vec<f32> {
    let cs = CT / CC as f64;
    let mut rnd = Mulberry32::new(7);
    let mut pts = vec![];
    for j in 0..CC {
        for i in 0..CC {
            let px = (i as f64 + 0.15 + 0.7 * rnd.next_f64()) * cs;
            let py = (j as f64 + 0.15 + 0.7 * rnd.next_f64()) * cs;
            pts.push((px, py));
        }
    }
    let n = CT as usize * CQ;
    let mut caus = vec![0.0f32; n * n];
    for y in 0..n {
        for x in 0..n {
            let (sx, sy) = ((x as f64 + 0.5) / CQ as f64, (y as f64 + 0.5) / CQ as f64);
            let (mut f1, mut f2) = (1e9f64, 1e9f64);
            for &(px, py) in &pts {
                for oy in [-CT, 0.0, CT] {
                    for ox in [-CT, 0.0, CT] {
                        let d = (sx - px - ox).hypot(sy - py - oy);
                        if d < f1 {
                            f2 = f1;
                            f1 = d;
                        } else if d < f2 {
                            f2 = d;
                        }
                    }
                }
            }
            caus[y * n + x] = (1.0 - smooth(0.0, 0.42 * cs, f2 - f1)).powf(2.2) as f32;
        }
    }
    caus
}

/// A deep reef; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;

    // --- the water: deep navy, lit from the surface and the sun -------------
    let mut water = vec![[0.0f32; 3]; n];
    // which shaft each pixel sits in: shafts fan out from the sun, above the frame
    let mut ray_at = vec![0u16; n];
    for py in 0..H {
        let y = row_y(py);
        let r = y - 0.5;
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            let up = (1.0 - y / TH).powf(2.2);
            let sun = (-((x - SUNX) / 52.0).powi(2) - ((r - 4.0) / 34.0).powi(2)).exp();
            // looking level, the distance is a lit blue haze the reefs stand against
            let haze = (-((r - 54.0) / 15.0).powi(2)).exp() * (1.0 - 0.6 * smooth(30.0, 100.0, (x - 112.0).abs()));
            // darker toward the sides and the floor, but the upper water stays
            // lit so the kelp stands dark against it
            let edge = 1.0 - 0.4 * smooth(50.0, 104.0, (x - 112.0).abs()) * smooth(8.0, 70.0, r) - 0.3 * smooth(66.0, 100.0, r);
            // soft clouds of plankton haze, so open water is never one flat tone
            let veil = 0.78 + 0.44 * fbm(x * 0.022 + 5.0, r * 0.04, 4);
            water[k] = [
                ((0.012 + 0.09 * up + 0.05 * sun + 0.04 * haze) * edge * veil) as f32,
                ((0.1 + 0.32 * up + 0.14 * sun + 0.15 * haze) * edge * veil) as f32,
                ((0.15 + 0.27 * up + 0.12 * sun + 0.17 * haze) * edge * veil) as f32,
            ];
            let a = (x + 0.5 - SUNX).atan2(y - SUNY);
            ray_at[k] = ((a + 1.6) * 100.0).round().clamp(0.0, RAYS as f64 - 1.0) as u16;
        }
    }

    // --- the static scene: sand, the reef, a far reef in the haze, a sea fan
    let mut mat = vec![Mat::None; n];
    let mut lit_c = vec![[0.0f64; 3]; n]; // lit colour
    let mut fog = vec![0.0f32; n]; // how much water stands between us and it
    let mut cuvw = vec![[0.0f32; 3]; n]; // caustic coordinates, weight
                                         // brain corals: domes on the crests: x, the centre's row, radius, kind
    let domes = [
        [16.0, 3.0, 7.0, 0.0],
        [41.0, 2.5, 6.5, 1.0],
        [53.0, 2.0, 4.5, 3.0],
        [65.0, 2.0, 5.0, 2.0],
        [158.0, 2.0, 4.0, 2.0],
        [186.0, 3.0, 5.0, 1.0],
        [196.0, 3.0, 4.5, 3.0],
    ]
    .map(|[x, d, r, kind]| [x, if x < 100.0 { left_top(x) } else { right_top(x) } + d, r, kind]);
    // small coral heads out on the sand: x, crest row, half width, fog, coral kind
    let bommies = [[114.0, 69.0, 9.0, 0.18, 0.0], [94.0, 63.6, 4.0, 0.5, 2.0], [139.0, 64.4, 4.0, 0.42, 1.0]];
    let fan_c = (167.0, right_top(167.0) + 1.5);
    let fan_r = 27.0;
    for py in 0..H {
        let y = row_y(py);
        let r = y - 0.5;
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            let mut set = |m: Mat, c: [f64; 3], f: f64, uvw: [f64; 3]| {
                mat[k] = m;
                lit_c[k] = c;
                fog[k] = f as f32;
                cuvw[k] = uvw.map(|v| v as f32);
            };
            // the sea floor, a plane running off into the haze
            if y.floor() >= HZ + 2.0 {
                let d = y - HZ;
                let z = 300.0 / d; // distance
                                   // the floor is seen at a low angle, so its pattern squeezes
                                   // toward the haze; spacing grows as it comes nearer, and fades
                                   // out before it gets too fine to draw
                let near = 0.4 + 0.6 * (d / 38.0);
                let sx = (x + 0.5 - 100.0) / near;
                let sy = 30.0 * d.ln();
                let aa = smooth(2.2, 1.4, 30.0 / d);
                // ripples in the sand run across our view, bending a little:
                // thin bright crests over darker troughs
                let ph = sy + 7.0 * fbm(sx * 0.07, sy * 0.06, 2) + sx * 0.05;
                let crest = (0.5 + 0.5 * ph.sin()).powi(3) * smooth(0.3, 0.6, fbm(sx * 0.09 + 7.0, sy * 0.12, 2));
                let rip = 0.74 + aa * (0.5 * crest - 0.12) + 0.12 * (fbm(sx * 0.08, sy * 0.15, 3) - 0.5);
                let lit = 0.6 + 0.15 * smooth(100.0, 130.0, x) - 0.3 * smooth(84.0, 100.0, r);
                set(
                    Mat::Sand,
                    [0.38 * rip * lit, 0.37 * rip * lit, 0.3 * rip * lit],
                    1.0 - (-z / 40.0).exp(),
                    [sx * 0.9, sy * 2.3, 0.5 * smooth(14.0, 26.0, d)],
                );
            }
            let ft = far_top(x);
            if y >= ft && y.floor() < HZ + 6.0 {
                // lit along its crest, dim below, half lost in the blue
                let s = (0.45 + 0.55 * fbm(x * 0.15, y * 0.15, 2)) * (0.5 + 0.8 * (-(y - ft) / 2.0).exp());
                set(Mat::Far, [0.06 * s, 0.11 * s, 0.14 * s], 0.68, [0.0; 3]);
            }
            // the sea fan: a thin lattice of veins spreading up from its root
            {
                let (dx, dy) = (x + 0.5 - fan_c.0, fan_c.1 - y);
                let (d, a) = (dx.hypot(dy), dx.atan2(dy));
                let reach = fan_r * (0.68 + 0.36 * fbm(a * 2.6 + 5.0, 1.0, 3));
                if dy > -1.0 && a.abs() < 1.2 && d < reach {
                    let aw = a + 0.08 * (fbm(d * 0.2, a * 2.0, 2) - 0.5) * 4.0;
                    let ray = (((aw * 7.5) / PI + 100.0) % 1.0 - 0.5).abs();
                    let ring = ((d / 3.2 + 100.0) % 1.0 - 0.5).abs();
                    let mesh = hash(px as f64 * 3.0, py as f64 * 5.0) < 0.08;
                    let vein = ray > 0.42 || (ring > 0.45 && d > 5.0) || mesh || (d < 4.0 && dx.abs() < 1.2) || d > reach - 1.2;
                    // dark at the root, catching the light toward its rim;
                    // between the veins a thin dim web the water shows through
                    let o = smooth(2.0, reach, d);
                    let s = (0.35 + 0.75 * o) * if vein { 1.0 } else { 0.22 };
                    set(Mat::Fan, [mix(0.32, 0.72, o) * s, mix(0.12, 0.38, o) * s, mix(0.38, 0.8, o) * s], if vein { 0.2 } else { 0.4 }, [0.0; 3]);
                }
            }
            // small coral heads out on the sand, half lost in the blue
            for &[bx, crest, hw, f0, kind] in &bommies {
                let u = (x + 0.5 - bx) / hw;
                let top = crest + u * u * u * u * hw * 0.5 - 1.6 * fbm(x * 0.35, crest, 2);
                if u.abs() < 1.1 && y >= top && y < crest + hw * 0.5 + 1.5 * fbm(x * 0.3, crest + 5.0, 2) - 0.6 * u * u {
                    let below = y - top;
                    let living = smooth(2.2, 0.6, below) * smooth(0.45, 0.6, fbm(x * 0.25 + bx, y * 0.3, 2));
                    let lit = 0.3 + 0.6 * (-below / 1.5).exp();
                    let c = CORAL[kind as usize];
                    let col = [0, 1, 2].map(|i| mix(ROCK[i], c[i], living) * lit);
                    set(Mat::Reef, col, f0, [x * 0.5, y * 0.9, 0.5 * (-below / 1.5).exp()]);
                }
            }
            // reef masses, left and right: dark rock, rimmed with lit coral
            let top = if x < 100.0 { left_top(x) } else { right_top(x) };
            if y >= top && !(87.0..=147.0).contains(&x) {
                let below = y - top;
                let nz = fbm(x * 0.18, y * 0.22, 4);
                // lumps of rock and coral heads, each lit on its upper side
                let (lump, lump_up) = (fbm(x * 0.07, y * 0.1, 3), fbm(x * 0.07, (y - 1.5) * 0.1, 3));
                let face = clamp(0.5 + (lump_up - lump) * 14.0);
                let lit = (0.25 + 0.5 * face + 0.6 * (-below / 5.0).exp() + 0.14 * (nz - 0.5)) * (0.55 + 0.45 * smooth(100.0, 40.0, r));
                // patches of living coral, thick along the crest, a few sponges below
                let kind = ((fbm(x * 0.06 + 11.0, y * 0.09, 3) * 7.0).floor() as usize) % 4;
                let patch = fbm(x * 0.12 + 4.0, y * 0.12, 3);
                let living = (smooth(0.44, 0.56, patch) * smooth(7.0, 1.5, below)).max(smooth(0.66, 0.72, patch) * 0.35 * smooth(24.0, 6.0, below));
                let c = CORAL[kind];
                // a cool rim of light along the bare rock of the crest
                let rim = smooth(2.4, 0.3, below) * 0.6;
                // and the upper lips of ledges down the face catch a little of it
                let ledge = smooth(0.66, 0.9, face) * smooth(3.0, 8.0, below) * 0.55;
                let bare = [mix(mix(ROCK[0], 0.05, ledge), 0.12, rim), mix(mix(ROCK[1], 0.11, ledge), 0.24, rim), mix(mix(ROCK[2], 0.13, ledge), 0.25, rim)];
                let col = [0, 1, 2].map(|i| mix(bare[i], c[i], living) * lit);
                let f = if x < 100.0 { 0.04 + 0.04 * smooth(0.0, 80.0, x) } else { 0.08 };
                set(Mat::Reef, col, f, [x * 0.5, y * 0.9, 0.8 * (-below / 1.5).exp()]);
            }
            for &[dx0, dy0, dr, kind] in &domes {
                let (dx, dy) = (x + 0.5 - dx0, y - dy0);
                let d = dx.hypot(dy * 1.25);
                if d < dr && dy < dr * 0.3 {
                    let nz = (1.0 - (d / dr).powi(2)).max(0.0).sqrt();
                    let lamb = clamp(0.15 + 0.85 * (nz * 0.6 - (dy / dr) * 0.55 + (dx / dr) * 0.25));
                    let groove = 0.75 + 0.25 * (d * 2.4 + 2.0 * fbm(x * 0.3, y * 0.3, 2) * 3.0).sin();
                    let c = CORAL[(kind as usize + 1) % 4];
                    let s = (0.15 + 0.9 * lamb) * groove;
                    let f = if dx0 < 100.0 { 0.04 + 0.04 * smooth(0.0, 80.0, dx0) } else { 0.08 };
                    set(Mat::Reef, c.map(|v| v * s), f, [x * 0.5, y * 0.9, 0.5 * clamp(-dy / dr + 0.6)]);
                }
            }
        }
    }
    // branching coral standing up off both crests: a cell-thick stroke, drawn
    // here at pixel resolution
    for (seed, count, x0, span, right) in [(31, 10, 3.0, 60.0, false), (53, 3, 178.0, 20.0, true)] {
        let mut rnd = Mulberry32::new(seed);
        for i in 0..count {
            let bx = x0 + rnd.next_f64() * span;
            if (bx - fan_c.0).abs() < 3.0 {
                continue; // leave the fan's root clear
            }
            let base = if right { right_top(bx) } else { left_top(bx) };
            let h = 2.0 + rnd.next_f64() * 3.5;
            let lean = (rnd.next_f64() - 0.5) * 0.5;
            let kind = if rnd.next_f64() < 0.5 { 1 } else { 3 };
            let side = if i % 2 == 1 { 1.0 } else { -1.0 };
            let mut s = 0.0;
            while s < h {
                let x = bx + lean * s + if s > h * 0.55 { side * (s - h * 0.55) * 0.6 } else { 0.0 };
                let r = base - s;
                let tip = s / h;
                let sh = 0.4 + 0.6 * tip;
                let (cx, cy) = to_pixel(x, r);
                for py in cy - 1..=cy + 1 {
                    for px in cx - 1..=cx + 1 {
                        if px < 0 || py < 0 || px >= W as i64 || py >= H as i64 {
                            continue;
                        }
                        let (pxu, pyu) = (px as usize, py as usize);
                        if (col_x(pxu) - x).abs() > 0.5 || (row_y(pyu) - 0.5 - r).abs() > 0.5 {
                            continue;
                        }
                        let k = pyu * W + pxu;
                        mat[k] = Mat::Reef;
                        lit_c[k] = CORAL[kind].map(|v| v * sh);
                        fog[k] = 0.06;
                        cuvw[k] = [(col_x(pxu) * 0.5) as f32, ((row_y(pyu) - 0.5) * 0.9) as f32, 0.4];
                    }
                }
                s += 0.25;
            }
        }
    }
    // the water's colour filters what is behind it: reds go first
    let back = (0..n)
        .map(|k| {
            let w = water[k].map(f64::from);
            if mat[k] == Mat::None {
                return water[k];
            }
            let f = f64::from(fog[k]);
            let tint = 1.0 - f;
            let ex = 1.9; // shallow water: the reef takes plenty of light
            let a = lit_c[k];
            [mix(a[0] * ex * (0.55 + 0.45 * tint), w[0], f) as f32, mix(a[1] * ex * (0.85 + 0.15 * tint), w[1], f) as f32, mix(a[2] * ex, w[2], f) as f32]
        })
        .collect();

    // --- the moving things, laid out once ------------------------------------
    let mut rnd = Mulberry32::new(1234);
    let gauss = |rnd: &mut Mulberry32| {
        let mut s = 0.0;
        for _ in 0..4 {
            s += rnd.next_f64();
        }
        (s - 2.0) * 1.7
    };
    let mut fish = vec![];
    for _ in 0..150 {
        let a = gauss(&mut rnd) * 10.0;
        let b = gauss(&mut rnd) * 3.8;
        let ph = rnd.next_f64() * TAU;
        let sp = 0.8 + rnd.next_f64() * 0.6;
        fish.push(Fish { a, b, ph, sp });
    }
    let mut bubbles = vec![];
    for (x0, y0, count) in [(44.0, 52.0, 8), (168.0, 66.0, 7), (116.0, 96.0, 6)] {
        for _ in 0..count {
            let ph = rnd.next_f64();
            let sp = 4.0 + rnd.next_f64() * 3.0;
            let w = rnd.next_f64() * TAU;
            bubbles.push(Bubble { x0, y0, ph, sp, w });
        }
    }
    let mut specks = vec![];
    for _ in 0..90 {
        let x = rnd.next_f64() * TW;
        let y = rnd.next_f64() * TH;
        let sp = 0.3 + rnd.next_f64() * 0.7;
        let ph = rnd.next_f64() * TAU;
        let b = 0.12 + rnd.next_f64() * 0.22;
        specks.push(Speck { x, y, sp, ph, b });
    }

    Box::new(DeepReef { caus: caustics(), water, ray_at, mat, back, fog, cuvw, fish, bubbles, specks, rays: vec![0.0; RAYS], pixels: Pixels::new(&META) })
}

impl DeepReef {
    /// The caustic web at (u, v), in ascii.rest's cells, wrapping.
    fn caus_at(&self, u: f64, v: f64) -> f64 {
        let n = CT as usize * CQ;
        let q = CQ as f64;
        let u = (u * q - 0.5).rem_euclid(n as f64);
        let v = (v * q - 0.5).rem_euclid(n as f64);
        let (x0, y0) = (u as usize % n, v as usize % n);
        let (ax, ay) = (u - u.floor(), v - v.floor());
        let (x1, y1) = ((x0 + 1) % n, (y0 + 1) % n);
        let c = &self.caus;
        let (a, b, cc, d) = (f64::from(c[y0 * n + x0]), f64::from(c[y0 * n + x1]), f64::from(c[y1 * n + x0]), f64::from(c[y1 * n + x1]));
        a + (b - a) * ax + (cc - a) * ay + (a - b - cc + d) * ax * ay
    }

    /// Mixes a colour into a pixel, by `a`.
    fn put_px(&mut self, px: i64, py: i64, c: [f64; 3], a: f64) {
        if px < 0 || py < 0 || px >= W as i64 || py >= H as i64 {
            return;
        }
        let o = self.pixels.get(px as usize, py as usize);
        let m = |i: usize| mix(f64::from(o[i]), c[i], a) as f32;
        self.pixels.set(px as usize, py as usize, [m(0), m(1), m(2)]);
    }

    /// Adds a colour to a pixel.
    fn add_px(&mut self, px: i64, py: i64, c: [f64; 3]) {
        if px < 0 || py < 0 || px >= W as i64 || py >= H as i64 {
            return;
        }
        let o = self.pixels.get(px as usize, py as usize);
        self.pixels.set(px as usize, py as usize, [o[0] + c[0] as f32, o[1] + c[1] as f32, o[2] + c[2] as f32]);
    }
}

impl Piece for DeepReef {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        // shafts: a slow pattern across the surface, shimmering as the waves
        // pass, strongest straight under the sun
        for (u, ray) in self.rays.iter_mut().enumerate() {
            let u = u as f64;
            let s = smooth(0.5, 0.62, fbm(u * 0.06 + t * 0.025, 3.3, 3)).powf(1.2);
            let shimmer = 0.6 + 0.4 * noise(u * 0.18 - t * 0.7, t * 0.3);
            *ray = s * shimmer * (0.35 + 0.65 * (-((u - RAY0) / 60.0).powi(2)).exp());
        }
        let (cx0, cy0, cx1, cy1) = (t * 0.9, t * 0.35, -t * 0.6 + 21.0, t * 0.5 + 9.0);
        // the sun's blaze wobbles as the swell passes over it
        let hx = SUNX + 1.6 * (t * 0.6).sin() + 0.8 * (t * 1.7 + 1.0).sin();

        for py in 0..H {
            let y = row_y(py);
            let r = y - 0.5;
            let depth_fade = (-r / 30.0).exp() * smooth(0.0, 14.0, r + 6.0);
            for px in 0..W {
                let k = py * W + px;
                let x = col_x(px);
                let [mut cr, mut cg, mut cb] = self.back[k].map(f64::from);
                let ray = self.rays[usize::from(self.ray_at[k])] * depth_fade;
                if self.mat[k] == Mat::None {
                    if r < SURF + 6.0 {
                        // the underside of the surface: a rippling ceiling,
                        // pressed flat toward the haze
                        let dist = SURF + 7.0 - y;
                        let q = 30.0 / dist;
                        let a = self.caus_at(((x - 100.0) / dist) * 1.6 + t * 1.6, q * 6.0 + t * 0.8);
                        let b2 = self.caus_at(((x - 100.0) / dist) * 1.2 - t * 1.1 + 31.0, q * 4.5 - t * 0.6 + 17.0);
                        let lit = smooth(SURF + 6.0, 0.0, y).powf(1.3);
                        let c = a.max(b2).powf(1.6);
                        let near = (-((x - hx) / 34.0).powi(2)).exp();
                        // troughs darken the water under them, crests focus light into lines
                        let dim = 1.0 - lit * 0.7 * (1.0 - c);
                        cr *= dim;
                        cg *= dim;
                        cb *= dim;
                        let v = lit * c * (0.1 + 0.42 * near);
                        // the sun: a compact blaze through the surface, broken by the ripples
                        let dx = x + 0.5 - hx;
                        let hot = (-(dx / 8.0).powi(2) - ((y - 6.0) / 3.6).powi(2)).exp() * (1.0 + 0.3 * c);
                        let halo = (-(dx / 15.0).powi(2) - ((y - 6.0) / 7.0).powi(2)).exp() * 0.3 * (0.6 + 0.6 * c);
                        cr += 0.55 * v + 0.9 * hot + 0.35 * halo;
                        cg += 0.85 * v + 1.0 * hot + 0.6 * halo;
                        cb += 0.8 * v + 0.97 * hot + 0.58 * halo;
                    }
                    cr += 0.42 * ray;
                    cg += 0.86 * ray;
                    cb += 0.78 * ray;
                } else {
                    let f = f64::from(self.fog[k]);
                    let [cu, cv, cw] = self.cuvw[k].map(f64::from);
                    if cw > 0.0 {
                        let c = self.caus_at(cu + cx0, cv + cy0).min(1.0) * 0.6 + self.caus_at(cu * 0.8 + cx1, cv * 0.8 + cy1) * 0.6;
                        let s = cw * c * c * (1.0 - f) * (0.6 + 0.6 * ray + 0.3 * depth_fade);
                        // on the sand the light comes back warm, on the reef cool
                        if self.mat[k] == Mat::Sand {
                            cr += 0.62 * s;
                            cg += 0.66 * s;
                            cb += 0.5 * s;
                        } else {
                            cr += 0.55 * s;
                            cg += 0.72 * s;
                            cb += 0.62 * s;
                        }
                    }
                    cr += 0.16 * ray * f;
                    cg += 0.3 * ray * f;
                    cb += 0.28 * ray * f;
                }
                self.pixels.set(px, py, [cr as f32, cg as f32, cb as f32]);
            }
        }

        // the school: one body turning along a slow loop, each fish a beat
        // behind; each a little sprite turned along its heading
        let path_x = |s: f64| 95.0 + 24.0 * (0.11 * s + 0.4).sin();
        let path_y = |s: f64| 44.0 + 8.0 * (0.17 * s + 2.2).sin();
        for i in 0..self.fish.len() {
            let Fish { a, b, ph, sp } = self.fish[i];
            let s = t - a * 0.08;
            let (vx, vy) = (24.0 * 0.11 * (0.11 * s + 0.4).cos(), 8.0 * 0.17 * (0.17 * s + 2.2).cos());
            let th = vy.atan2(vx);
            let (ct, st) = (th.cos(), th.sin());
            let wob = (t * 1.3 * sp + ph).sin();
            let fx = path_x(s) + a * ct - b * st + wob * 0.6;
            let fy = path_y(s) + a * st + b * ct + (t * sp + ph).cos() * 0.4;
            let h = th + 0.15 * wob;
            let (dx, dy) = (h.cos(), h.sin());
            // dark shapes against the light, until a fish turns its silver flank to it
            let flash = smooth(0.72, 0.97, (th * 1.6 + a * 0.35 + b * 0.4 - t * 0.5 + 0.6).sin().abs());
            let body = [mix(0.008, 0.85, flash), mix(0.018, 0.97, flash), mix(0.026, 1.0, flash)];
            // in pixels: the head, two of body, then a forked tail beating
            let (hx, hy) = ((fx + 0.5) * S - 0.5, (fy + 0.5) * S - 0.5);
            let tail = 0.6 * (t * 6.0 * sp + ph).sin();
            for (j, side, alpha) in [(0.0, 0.0, 1.0), (1.0, 0.0, 1.0), (2.0, 0.0, 0.9), (3.0, 1.0 + tail, 0.75), (3.0, -1.0 + tail, 0.75)] {
                let (qx, qy) = (hx - dx * j * 0.9 - dy * side * 0.8, hy - dy * j * 0.9 + dx * side * 0.8);
                self.put_px(qx.round() as i64, qy.round() as i64, body, alpha);
            }
        }

        // kelp: dark fronds framing the view, a gold edge only where a shaft hits
        for &[bx, base, len, sz, ph, haze] in &KELP {
            let side = if bx < 100.0 { 1.0 } else { -1.0 }; // which way the sun lies
            for py in 0..H {
                let r = row_y(py) - 0.5;
                if r > base - 1.0 + 0.5 || r < base - len - 0.5 {
                    continue;
                }
                let s01 = (base - r) / len;
                let lean = side * 2.5 * sz * s01 * s01;
                // the swell runs up the frond, so it bends in an S rather than
                // tipping like a stick
                let bend = (t * 0.55 + ph - s01 * 4.2).sin() * 2.2 * sz * s01.powf(1.2) + (t * 0.21 + ph).sin() * 1.5 * s01 + lean;
                let x = bx + bend;
                // the frond: a stipe with blades off alternate sides, each blade
                // a lobe that swells and tapers, trailing a little behind the
                // stipe's sway
                let stem = 0.8 + 0.4 * sz * (1.0 - s01);
                let beat = s01 * len * 0.32 + ph;
                let lobe_l = beat.sin().max(0.0).powf(1.2) * sz * 2.3 * (1.0 - 0.35 * s01);
                let lobe_r = (-beat.sin()).max(0.0).powf(1.2) * sz * 2.3 * (1.0 - 0.35 * s01);
                let drag = (t * 0.55 + ph - s01 * 4.2).cos() * 0.8 * sz;
                let (x0, x1) = (x - stem - lobe_l + drag.min(0.0), x + stem + lobe_r + drag.max(0.0));
                let (cx, _) = to_pixel(x, r);
                let ray = self.rays[usize::from(self.ray_at[py * W + cx.clamp(0, W as i64 - 1) as usize])] * (-r / 30.0).exp();
                let lit = smooth(0.3, 0.6, ray);
                let pa = (x0 * S - 0.5).ceil() as i64;
                let pb = ((x1 + 1.0) * S - 0.5).floor() as i64;
                for px in pa..=pb {
                    if px < 0 || px >= W as i64 {
                        continue;
                    }
                    let k = py * W + px as usize;
                    let xx = col_x(px as usize);
                    // backlit: dark through the middle, the edges glowing, the
                    // sun-facing edge most of all and gold where a shaft catches it
                    let sun = if side > 0.0 { px == pb } else { px == pa };
                    let rim = px == pa || px == pb;
                    let blade = if (xx - x).abs() > stem { 1.0 } else { 0.0 };
                    // the stipe is black, the blades let a little olive light through
                    let through = blade * 0.12 * smooth(0.0, 1.0, (xx - x).abs() - stem) * (0.5 + 0.5 * hash(px as f64 * 7.0, py as f64 * 3.0));
                    let g = 0.02 + through + if rim { 0.16 } else { 0.0 } + if sun { 0.34 + 0.66 * lit * (0.4 + 0.6 * blade) } else { 0.0 };
                    let w = self.water[k].map(f64::from);
                    self.put_px(px, py as i64, [mix(0.62 * g, w[0], haze), mix(0.58 * g, w[1], haze), mix(0.18 * g, w[2], haze)], 1.0);
                }
            }
        }

        // bubbles: wobbling up to the surface, growing as they rise
        for i in 0..self.bubbles.len() {
            let Bubble { x0, y0, ph, sp, w } = self.bubbles[i];
            let span = y0 - SURF;
            let p = ((t * sp) / span + ph) % 1.0;
            let y = y0 - p * span;
            let x = x0 + (y * 0.35 + w).sin() * 1.2 + p * 3.0;
            let v = 0.55 + 0.45 * p;
            let c = [0.7 * v, 0.95 * v, v];
            let (px, py) = to_pixel(x, y);
            self.add_px(px, py, c);
            if p > 0.5 {
                // a bigger bubble: a rim round a bright point
                let e = c.map(|v| v * 0.45);
                for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    self.add_px(px + ox, py + oy, e);
                }
            }
        }
        // specks drifting in the water
        for i in 0..self.specks.len() {
            let Speck { x, y, sp, ph, b } = self.specks[i];
            let x = (x + t * sp + 2.0 * (t * 0.3 + ph).sin()).rem_euclid(TW);
            let y = (y + t * sp * 0.4).rem_euclid(TH);
            self.add_px((x * S) as i64, (y * S) as i64, [b * 0.7, b * 0.9, b]);
        }
        self.pixels.draw(grid, env);
    }
}
