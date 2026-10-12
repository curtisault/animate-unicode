//! lantern lake: sky lanterns rising over a still mountain lake at night. A
//! festival on the far shore lets them go by the hundred: they climb from
//! behind the trees, lean on a slow breeze and burn out high over the water.
//! Nearer ones sail up past us, and a figure standing in a boat, a lamp on its
//! bow, holds one up, lets it go, and lights the next. After ascii.rest's
//! lantern-lake.ts (MIT, @bas3line).
//!
//! The mountains, the festival's glow, the lake's broken mirror, the boat and
//! every lantern's flight are ascii.rest's, shaded here per pixel instead of
//! per cell. Where ascii.rest draws each of 200 × 100 square cells as a dot as
//! big as it is bright, this draws 320 × 80 cells of two square pixels each,
//! an upper half block over a back colour ([`crate::pixels`]): 1.6 times the
//! detail each way, solid colour, and gradients dithered in colour rather
//! than in dot size. So a lantern is drawn at its true size in pixels (the
//! far ones a pixel or two, the near ones a shaped paper body lit through),
//! and the lake mirrors the lanterns as well as the sky, each reflection
//! broken by the ripples. The far range's slate is written as a greyed copy
//! of its own shade rather than picked from three tones. The palette adds the
//! dark tones a solid pixel needs where a small dot was enough. In one ink
//! each pixel is octant dots, as many as it is bright.

use crate::noise::{clamp, fbm, hash, mix, noise, smooth};
use crate::pixels::{Pixels, Rgb};
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// Lantern lake's meta.
pub static META: Meta = Meta {
    name: "lantern lake",
    slug: "lantern-lake",
    category: Category::Nights,
    note: "sky lanterns rising over a still mountain lake, one let go from a boat",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: night blues, violets, mauves and roses to the festival's
    // ambers, the lanterns' oranges and golds, embers, greys and browns, the
    // far range's slate and the stars. Then the night's darks.
    palette: Some(&[
        "#141b30", "#1c2a52", "#22346a", "#2a4080", "#324c96", "#3c58a8", "#33306e", "#433c88", "#54449a", "#62407e", "#7a4a88", "#934e86", "#a4546e",
        "#bc5e66", "#d06a62", "#dc7650", "#e88444", "#f09238", "#d4581a", "#ea6c1e", "#f78226", "#ff9530", "#ffa83c", "#ffbb4e", "#ffcc66", "#ffdc86",
        "#ffe8a8", "#fff3cf", "#fffbec", "#b8341a", "#d84420", "#2f3748", "#4f586c", "#5e5a7e", "#7a6a8c", "#5a3220", "#7c4628", "#4a5578", "#5d6a92",
        "#7381ab", "#c4cdec", "#e4e9fb", "#ffffff", "#04060d", "#070a16", "#0b1020", "#0a1230", "#0e1a40", "#13215a", "#161437", "#211b4a", "#2b1a3c",
        "#3e2440", "#1a1c2a", "#252b3e", "#323a54", "#140c0a", "#26140c", "#3c200e", "#5a2a0e", "#7c3610", "#8e2c14",
    ]),
    ground: Some("#04060d"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame, in its cells.
const SW: f64 = 200.0;
const SH: f64 = 100.0;
/// The waterline.
const WL: f64 = 64.0;
/// The pixel rows above the waterline: the sky, the land, and what the lake mirrors.
const SKY: usize = 102;
/// The middle of the festival on the far shore.
const SRC: f64 = 100.0;
/// The boat's middle and its waterline, far enough out that the lantern held
/// over its head sits against the dark trees' reflection.
const BX: f64 = 108.5;
const BW: f64 = 90.0;
/// The boat's scale.
const SC: f64 = 1.4;
/// Where the figure stands in it.
const FX: f64 = BX - 3.0 * SC;
/// The figure lets a lantern go this far into every cycle and lights the
/// next one at LIT, so it holds one up most of the time.
const CYCLE: f64 = 20.0;
const LET_GO: f64 = 3.0;
const LIT: f64 = 6.0;
/// The boat's lanterns, in cells tall.
const HELD: f64 = 5.0 * SC;
/// The range of a twinkle's or a sway's phase: ascii.rest's 6.28, kept so
/// every phase is the TypeScript's.
#[allow(clippy::approx_constant)]
const PHASE: f64 = 6.28;
const GROUND: Rgb = [4.0 / 255.0, 6.0 / 255.0, 13.0 / 255.0];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Air,
    Far,
    Back,
    Peak,
    Tree,
}

/// The scene's x, in ascii.rest's columns, at the middle of pixel column `px`.
fn xc(px: usize) -> f64 {
    (px as f64 + 0.5) / S
}

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn yc(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

/// The pixel holding scene coordinate `v`, which may be off the picture.
fn pix(v: f64) -> i64 {
    (v * S).floor() as i64
}

/// Distance from a point to a segment.
fn seg(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    let (dx, dy) = (bx - ax, by - ay);
    let k = clamp(((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy));
    (ax + dx * k - px).hypot(ay + dy * k - py)
}

/// Ranges as tent peaks [x, height, slope], roughened: the high peaks either
/// side, the nearer shoulders in front of them, and a distant range between.
const PEAKS: [[f64; 3]; 4] = [[22.0, 42.0, 1.2], [-4.0, 33.0, 0.9], [172.0, 33.0, 1.05], [215.0, 22.0, 0.6]];
const SHOULDERS: [[f64; 3]; 4] = [[50.0, 25.0, 1.0], [64.0, 12.0, 0.75], [150.0, 17.0, 0.95], [196.0, 21.0, 0.75]];
const DISTANT: [[f64; 3]; 4] = [[98.0, 17.0, 0.42], [122.0, 14.0, 0.5], [82.0, 12.0, 0.45], [140.0, 11.0, 0.5]];

/// A range's height at x and the peak it belongs to.
fn range(x: f64, peaks: &[[f64; 3]], seed: f64, rough: f64) -> (f64, f64) {
    let (mut m, mut px) = (-99.0, 0.0);
    for &[cx, h, s] in peaks {
        let v = h - (x - cx).abs() * s;
        if v > m {
            (m, px) = (v, cx);
        }
    }
    let j = rough * (fbm(x * 0.09, seed, 3) - 0.5) + rough * 0.45 * (noise(x * 0.45, seed + 5.0) - 0.5);
    (m + j * (m.max(0.0) / 6.0).min(1.0), px)
}

/// A sky lantern's outline, in units of its height from its top: wider at the
/// shoulders than at the open bottom, the shoulders rounded.
fn inside(u: f64, v: f64) -> bool {
    if !(0.0..=1.0).contains(&v) {
        return false;
    }
    let mut hw = 0.34 - 0.08 * v;
    if v < 0.14 {
        hw *= (1.0 - 0.6 * ((0.14 - v) / 0.14).powi(2)).sqrt();
    }
    u.abs() <= hw
}

/// The festival's light: a warm dome in the air over the middle of the far
/// shore, leaning the way the breeze carries the lanterns.
fn dome(x: f64, y: f64) -> f64 {
    let up = (WL - 6.0 - y).max(0.0); // from the tops of the trees
    let dx = (x - SRC - 0.3 * up) / (44.0 + up);
    (-dx * dx).exp() * (0.6 * (-up / 14.0).exp() + 0.4 * (-up / 40.0).exp())
}

/// The sky's hue by how warm it is: night blue, through violet, mauve and
/// rose, to amber right over the trees.
const HUES: [[f64; 3]; 6] = [[0.22, 0.36, 1.0], [0.46, 0.36, 1.0], [0.76, 0.42, 0.92], [1.0, 0.48, 0.58], [1.0, 0.54, 0.34], [1.0, 0.58, 0.2]];

fn hue(w: f64) -> [f64; 3] {
    let f = clamp(w) * 5.0;
    let i = (f.floor() as usize).min(4);
    let k = f - i as f64;
    [mix(HUES[i][0], HUES[i + 1][0], k), mix(HUES[i][1], HUES[i + 1][1], k), mix(HUES[i][2], HUES[i + 1][2], k)]
}

/// A mountainside's colour at (x, y): the faces turned toward the festival
/// take a little of its light, the ridge between them wandering as it comes
/// down from the peak, and the ridgeline catches the sky behind it. Also
/// returns the festival's light on it.
#[allow(clippy::too_many_arguments)]
fn slope(x: f64, y: f64, top: f64, px: f64, haze: f64, relief: f64, edge: f64, warm: f64) -> ([f64; 3], f64) {
    let xc = x + 0.5;
    let depth = y - top;
    let inward = if px < SRC { 1.0 } else { -1.0 };
    let ridge = px + (depth + 1.0) * 0.6 * (noise(y * 0.1, px) - 0.5);
    let face = smooth(-4.0, 4.0, (xc - ridge) * inward);
    let side = if xc - px < 0.0 { -1.0 } else { 1.0 };
    let u = x + depth * 0.45 * side;
    let rib = |q: f64| fbm(q * 0.13, px * 0.37, 3);
    let grad = (rib(u + 1.0) - rib(u - 1.0)) * 7.0 * inward;
    let lit = clamp(0.3 + 0.5 * face + grad * 0.5);
    let tex = 0.8 + 0.4 * fbm(x * 0.35, y * 0.35, 2);
    let w = lit * (-((xc - SRC) / 70.0).powi(2)).exp() * (0.4 + 0.6 * smooth(top + 2.0, WL, y)) * 0.9;
    let b = (haze + relief * lit * lit) * tex + 0.16 * w;
    let rim = smooth(1.8, 0.3, depth) * edge;
    let (hw, hs) = (hue(w * warm), hue(dome(xc, top - 1.0)));
    ([b * hw[0] + rim * hs[0], b * hw[1] + rim * hs[1], b * hw[2] + rim * hs[2]], w)
}

/// The far shore's scrub line at x.
fn scrub(x: f64) -> f64 {
    WL - 2.4 - 1.4 * fbm(x * 0.15, 5.5, 3)
}

/// A lantern let go on the far shore.
struct Flight {
    i: f64,
    /// size, in cells tall
    s: f64,
    /// rise, cells a second
    v: f64,
    /// seconds from one start to the next
    life: f64,
    off: f64,
    /// lean on the breeze, cells across for each row climbed
    wd: f64,
    sw: f64,
    sf: f64,
    ph: f64,
    f1: f64,
    f2: f64,
    /// sails off the top rather than burning out
    out: bool,
    /// starts anywhere along the shore, not just at the festival
    wide: bool,
}

impl Flight {
    fn start_x(&self, lap: f64) -> f64 {
        let n = self.i * 7.0 + lap * 131.0;
        if self.wide {
            12.0 + 176.0 * hash(n, 23.0)
        } else {
            SRC - 6.0 + 30.0 * (hash(n, 23.0) + hash(n, 24.0) + hash(n, 25.0) - 1.5)
        }
    }
}

/// A lantern let go near us, below the frame.
struct Close {
    i: f64,
    s: f64,
    v: f64,
    x: f64,
    path: f64,
    life: f64,
    off: f64,
    wd: f64,
    sw: f64,
    sf: f64,
    ph: f64,
}

/// [size, where it is across and down at the start]; the last few are still
/// below the frame, on their way up.
const NEAR: [[f64; 3]; 11] = [
    [12.5, 34.0, 30.0],
    [8.5, 178.0, 14.0],
    [6.5, 66.0, 74.0],
    [5.2, 154.0, 46.0],
    [4.4, 18.0, 86.0],
    [4.6, 136.0, 4.0],
    [9.5, 190.0, 116.0],
    [10.5, 150.0, 160.0],
    [5.8, 80.0, 128.0],
    [7.0, 28.0, 190.0],
    [4.2, 172.0, 150.0],
];

/// The figure's [shoulder, elbow, hand] for each pose, one arm; the other mirrors it.
type Arm = [[f64; 2]; 3];
const UP: Arm = [[1.3, 9.4], [3.0, 11.5], [2.0, 14.1]];
const DOWN: Arm = [[1.3, 9.4], [2.0, 6.6], [1.9, 4.4]];
const CHEST: Arm = [[1.3, 9.4], [2.4, 7.2], [1.2, 6.0]];

/// A long low hull's top, its ends sweeping up, the bow (on the right) higher.
fn gunwale(dx: f64) -> f64 {
    2.4 + if dx > 0.0 { 3.0 } else { 1.8 } * smooth(4.0, 12.5, dx.abs()).powi(2)
}

fn hull(dx: f64, up: f64) -> bool {
    up <= gunwale(dx) && up > -0.6 && dx.abs() <= 12.5 - (1.8 - up).max(0.0) * 2.4
}

fn body(dx: f64, up: f64, arms: &Arm) -> bool {
    if dx * dx / 1.3 + (up - 11.2).powi(2) / 1.5 < 1.0 {
        return true; // the head
    }
    if up <= 9.8 && up > 1.0 && dx.abs() <= 1.35 + 0.5 * smooth(9.8, 2.0, up) {
        return true; // a long coat
    }
    let ax = dx.abs();
    seg(ax, up, arms[0][0], arms[0][1], arms[1][0], arms[1][1]) < 0.66 || seg(ax, up, arms[1][0], arms[1][1], arms[2][0], arms[2][1]) < 0.58
}

/// The working picture: colour, and how much lantern paint lies on each pixel.
struct Canvas {
    c: Vec<[f64; 3]>,
    painted: Vec<f32>,
}

impl Canvas {
    #[inline]
    fn paint(&mut self, k: usize, p: [f64; 3], a: f64) {
        let c = &mut self.c[k];
        for j in 0..3 {
            c[j] += (p[j] - c[j]) * a;
        }
        self.painted[k] = self.painted[k].max(a as f32);
    }

    /// Draws a lantern `h` cells tall centred on (cx, cy) at brightness `i`
    /// and opacity `a`, above pixel row `limit`, leaving out the pixels
    /// `hide` marks.
    #[allow(clippy::too_many_arguments)]
    fn lantern(&mut self, cx: f64, cy: f64, h: f64, i: f64, a: f64, limit: usize, hide: Option<&[bool]>) {
        let hidden = |k: usize| hide.is_some_and(|m| k < m.len() && m[k]);
        if h < 2.0 {
            // a pixel wide: a spark, or the paper above and the burner's
            // glow below, spread over the rows it straddles so it slides up
            // rather than jumping
            let x = pix(cx);
            if x < 0 || x >= W as i64 {
                return;
            }
            let hp = (h * S).max(1.0);
            let top = cy * S - hp / 2.0;
            for r in top.floor() as i64..=(top + hp).floor() as i64 {
                if r < 0 || r >= limit as i64 {
                    continue;
                }
                let k = r as usize * W + x as usize;
                if hidden(k) {
                    continue;
                }
                let rf = r as f64;
                if h < 1.4 {
                    let cov = ((rf + 1.0).min(top + hp) - rf.max(top)).max(0.0);
                    if cov < 0.02 {
                        continue;
                    }
                    let q = i * (0.85 + 0.15 * h);
                    self.paint(k, [q, q * (0.42 + 0.3 * q), q * (0.1 + 0.18 * q * q)], a * cov);
                } else {
                    let mid = top + hp / 2.0;
                    let up = ((rf + 1.0).min(mid) - rf.max(top)).max(0.0);
                    let dn = ((rf + 1.0).min(top + hp) - rf.max(mid)).max(0.0);
                    if up + dn < 0.02 {
                        continue;
                    }
                    let q = i * (0.8 * up + 1.1 * dn) / (up + dn);
                    self.paint(k, [q, q * (0.34 + 0.36 * q), q * (0.06 + 0.2 * q * q)], a * (up + dn));
                }
            }
            return;
        }
        let top = cy - h / 2.0;
        let x0 = pix(cx - 0.36 * h).max(0);
        let x1 = pix(cx + 0.36 * h).min(W as i64 - 1);
        let r0 = pix(top).max(0);
        let r1 = pix(top + h).min(limit as i64 - 1);
        for r in r0..=r1 {
            for x in x0..=x1 {
                let (r, x) = (r as usize, x as usize);
                let k = r * W + x;
                if hidden(k) {
                    continue;
                }
                let mut cov = 0;
                for sy in 0..3 {
                    for sx in 0..3 {
                        let (qx, qy) = ((x as f64 + (sx as f64 + 0.5) / 3.0) / S, (r as f64 + (sy as f64 + 0.5) / 3.0) / S);
                        if inside((qx - cx) / h, (qy - top) / h) {
                            cov += 1;
                        }
                    }
                }
                if cov == 0 {
                    continue;
                }
                let u = (xc(x) - cx) / h;
                let v = clamp((yc(r) - top) / h);
                let xn = (u.abs() / (0.34 - 0.08 * v)).min(1.0);
                // Lit through: past full brightness, so the body is solid, its
                // colour deep orange up under the crown and pale gold over the
                // burner.
                let q = i * (1.0 - 0.3 * xn * xn) * (0.78 + 0.42 * v);
                let fl = (-(u / 0.13).powi(2) - ((v - 0.9) / 0.1).powi(2)).exp() * i;
                self.paint(k, [q + fl * 0.3, q * (0.3 + 0.42 * q) + fl * 0.6, q * (0.04 + 0.2 * q * q) + fl * 0.5], f64::from(cov) / 9.0 * a);
            }
        }
    }

    /// The warm air round a lantern.
    fn halo(&mut self, cx: f64, cy: f64, h: f64, g: f64, limit: usize) {
        let e = 0.4 + 0.32 * h;
        let reach = (e * 4.0).ceil();
        let r0 = pix(cy - reach).max(0);
        let r1 = pix(cy + reach).min(limit as i64 - 1);
        let x0 = pix(cx - reach).max(0);
        let x1 = pix(cx + reach).min(W as i64 - 1);
        for r in r0..=r1 {
            let dy = yc(r as usize) - cy;
            for x in x0..=x1 {
                let dx = xc(x as usize) - cx;
                let d = (dx.hypot(dy) - 0.3 * h).max(0.0);
                let q = g * ((-d / e).exp() + 0.25 * (-d / (e * 2.5)).exp());
                let c = &mut self.c[r as usize * W + x as usize];
                c[0] += q;
                c[1] += q * 0.5;
                c[2] += q * 0.17;
            }
        }
    }
}

/// The lake: the sky and the land, built once, every lantern's flight, the
/// boat, and the picture.
pub struct LanternLake {
    /// The sky and the land above the waterline, before the festival breathes.
    sky: Vec<[f64; 3]>,
    /// How much of the festival's warm light each pixel takes.
    take: Vec<f32>,
    /// The far range's pixels, drawn in slate.
    far: Vec<bool>,
    /// The far shore's trees, which hide the lanterns' starts.
    tree: Vec<bool>,
    shore: Vec<Flight>,
    close: Vec<Close>,
    /// People on the shore with lanterns still in their hands: [x, row, phase].
    shore_lights: Vec<(f64, f64, f64)>,
    /// [pixel, phase, rate, brightness]
    stars: Vec<(usize, f64, f64, f64)>,
    /// The scene rows of the water that break the reflection into strips.
    gap: Vec<bool>,
    /// The boat's box in pixels, [x, y, w, h], its hull, and the hull and
    /// figure this frame (1 boat, 2 figure).
    bx: [usize; 4],
    hulls: Vec<bool>,
    shape: Vec<u8>,
    canvas: Canvas,
    pixels: Pixels,
}

/// A lantern lake; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = SKY * W;
    let mut mat = vec![Mat::Air; n];
    let mut sky = vec![[0.0; 3]; n];
    let mut take = vec![0.0f32; n];
    let mut far = vec![false; n];
    for px in 0..W {
        let x = xc(px) - 0.5;
        let (hb, pb) = range(x + 0.5, &PEAKS, 3.1, 4.0);
        let (hf, pf) = range(x + 0.5, &SHOULDERS, 7.7, 3.0);
        let (hd, _) = range(x + 0.5, &DISTANT, 11.3, 2.2);
        let (back_top, front_top, far_top) = (WL - hb.max(0.0), WL - hf.max(0.0), WL - hd.max(0.0));
        for py in 0..SKY {
            let k = py * W + px;
            let y = yc(py);
            if y >= front_top {
                mat[k] = Mat::Peak;
                sky[k] = slope(x, y, front_top, pf, 0.05, 0.06, 0.3, 1.0).0;
            } else if y >= back_top {
                mat[k] = Mat::Back;
                let (c, w) = slope(x, y, back_top, pb, 0.24, 0.3, 0.35, 0.5);
                sky[k] = c;
                // slate, but not where the festival lights it or on the
                // ridgeline, which keeps the sky's own blue
                far[k] = w < 0.3 && y - back_top > 1.2;
            } else if y >= far_top {
                mat[k] = Mat::Far;
                // a distant range behind the festival, veiled in the warm haze
                // its light makes in the air in front, dark up on the ridge
                let tex = fbm(x * 0.2, y * 0.3, 3);
                let w = dome(x + 0.5, y);
                let b = 0.06 + 0.04 * tex + 0.3 * w + 0.7 * w.powi(3);
                let h = hue(w);
                sky[k] = [b * h[0], b * h[1], b * h[2]];
            } else {
                // night blue overhead, lighter on the horizon, warmed over the festival
                let v = (y / WL).powf(1.5);
                let hz = 0.85 + 0.3 * fbm(x * 0.04, (y - 0.5) * 0.07, 3);
                let w = dome(x + 0.5, y);
                let b = (0.18 + 0.32 * v) * hz + 0.5 * w;
                let h = hue(w);
                sky[k] = [b * h[0], b * h[1], b * h[2]];
                take[k] = w as f32;
            }
        }
    }

    // The far shore's trees: a low band of scrub along the water with spruce
    // standing out of it, shorter in the clearing where the festival is.
    for py in 0..SKY {
        for px in 0..W {
            if yc(py) >= scrub(xc(px)) {
                mat[py * W + px] = Mat::Tree;
            }
        }
    }
    let mut x = -2.0f64;
    while x < SW + 2.0 {
        let h = hash((x * 7.0).floor(), 21.0);
        let clearing = smooth(22.0, 0.0, (x - SRC).abs());
        let th = (2.6 + 4.4 * hash((x * 3.0).floor(), 5.0).powf(1.2)) * (1.0 - 0.3 * clearing);
        let (tx, tw, tb) = (x + hash(x.floor(), 2.0), 1.0 + 0.8 * hash(x.floor(), 4.0), scrub(x) + 1.0);
        for py in pix(tb - th).max(0) as usize..SKY {
            let y = yc(py);
            let dy = y - (tb - th);
            if dy < 0.0 || y >= tb + 0.5 {
                continue;
            }
            let tier = (dy + th * 0.3) / 1.6;
            let w = (dy / th) * tw * (0.7 + 0.45 * (tier - tier.floor())) + 0.3;
            for px in pix(tx - tw - 1.0).max(0) as usize..=(pix(tx + tw + 1.0).min(W as i64 - 1)).max(0) as usize {
                if (xc(px) - tx).abs() <= w {
                    mat[py * W + px] = Mat::Tree;
                }
            }
        }
        x += 1.2 + 1.8 * h;
    }
    let tree: Vec<bool> = mat.iter().map(|&m| m == Mat::Tree).collect();
    for py in 0..SKY {
        for px in 0..W {
            let k = py * W + px;
            if !tree[k] {
                continue;
            }
            let h = hash(px as f64 * 13.0 + py as f64, 5.0);
            // the tops of the trees, backlit by the festival behind them: how
            // far down from the top of the tree this pixel is
            let depth = (0..4).take_while(|&d| py > d && tree[k - (d + 1) * W]).count();
            let edge = [1.0, 0.74, 0.35, 0.0, 0.0][depth];
            let g = edge * dome(xc(px), yc(py)) * 0.45;
            sky[k] = [0.004 + 0.004 * h + g, 0.006 + 0.006 * h + g * 0.5, 0.012 + 0.008 * h + g * 0.18];
            take[k] = (edge * 0.5) as f32;
            far[k] = false;
        }
    }

    // people on the shore with lanterns still in their hands
    let mut shore_lights = vec![];
    let mut x = SRC - 24.0;
    for i in 0..8 {
        let i = f64::from(i);
        shore_lights.push((x.round(), WL - 1.0 - if hash(i, 62.0) > 0.6 { 1.0 } else { 0.0 }, hash(i, 63.0) * PHASE));
        x += 2.0 + 9.0 * hash(i, 61.0).powi(2);
    }

    // stars, a pixel big, thinning out where the festival lights the sky
    let mut stars = vec![];
    for k in 0..n {
        if mat[k] != Mat::Air {
            continue;
        }
        let h = hash((k % W) as f64, (k / W) as f64 + 101.0);
        let tk = f64::from(take[k]);
        if h > 0.994 && tk < 0.25 {
            let kf = k as f64;
            stars.push((k, hash(kf, 3.0) * PHASE, 0.8 + hash(kf, 4.0) * 2.5, (0.45 + (h - 0.994) * 100.0) * (1.0 - tk * 2.5)));
        }
    }
    let gap = (0..SH as usize).map(|r| r as f64 >= WL && hash(r as f64, 404.0) < 0.3).collect();

    // Let go on the far shore. Each climbs from below the scrub, so its start
    // is hidden, and either burns out or sails off the top before it starts
    // again.
    let mut shore: Vec<Flight> = (0..240)
        .map(|i| {
            let f = f64::from(i);
            let near = i >= 196;
            let s = if near { 1.5 + 1.7 * hash(f, 11.0).powf(1.5) } else { 0.45 + 0.9 * hash(f, 11.0).powf(1.4) };
            let v = 0.3 + 0.62 * s;
            let out = near && hash(f, 12.0) < 0.7;
            let climb = if out { WL + 3.0 * s + 4.0 } else { 12.0 + 46.0 * hash(f, 13.0).powf(0.8) };
            Flight {
                i: f,
                s,
                v,
                life: (climb + s + 1.0) / v,
                off: hash(f, 14.0),
                wd: 0.12 + 0.3 * hash(f, 15.0),
                sw: 0.25 * s + 0.2,
                sf: 0.3 + 0.5 * hash(f, 16.0),
                ph: hash(f, 17.0) * PHASE,
                f1: 2.0 + 3.0 * hash(f, 18.0),
                f2: 5.0 + 4.0 * hash(f, 19.0),
                out,
                wide: near || hash(f, 20.0) < 0.08,
            }
        })
        .collect();
    shore.sort_by(|a, b| a.s.total_cmp(&b.s));

    // Let go near us, below the frame: they rise past large and fast.
    let close = NEAR
        .iter()
        .enumerate()
        .map(|(i, &[s, x, y])| {
            let i = i as f64;
            let v = 0.35 + 0.42 * s;
            let path = SH + 5.0 * s; // from below the frame to above it, glow and all
            let life = (path / v) * (1.08 + 0.3 * hash(i, 41.0));
            Close {
                i,
                s,
                v,
                x,
                path,
                life,
                off: (SH + 2.5 * s - y) / v / life,
                wd: s * 0.05,
                sw: 0.35 * s,
                sf: 0.2 + 0.2 * hash(i, 42.0),
                ph: hash(i, 43.0) * PHASE,
            }
        })
        .collect();

    // The boat and the figure, drawn into a box of pixels round the boat.
    let x0 = pix(BX - 13.0 * SC).max(0) as usize;
    let y0 = pix(BW - 16.0 * SC) as usize;
    let bx = [x0, y0, (pix(BX + 14.0 * SC) as usize).min(W - 1) - x0 + 1, (pix(BW + 2.0) as usize).min(H - 1) - y0 + 1];
    let mut hulls = vec![false; bx[2] * bx[3]];
    for j in 0..bx[3] {
        for i in 0..bx[2] {
            hulls[j * bx[2] + i] = hull((xc(bx[0] + i) - BX) / SC, (BW - yc(bx[1] + j)) / SC);
        }
    }
    let shape = vec![0; bx[2] * bx[3]];
    let canvas = Canvas { c: vec![[0.0; 3]; W * H], painted: vec![0.0; W * H] };
    Box::new(LanternLake { sky, take, far, tree, shore, close, shore_lights, stars, gap, bx, hulls, shape, canvas, pixels: Pixels::new(&META) })
}

impl LanternLake {
    /// The hull and the figure, its arms `k` of the way from pose `a` to `b`.
    fn pose(&mut self, a: &Arm, b: &Arm, k: f64) {
        let mut arms = [[0.0; 2]; 3];
        for i in 0..3 {
            arms[i] = [mix(a[i][0], b[i][0], k), mix(a[i][1], b[i][1], k)];
        }
        let [x0, y0, bw, bh] = self.bx;
        for j in 0..bh {
            for i in 0..bw {
                let q = j * bw + i;
                self.shape[q] = if self.hulls[q] {
                    1
                } else if body((xc(x0 + i) - FX) / SC, (BW - yc(y0 + j)) / SC, &arms) {
                    2
                } else {
                    0
                };
            }
        }
    }

    /// The lake: the sky upside down, stretched and broken by slow ripples,
    /// the festival's road on it and the shore lights.
    fn lake(&mut self, t: f64) {
        let c = &mut self.canvas.c;
        for py in SKY..H {
            let y = yc(py);
            let r = y - 0.5;
            let dw = y - WL;
            let deep = mix(1.0, 0.65, dw / (SH - WL));
            // the rows mirrored, and the two above them, smearing it downward
            let ry = 2.0 * WL - y;
            let row = |o: f64| (pix(ry - o).clamp(0, SKY as i64 - 1) as usize) * W;
            let (r0, r1, r2) = (row(0.0), row(1.0), row(2.0));
            let fade = smooth(SH + 4.0, SH - 7.0, y);
            let gap = self.gap[(y.floor() as usize).min(SH as usize - 1)];
            // the festival's light laid on the water as a broad road, widening toward us
            let (rx, rw) = (SRC + 4.0 + dw * 0.12, 7.0 + dw * 0.9);
            // near the shore the mirror carries the festival's light; the road
            // takes over further out
            let ra = 0.9 * smooth(3.0, 16.0, dw) * (-dw / 60.0).exp();
            for px in 0..W {
                let x = xc(px) - 0.5;
                let wave = noise(x * 0.04 + t * 0.05, r * 0.55 - t * 0.35);
                let sx = x + (0.45 + dw * 0.075) * (r * 1.3 + t * 1.6 + wave * 4.0).sin();
                let sp = (sx + 0.5) * S - 0.5;
                let ix = sp.floor();
                let fx = sp - ix;
                let ix = (ix as i64).clamp(0, W as i64 - 2) as usize;
                // long level streaks where the ripples face us, darker troughs between
                let streak = smooth(0.25, 0.75, noise(x * 0.035 + t * 0.04, r * 0.75 - t * 0.3));
                let mut kr = 0.9 * (0.5 + 0.5 * streak) * (0.85 + 0.25 * wave) * deep;
                if gap && wave < 0.45 {
                    kr *= 0.2;
                }
                let gx = 1.0 - fx;
                let mut m = [0.0; 3];
                for (j, mj) in m.iter_mut().enumerate() {
                    let at = |o: usize| c[o + ix][j] * gx + c[o + ix + 1][j] * fx;
                    *mj = (at(r0) * 0.5 + at(r1) * 0.3 + at(r2) * 0.2) * kr;
                }
                let (mut cr, mut cg, mut cb) = (m[0] + 0.006, m[1] + 0.012, m[2] + 0.028);
                // the festival's road: a glow on the water, broken into short
                // level dashes where the ripples catch it
                let mut g = 0.0;
                let qx = (x + 0.5 - rx) / rw;
                if qx > -2.2 && qx < 2.2 {
                    let dash = smooth(0.42, 0.72, noise(x * 0.11 + t * 0.1, r * 0.9 - t * 0.55));
                    g = ra * (-qx * qx).exp() * (0.3 + 0.7 * dash) * if gap { 0.55 } else { 1.0 };
                }
                // the shore lights laid on the water
                if dw < 10.0 {
                    for &(lx, _, ph) in &self.shore_lights {
                        let d = x - lx;
                        if d.abs() > 3.0 {
                            continue;
                        }
                        g += (-d * d * 1.5).exp() * (-dw / 4.0).exp() * smooth(0.35, 0.7, noise(x * 0.6 + ph, y * 1.3 - t * 1.4)) * 0.8;
                    }
                }
                // where the warm light lies on the water it replaces the blue,
                // so it stays gold instead of greying
                if g > 0.0 {
                    let keep = 1.0 - (g * 4.0).min(0.9);
                    (cr, cg, cb) = (cr * keep + g, cg * keep + g * 0.56, cb * keep + g * 0.2);
                }
                // elsewhere the crests of the ripples catch the blue of the sky
                if g < 0.2 {
                    let sheen = smooth(0.6, 0.85, noise(x * 0.07 + t * 0.03, r * 1.1 - t * 0.25)) * 0.12 * (1.0 - g * 5.0) * deep;
                    cr += sheen * 0.3;
                    cg += sheen * 0.48;
                    cb += sheen;
                }
                let g = GROUND.map(f64::from);
                c[py * W + px] = [mix(g[0], cr, fade), mix(g[1], cg, fade), mix(g[2], cb, fade)];
            }
        }
    }
}

impl Piece for LanternLake {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let breathe = 0.06 * (t * 0.5).sin() + 0.04 * (t * 1.7 + 1.0).sin();

        // --- the sky and the land ---------------------------------------------
        for k in 0..SKY * W {
            let g = 1.0 + f64::from(self.take[k]) * breathe;
            let s = self.sky[k];
            self.canvas.c[k] = [s[0] * g, s[1] * g, s[2] * g];
        }
        self.canvas.painted.fill(0.0);

        // lanterns from the far shore, the furthest first
        for f in &self.shore {
            let u = t / f.life + f.off;
            let lap = u.floor();
            let a = (u - lap) * f.life;
            let climb = f.v * a;
            // the breeze leans the plume over to the right, more the higher it gets
            let cx = f.start_x(lap) + climb * f.wd + climb * climb * 0.004 + f.sw * ((a * f.sf + f.ph).sin() - f.ph.sin());
            let cy = WL + f.s * 0.5 + 0.6 - climb;
            if cy < -2.0 * f.s - 2.0 {
                continue;
            }
            let mut i = (0.7 + 0.45 * f.s.min(1.0)) * (0.88 + 0.08 * (t * f.f1 + f.ph).sin() + 0.04 * (t * f.f2).sin());
            let mut alpha = 1.0;
            if !f.out {
                // burning out: it gutters, reddens and goes
                let end = smooth(f.life - 4.0, f.life, a);
                i *= 1.0 - 0.55 * end * (0.6 + 0.4 * (a * 11.0 + f.ph).sin());
                alpha = 1.0 - end;
            }
            if alpha <= 0.01 {
                continue;
            }
            if f.s >= 1.4 {
                self.canvas.halo(cx, cy, f.s, 0.07 * i * alpha * smooth(0.0, 2.0, a), SKY);
            }
            self.canvas.lantern(cx, cy, f.s, i, alpha, SKY, Some(&self.tree));
        }
        for &(x, r, ph) in &self.shore_lights {
            let k = pix(r + 0.5) as usize * W + pix(x + 0.5) as usize;
            let q = 0.9 + 0.15 * (t * 3.1 + ph).sin() + 0.1 * (t * 7.3 + ph * 2.0).sin();
            self.canvas.c[k] = [q, q * 0.62, q * 0.24];
        }

        // --- the lake ------------------------------------------------------------
        self.lake(t);

        // --- the boat ------------------------------------------------------------
        let tc = t.rem_euclid(CYCLE);
        let bob = |at: f64| 0.3 * (at * 1.3).sin();
        let hold_y = BW - 14.1 * SC - HELD / 2.0;
        // each of the boat's lanterns flickers in its own time, the same in
        // the hands and after it is let go: m is the cycle it is let go in
        let flick = |m: f64| 1.1 + 0.05 * (t * 4.7 + m * 2.1).sin() + 0.03 * (t * 11.3 + m * 5.3).sin();
        let n = (t / CYCLE).floor();
        let mine = flick(if tc < LET_GO { n } else { n + 1.0 });
        // the lantern in the figure's hands, if there is one, and how lit it is
        let (mut hy, mut hs, mut ha) = (0.0, HELD, 0.0);
        if !(LET_GO..LIT + 5.0).contains(&tc) {
            (hy, ha) = (hold_y + bob(t), 1.0);
        } else if tc >= LIT {
            // lit at the chest, filling with light, then lifted over the head
            ha = smooth(LIT, LIT + 2.5, tc);
            hs = HELD * (0.75 + 0.25 * smooth(LIT, LIT + 3.0, tc));
            hy = mix(BW - 8.6 * SC, hold_y + bob(t), smooth(LIT + 3.5, LIT + 5.0, tc));
        }
        // the arms: held up a moment after the let-go, lowered, brought up to
        // the chest to light the next, and lifted with it over the head
        if tc < LET_GO + 2.0 {
            self.pose(&UP, &DOWN, smooth(LET_GO + 1.0, LET_GO + 2.0, tc));
        } else if tc < LIT + 1.0 {
            self.pose(&DOWN, &CHEST, smooth(LIT - 0.6, LIT, tc));
        } else {
            self.pose(&CHEST, &UP, smooth(LIT + 3.5, LIT + 5.0, tc));
        }
        // the lanterns already let go: this cycle's and the two before
        let mut flights = vec![];
        let mut m = n;
        while m >= n - 2.0 {
            let tr = m * CYCLE + LET_GO;
            let age = t - tr;
            m -= 1.0;
            if age < 0.0 {
                continue;
            }
            let y = hold_y + bob(tr) - 2.8 * (age - 1.5 * (1.0 - (-age / 1.5).exp()));
            if y < -3.0 * HELD {
                continue;
            }
            flights.push((FX + 0.3 * age + 1.2 * (age * 0.35).sin(), y, flick(m + 1.0)));
        }

        // their light: a glow in the air round each, and on the water a pool
        // round the boat and a broken road toward us, fading as a lantern
        // rises. [x, the bottom of the lantern, how much of it reaches the
        // water and the boat, how big its pool is]
        let bow_x = BX + 11.2 * SC;
        let bow_y = BW - (gunwale(11.2) + 1.2) * SC;
        let mut lights = vec![];
        let lamp = 1.2 + 0.04 * (t * 5.3).sin() + 0.03 * (t * 12.7).sin();
        lights.push((bow_x, bow_y + 1.0, 0.4 * lamp, 0.4));
        self.canvas.halo(bow_x, bow_y, 2.0, 0.07 * lamp, H);
        if ha > 0.0 {
            lights.push((FX, hy + hs / 2.0, ha * mine, 1.0));
        }
        for &(x, y, l) in &flights {
            self.canvas.halo(x, y, HELD, 0.2 * l, H);
            lights.push((x, y + HELD / 2.0, l * (-(hold_y - y).max(0.0) / 8.0).exp(), 1.0));
        }
        if ha > 0.0 {
            self.canvas.halo(FX, hy, HELD, 0.2 * ha * mine, H);
        }
        for &(lx, _, q, sz) in &lights {
            if q < 0.003 {
                continue;
            }
            let x0 = pix(lx - 34.0 * sz).max(0) as usize;
            let x1 = (pix(lx + 34.0 * sz) + 1).min(W as i64) as usize;
            for py in pix(BW - 9.0 * SC) as usize..H {
                let y = yc(py);
                let dw = y - BW;
                for px in x0..x1 {
                    let x = xc(px) - 0.5;
                    let dx = x + 0.5 - lx;
                    // the water behind the boat is further off, so its pool is squashed
                    let dh = if dw < 0.0 { 2.6 } else { 3.6 } * SC * sz;
                    let mut g = (-((dx / (16.0 * SC * 0.8 * sz)).powi(2) + (dw / dh).powi(2)).sqrt()).exp() * 0.5;
                    if dw > -1.0 {
                        let w = (1.4 + 0.2 * dw.max(0.0)) * (0.5 + 0.5 * sz);
                        g += (-(dx / w).powi(2)).exp() * (-dw.max(0.0) / (16.0 * sz)).exp() * smooth(0.4, 0.72, noise(x * 0.45 + 3.0, y * 1.2 - t * 1.3)) * 1.1;
                    }
                    g *= q;
                    let keep = 1.0 - (g * 4.0).min(0.9);
                    let c = &mut self.canvas.c[py * W + px];
                    *c = [c[0] * keep + g, c[1] * keep + g * 0.62, c[2] * keep + g * 0.26];
                }
            }
        }
        // the boat and the figure, dark against it, their edges lit from above
        let [x0, y0, bw, bh] = self.bx;
        for j in 0..bh {
            for i in 0..bw {
                let m = self.shape[j * bw + i];
                if m == 0 {
                    continue;
                }
                let (px, py) = (x0 + i, y0 + j);
                let k = py * W + px;
                let open = |i: i64, j: i64| i < 0 || j < 0 || i >= bw as i64 || j >= bh as i64 || self.shape[j as usize * bw + i as usize] == 0;
                let (ii, jj) = (i as i64, j as i64);
                // the sides of the figure, a thin rim against the road behind
                let mut g = if m == 2 && (open(ii - 1, jj) || open(ii + 1, jj)) { 0.16 } else { 0.0 };
                let top = open(ii, jj - 1);
                for &(lx, ly, lq, _) in &lights {
                    if lq < 0.003 {
                        continue;
                    }
                    let (dx, dy) = (lx - xc(px), ly - yc(py));
                    let d = dx.hypot(dy).max(1e-9);
                    if m == 2 {
                        // the neighbour toward the light: if it is open, this
                        // pixel is an edge facing it
                        let o = open(ii + (dx / d).round() as i64, jj + (dy / d).round() as i64);
                        g += lq * if o { 0.95 } else { 0.05 } * (-d / (6.0 * SC)).exp();
                    } else {
                        // the gunwale catches the light along its length
                        g += lq * if top { 0.6 * (-dx.abs() / (11.0 * SC)).exp() } else { 0.02 };
                    }
                }
                self.canvas.c[k] = [0.01 + g, 0.008 + g * 0.55, 0.014 + g * 0.22];
            }
        }
        self.canvas.lantern(bow_x, bow_y, 2.0, lamp, 1.0, H, None);
        for &(x, y, q) in &flights {
            self.canvas.lantern(x, y, HELD, q, 1.0, H, None);
        }
        if ha > 0.0 {
            self.canvas.lantern(FX, hy, hs, ha * mine, (ha * 1.4).min(1.0), H, None);
        }

        // --- lanterns rising past us -----------------------------------------------
        for c in &self.close {
            let u = t / c.life + c.off;
            let lap = u.floor();
            let a = (u - lap) * c.life;
            // before its first flight it is still waiting below the frame
            if lap < 0.0 || a * c.v > c.path {
                continue;
            }
            let mut x0 = c.x;
            if lap > 0.0 {
                x0 = 6.0 + 188.0 * hash(c.i * 17.0 + lap, 44.0);
                if x0 > 84.0 && x0 < 132.0 {
                    x0 += if x0 < 108.0 { -32.0 } else { 32.0 }; // clear of the boat
                }
            }
            let cx = x0 + c.wd * a + c.sw * ((a * c.sf + c.ph).sin() - c.ph.sin());
            let cy = SH + 2.5 * c.s - c.v * a;
            let i = 1.15 + 0.05 * (t * 3.3 + c.ph).sin() + 0.03 * (t * 8.1 + c.i).sin();
            self.canvas.halo(cx, cy, c.s, 0.1 * i, H);
            self.canvas.lantern(cx, cy, c.s, i, 1.0, H, None);
        }

        // --- the picture -------------------------------------------------------------
        for k in 0..W * H {
            let c = self.canvas.c[k];
            let mut out = [c[0] as f32, c[1] as f32, c[2] as f32];
            // The far range in slate, a greyed blue no other part of the
            // picture uses, unless a lantern or its glow is over it.
            if k < SKY * W && self.far[k] && self.canvas.painted[k] < 0.4 && c[2] >= c[0] {
                let p = c[0].max(c[1]).max(c[2]) as f32;
                out = [0.32 * p, 0.36 * p, 0.5 * p];
            }
            self.pixels.set(k % W, k / W, out);
        }
        // stars twinkle, wherever nothing brighter is in front of them
        for &(k, ph, rate, b) in &self.stars {
            let s = b * (0.7 + 0.3 * (t * rate + ph).sin());
            let c = self.canvas.c[k];
            if s < 0.3 || c[0].max(c[1]).max(c[2]) > 0.4 {
                continue;
            }
            let s = s.min(1.0) as f32;
            let (px, py) = (k % W, k / W);
            let o = self.pixels.get(px, py);
            self.pixels.set(px, py, [o[0].max(s * 0.86), o[1].max(s * 0.9), o[2].max(s)]);
        }
        self.pixels.draw(grid, env);
    }
}
