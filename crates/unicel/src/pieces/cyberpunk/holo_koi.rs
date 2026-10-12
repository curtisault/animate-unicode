//! holo koi: a rooftop at night among the towers, in the rain. A wooden
//! water tank stands on its stilts, an aerial mast blinks red at the top,
//! a dish leans toward the sky, an air conditioner hums and a vent pipe
//! breathes steam, and cables sag from the mast across the sky. Above it all
//! a giant hologram koi, a tancho with a magenta crown, swims slowly between
//! the skyscrapers, thrown up by a projector on the roof: its body is lines
//! of light that crawl upward, its long tail and fins ripple, a refresh
//! sweeps down it now and then, and the projection glitches, tearing into
//! shifted bands and split colour, with now and then a long tear of blocks.
//! A smaller pink koi swims the other way, higher and further off, behind
//! the big one; both slip behind the two skyscrapers that frame the roof.
//!
//! The koi's light falls on what is round it: the rain that crosses it, the
//! smog clouds, the near towers' edges, the tank, the mast and the dish, and
//! the wet roof, whose puddles give the sky back, rippled by the drops. The
//! projector's beam climbs through the rain to the fish. The towers fade into
//! a violet smog toward the horizon, their windows lit, a sign or two buzzing
//! in pixel-stroke glyphs, and a drone crosses far off. In one ink each pixel
//! is octant dots, as many as it is bright.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Holo koi's meta.
pub static META: Meta = Meta {
    name: "holo koi",
    slug: "holo-koi",
    category: Category::Cyberpunk,
    note: "a giant hologram koi swims through the rain above a night rooftop",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // The night's near-blacks, navies and violets, the smog's plums, then the
    // hologram's cyans and magentas, the signs' greens and ambers, the
    // beacon's reds, the roof's greys, the rain's blues and the whites.
    palette: Some(&[
        "#050409", "#08070f", "#0c0a17", "#110e20", "#16122a", "#1c1634", "#231a3f", "#2c1f4b", "#0a0d18", "#0f1424", "#151c31", "#1c2540", "#3a2552",
        "#4b2d62", "#5e3570", "#723d7c", "#8a4787", "#2a1830", "#3d1a3a", "#5a1e4f", "#a3196c", "#d8278c", "#ff47a8", "#ff85c6", "#ffc4e3", "#0b3a4a",
        "#0f5a6e", "#127f94", "#1aa8bc", "#2fd4e6", "#7cecf4", "#c8fbff", "#0d2230", "#12303f", "#1f6b2c", "#3fbf4a", "#9dff5c", "#5e340a", "#a0600f",
        "#e8921e", "#ffc65e", "#ffe6a8", "#4a0c12", "#8f1520", "#e02a2a", "#ff6a5a", "#1a1a22", "#26252f", "#34323e", "#4a4756", "#67637a", "#8d89a3",
        "#2a3a6a", "#4a5f9e", "#7f95d0", "#b9c8f0", "#3b2f7a", "#5a4bb0", "#9a7fd0", "#e8f0ff", "#ffffff", "#3a5f8a", "#14262a",
    ]),
    ground: Some("#050409"),
    ..Meta::DEFAULT
};

/// The picture in pixels.
const W: usize = 320;
const H: usize = 160;
/// The parapet's top row, and the first row of the roof's surface.
const LEDGE: usize = 118;
const ROOF: usize = 126;
/// The row the puddles mirror the sky about.
const MIRROR: f64 = 108.0;
/// The cloud deck: wraps, so it can drift forever.
const CW: usize = 640;
const CH: usize = 84;
/// The mast's beacon and the projector's lens.
const BEACON: (f64, f64) = (262.5, 7.5);
const LENS: (f64, f64) = (177.5, 126.5);
/// The spine's samples, from just ahead of the nose to the tail's tip.
const NU: usize = 160;
const U0: f64 = -0.05;
const U1: f64 = 1.35;

type C3 = [f64; 3];
type F3 = [f32; 3];

const WHITE: C3 = [0.85, 1.0, 1.0];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Sky,
    Far,
    Mid,
    Near,
    Roof,
}

/// How a lit pixel changes in time.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lamp {
    Steady,
    Flicker,
    Neon,
    Aviation,
    Led,
}

/// What a pixel of the roof's furniture is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Item {
    None,
    Solid,
    Cable,
    Fan,
    Lens,
    Beacon,
    Led,
}

fn mix3(a: C3, b: C3, k: f64) -> C3 {
    [mix(a[0], b[0], k), mix(a[1], b[1], k), mix(a[2], b[2], k)]
}

fn f3(c: C3) -> F3 {
    [c[0] as f32, c[1] as f32, c[2] as f32]
}

fn c3(c: F3) -> C3 {
    [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])]
}

/// The sky at (x, y): violet-black overhead to the smog's glow over the city.
fn sky(x: f64, y: f64) -> C3 {
    let v = clamp(y / LEDGE as f64);
    let hue = fbm(x * 0.012, 4.0, 3);
    let hor = [mix(0.26, 0.44, hue), mix(0.11, 0.13, hue), mix(0.38, 0.34, hue)];
    let (top, mid) = ([0.016, 0.012, 0.036], [0.05, 0.03, 0.095]);
    let (a, b) = (smooth(0.0, 0.65, v), v.powf(4.0));
    [0, 1, 2].map(|i| top[i] + (mid[i] - top[i]) * a + (hor[i] - mid[i]) * b)
}

/// A katakana-ish glyph, 5 × 7 pixels, rows as bits: three or four strokes
/// from a small set, picked by `seed`.
fn glyph(seed: f64) -> [u8; 7] {
    const STROKES: [[u8; 7]; 9] = [
        [0b11111, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0b11111, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0b11111],
        [0b00100; 7],
        [0b10000, 0b10000, 0b10000, 0b10000, 0, 0, 0],
        [0b00001; 7],
        [0, 0, 0, 0b00010, 0b00100, 0b01000, 0b10000],
        [0, 0b01000, 0b00100, 0, 0, 0, 0],
        [0b01000, 0b01000, 0b11110, 0b01010, 0b01010, 0b10010, 0b00110],
    ];
    let mut g = [0u8; 7];
    let mut n = 0;
    for (i, s) in STROKES.iter().enumerate() {
        if hash(seed * 17.0 + 3.0, i as f64 * 5.0 + 1.0) < 0.36 {
            n += 1;
            for r in 0..7 {
                g[r] |= s[r];
            }
        }
    }
    if n < 2 {
        for r in 0..7 {
            g[r] |= STROKES[0][r] | STROKES[6][r] | STROKES[3][r];
        }
    }
    g
}

/// A tower: its span, its top, and how it is finished.
struct Tower {
    x0: i32,
    x1: i32,
    top: i32,
    layer: Layer,
    seed: f64,
    step: i32,
    spire: i32,
}

/// The town and the roof, laid out pixel by pixel once.
struct Town {
    back: Vec<F3>,
    layer: Vec<Layer>,
    lamp: Vec<Lamp>,
    seed: Vec<f32>,
    glow: Vec<F3>,
    rim: Vec<f32>,
    item: Vec<Item>,
    icol: Vec<F3>,
    inorm: Vec<[f32; 2]>,
    lights: Vec<(f64, f64, C3)>,
}

impl Town {
    fn set(&mut self, x: i32, y: i32, c: C3, layer: Layer, lamp: Lamp, seed: f64) {
        if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < LEDGE {
            let k = y as usize * W + x as usize;
            self.back[k] = f3(c);
            self.layer[k] = layer;
            self.lamp[k] = lamp;
            self.seed[k] = seed as f32;
        }
    }

    /// A light that glows into the air round it.
    fn neon(&mut self, x: i32, y: i32, c: C3, lamp: Lamp, seed: f64, layer: Layer) {
        self.set(x, y, c, layer, lamp, seed);
        self.lights.push((x as f64 + 0.5, y as f64 + 0.5, c));
    }

    fn put(&mut self, x: i32, y: i32, kind: Item, c: C3, nx: f64, ny: f64) {
        if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
            let k = y as usize * W + x as usize;
            self.item[k] = kind;
            self.icol[k] = f3(c);
            self.inorm[k] = [nx as f32, ny as f32];
        }
    }

    /// A pixel-thin line of furniture.
    fn line(&mut self, a: (f64, f64), b: (f64, f64), kind: Item, c: C3, nx: f64, ny: f64) {
        let n = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * 1.5).ceil().max(1.0) as usize;
        for i in 0..=n {
            let s = i as f64 / n as f64;
            self.put(mix(a.0, b.0, s).floor() as i32, mix(a.1, b.1, s).floor() as i32, kind, c, nx, ny);
        }
    }

    /// A cable sagging from `a` to `b`.
    fn cable(&mut self, a: (f64, f64), b: (f64, f64), sag: f64) {
        let n = ((b.0 - a.0).abs() * 2.0).ceil() as usize + 2;
        let mut last = (i32::MIN, i32::MIN);
        for i in 0..=n {
            let s = i as f64 / n as f64;
            let (x, y) = (mix(a.0, b.0, s), mix(a.1, b.1, s) + sag * 4.0 * s * (1.0 - s));
            let p = (x.floor() as i32, y.floor() as i32);
            if p != last {
                self.put(p.0, p.1, Item::Cable, [0.025, 0.022, 0.035], 0.0, -1.0);
                last = p;
            }
        }
    }

    fn tower(&mut self, tw: &Tower) {
        let sd = tw.seed;
        let far = tw.layer == Layer::Far;
        for y in (tw.top - tw.spire).max(0)..LEDGE as i32 {
            for x in tw.x0.max(0)..tw.x1.min(W as i32) {
                let (fx, fy) = (x as f64, y as f64);
                let mid = (tw.x0 + tw.x1) / 2;
                if y < tw.top {
                    if x == mid {
                        self.set(x, y, mix3([0.04, 0.03, 0.07], sky(fx, fy), 0.4), tw.layer, Lamp::Steady, 0.0);
                    }
                    continue;
                }
                if tw.step > 0 && y < tw.top + 7 && (x < tw.x0 + tw.step || x >= tw.x1 - tw.step) {
                    continue;
                }
                let s = sky(fx, fy);
                let (rx, ry) = (x - tw.x0, y - tw.top);
                let mut c;
                let mut lamp = Lamp::Steady;
                if far {
                    c = mix3([0.05, 0.035, 0.085], s, 0.4 + 0.45 * smooth(66.0, 118.0, fy));
                    if rx % 3 == 1 && ry % 3 == 2 && x < tw.x1 - 1 && hash(fx + sd * 50.0, fy * 7.0) > 0.84 {
                        let w = if hash(fx, fy + sd) > 0.4 { [0.6, 0.42, 0.25] } else { [0.35, 0.45, 0.6] };
                        c = mix3(w, s, 0.4 + 0.3 * smooth(80.0, 118.0, fy));
                    }
                } else {
                    c = mix3([0.022, 0.018, 0.04], s, 0.05 + 0.3 * smooth(90.0, 118.0, fy));
                    if x == tw.x0 || x == tw.x1 - 1 {
                        c = mix3(c, s, 0.35);
                    }
                    let (wx, wy) = (rx / 3, ry / 4);
                    if rx % 3 == 1 && ry % 4 >= 2 && x < tw.x1 - 1 && ry > 2 {
                        let h = hash(wx as f64 + sd * 100.0, wy as f64 + sd * 7.0);
                        if h > 0.74 {
                            let pick = hash(wy as f64 * 3.0 + sd, wx as f64 + 9.0);
                            let w = if pick < 0.5 {
                                [0.95, 0.62, 0.3]
                            } else if pick < 0.8 {
                                [0.62, 0.78, 1.0]
                            } else if pick < 0.92 {
                                [0.3, 0.85, 0.95]
                            } else {
                                [0.95, 0.35, 0.7]
                            };
                            c = mix3(mix3(w, [0.0; 3], 0.4), s, 0.12 + 0.3 * smooth(80.0, 118.0, fy));
                            if h > 0.965 {
                                lamp = Lamp::Flicker;
                            }
                        }
                    }
                }
                self.set(x, y, c, tw.layer, lamp, hash(fx, fy * 3.0 + sd));
            }
        }
        if !far && tw.top < 50 {
            let mid = (tw.x0 + tw.x1) / 2;
            let y = tw.top - tw.spire - 1;
            self.neon(mid, y, [1.0, 0.15, 0.1], Lamp::Aviation, sd * 0.37, tw.layer);
        }
    }

    /// A sign of glyphs on a dark plate, framed in a tube of the same colour.
    fn sign(&mut self, x0: i32, y0: i32, n: usize, vertical: bool, c: C3, seed: f64) {
        let (w, h) = if vertical { (9, 2 + n as i32 * 9) } else { (3 + n as i32 * 7, 11) };
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                let border = x == x0 || y == y0 || x == x0 + w - 1 || y == y0 + h - 1;
                if border {
                    self.neon(x, y, mix3(c, [0.0; 3], 0.45), Lamp::Neon, seed, Layer::Mid);
                } else {
                    self.set(x, y, [0.05, 0.02, 0.05], Layer::Mid, Lamp::Steady, 0.0);
                }
            }
        }
        for i in 0..n {
            let g = glyph(seed * 10.0 + i as f64);
            let (gx, gy) = if vertical { (x0 + 2, y0 + 2 + i as i32 * 9) } else { (x0 + 2 + i as i32 * 7, y0 + 2) };
            for (r, bits) in g.iter().enumerate() {
                for b in 0..5 {
                    if bits >> (4 - b) & 1 == 1 {
                        self.neon(gx + b, gy + r as i32, c, Lamp::Neon, seed + i as f64 * 0.1, Layer::Mid);
                    }
                }
            }
        }
    }
}

/// One of the fish: its size, its path and its light.
struct Koi {
    /// Length in pixels, nose to the tail's tip being 1.3 of it.
    l: f64,
    /// Which way it swims: 1 right, −1 left.
    dir: f64,
    /// Seconds to cross the sky and come round again.
    period: f64,
    t0: f64,
    x_start: f64,
    travel: f64,
    y0: f64,
    wave: [f64; 4],
    /// Seconds a stroke of the tail takes.
    swim: f64,
    seed: f64,
    base: C3,
    rim: C3,
    patch: C3,
    fin: C3,
    /// How much of it the smog lets through.
    haze: f64,
    tancho: bool,
    /// The deepest layer it shows in front of.
    max: Layer,
}

/// A fin, in the fish's own frame: where it is fixed, which way it points
/// (along, across), how long and wide it is.
#[derive(Clone, Copy, Default)]
struct Fin {
    u: f64,
    v: f64,
    du: f64,
    dv: f64,
    len: f64,
    wid: f64,
    ph: f64,
}

/// A fish at one moment.
struct Pose {
    hx: f64,
    spine: [f64; NU],
    fins: [Fin; 4],
    t: f64,
    glitch: f64,
    slot: f64,
    tear: bool,
    inten: f64,
    sweep: f64,
    y_lo: f64,
    y_hi: f64,
}

impl Pose {
    fn spine_at(&self, u: f64) -> f64 {
        let f = (u - U0) / (U1 - U0) * (NU - 1) as f64;
        let i = (f.floor().max(0.0) as usize).min(NU - 2);
        mix(self.spine[i], self.spine[i + 1], f - i as f64)
    }
}

/// Half the body's width at `u` along it, in lengths.
fn body_hw(u: f64) -> f64 {
    let n = clamp(u / 0.2);
    let nose = (1.0 - (1.0 - n) * (1.0 - n)).max(0.0).sqrt();
    0.13 * nose * (1.0 - 0.83 * clamp((u - 0.3) / 0.56).powf(1.5))
}

/// `c` at opacity `ca` over premultiplied `col` at `a`.
fn over(col: &mut C3, a: &mut f64, c: C3, ca: f64) {
    for i in 0..3 {
        col[i] = c[i] * ca + col[i] * (1.0 - ca);
    }
    *a = ca + *a * (1.0 - ca);
}

impl Koi {
    /// Where the fish is at `t`, how its body bends, and how the projection is behaving.
    fn pose(&self, t: f64) -> Pose {
        let s = ((t + self.t0) / self.period).rem_euclid(1.0);
        let hx = self.x_start + self.dir * self.travel * s;
        let mut spine = [0.0; NU];
        let (mut y_lo, mut y_hi) = (f64::MAX, f64::MIN);
        for (i, sp) in spine.iter_mut().enumerate() {
            let u = U0 + (U1 - U0) * i as f64 / (NU - 1) as f64;
            // the body follows where the head has been
            let su = s - u * self.l / self.travel;
            let path = self.y0 + self.wave[0] * (TAU * 2.0 * su + self.wave[1]).sin() + self.wave[2] * (TAU * 3.0 * su + self.wave[3]).sin();
            // a stroke running down the body, and a slow turn one way and the other
            let u2 = u.max(0.0).powi(2);
            let bend = (0.012 + 0.06 * u2) * (TAU * 0.8 * u - t * TAU / self.swim).sin() + 0.07 * u2 * (t * TAU / 11.0 + self.seed * 2.0).sin();
            *sp = path + self.l * bend;
            y_lo = y_lo.min(*sp);
            y_hi = y_hi.max(*sp);
        }
        let mut fins = [Fin::default(); 4];
        for (j, side) in [1.0, -1.0].into_iter().enumerate() {
            let th = 0.7 + 0.28 * (t * TAU / self.swim * 0.8 + side * 0.9 + self.seed).sin();
            fins[j] = Fin { u: 0.2, v: side * body_hw(0.2) * 0.8, du: th.cos(), dv: side * th.sin(), len: 0.22, wid: 0.065, ph: side * 1.7 };
            let th = 0.5 + 0.2 * (t * TAU / self.swim * 0.8 + side * 0.9 + 2.0).sin();
            fins[j + 2] = Fin { u: 0.5, v: side * body_hw(0.5) * 0.75, du: th.cos(), dv: side * th.sin(), len: 0.14, wid: 0.045, ph: side + 0.5 };
        }
        let slot = (t * 7.5).floor();
        let mut glitch = 0.0;
        if slot > 0.0 && hash(slot, self.seed * 31.0) > 0.91 {
            glitch = 0.25 + 0.5 * hash(slot, self.seed * 31.0 + 1.0);
        }
        let tp = (t + self.seed * 3.1).rem_euclid(14.0);
        let tear = (9.0..9.9).contains(&tp);
        if tear {
            glitch = f64::max(glitch, 0.45 + 0.35 * hash(slot, self.seed * 31.0 + 2.0));
        }
        let inten = (0.92 + 0.08 * (t * 9.0 + self.seed).sin()) * (1.0 - 0.5 * glitch * hash(slot, self.seed * 31.0 + 5.0));
        let sweep = (t / 3.4 + self.seed * 0.37).rem_euclid(1.0) * 1.8 - 0.3;
        Pose { hx, spine, fins, t, glitch, slot, tear, inten, sweep, y_lo: y_lo - 0.42 * self.l, y_hi: y_hi + 0.42 * self.l }
    }

    /// How much of the patch colour is at (u, v).
    fn patch_at(&self, u: f64, v: f64) -> f64 {
        if self.tancho {
            smooth(0.056, 0.036, (u - 0.105).hypot(v * 1.1))
        } else {
            smooth(0.52, 0.6, fbm(u * 8.0 + self.seed * 10.0, v * 16.0, 2)) * smooth(0.06, 0.14, u) * smooth(0.82, 0.7, u)
        }
    }

    /// The fish at (x, y), premultiplied, and its opacity.
    fn eval(&self, p: &Pose, x: f64, y: f64) -> (C3, f64) {
        let l = self.l;
        let u = (x - p.hx) * -self.dir / l;
        if !(U0..U1).contains(&u) {
            return ([0.0; 3], 0.0);
        }
        let v = (y - p.spine_at(u)) / l;
        if v.abs() > 0.42 {
            return ([0.0; 3], 0.0);
        }
        let t = p.t;
        let mut col = [0.0; 3];
        let mut a = 0.0;
        // the tail: long and flowing, forked, fluttering, its rays spreading from the root
        if u > 0.74 {
            let ta = ((u - 0.82) / 0.42).max(0.0);
            let tv = v - 0.015 * ta * (ta * 8.0 - t * 5.0 + self.seed).sin();
            let tw = (0.022 + 0.19 * ta.powf(0.85)) * (1.0 + 0.08 * (ta * 9.0 - t * 4.0 + self.seed).sin());
            let r = tv.abs() / tw;
            let amax = 0.6 + 0.4 * r.powf(1.4) - 0.2 * smooth(0.7, 1.0, r) + 0.04 * (r * 6.0 + t * 3.0).sin();
            let v = tv;
            if r < 1.0 && ta < amax {
                let ray = smooth(0.3, 0.0, (v / (ta + 0.08) * 70.0).sin().abs());
                let edge = smooth(0.72, 1.0, r).max(smooth(amax - 0.12, amax, ta));
                let fa = (0.2 + 0.35 * ray + 0.6 * edge).min(1.0) * smooth(1.0, 0.9, r) * smooth(amax, amax - 0.05, ta) * (1.0 - 0.3 * ta);
                let irid = 0.5 + 0.5 * (ta * 6.0 - t * 1.7 + v * 20.0).sin();
                over(&mut col, &mut a, mix3(self.fin, self.patch, 0.45 * irid * edge), fa);
            }
        }
        // the fins, flapping
        for f in &p.fins {
            let (ru, rv) = (u - f.u, v - f.v);
            let fa = (ru * f.du + rv * f.dv) / f.len;
            if !(0.0..1.0).contains(&fa) {
                continue;
            }
            let fb = (rv * f.du - ru * f.dv) / f.wid;
            let half = (fa.powf(0.5) * (1.0 - fa.powi(3)).sqrt()) * (1.0 + 0.22 * (fa * 10.0 - t * 5.0 + f.ph).sin());
            let r = fb.abs() / half.max(0.02);
            if r >= 1.0 {
                continue;
            }
            let ray = smooth(0.3, 0.0, (fb / (fa + 0.2) * 14.0).sin().abs());
            let edge = smooth(0.65, 1.0, r).max(smooth(0.7, 0.97, fa));
            let fo = (0.22 + 0.35 * ray + 0.6 * edge).min(1.0) * smooth(1.0, 0.85, r) * smooth(1.0, 0.92, fa);
            let irid = 0.5 + 0.5 * (fa * 5.0 + t * 1.5 + f.ph).sin();
            over(&mut col, &mut a, mix3(self.fin, self.patch, 0.5 * irid * edge), fo);
        }
        // the barbels, either side of the mouth
        if u < 0.05 {
            let (bu, bv) = (0.02 - u, v.abs() - 0.03);
            let s = clamp((bu * 0.6 + bv * 0.8) / 0.05);
            let d = (bu - s * 0.03).hypot(bv - s * 0.04 - 0.004 * (t * 3.0).sin() * s) * l;
            if d < 0.7 && u > -0.04 {
                over(&mut col, &mut a, self.rim, 0.8 * (1.0 - s * 0.5));
            }
        }
        // the body: a bright rim, a ridge down the back, scales and their glints
        if (0.0..0.9).contains(&u) {
            let hw = body_hw(u);
            let r = v.abs() / hw.max(1e-4);
            if r < 1.0 {
                let rim = smooth(0.5, 0.97, r);
                let ridge = (-(r / 0.2).powi(2)).exp() * smooth(0.14, 0.3, u) * smooth(0.86, 0.55, u);
                let (sa, sb) = (u * l / 3.0, v * l / 3.0);
                let (d1, d2) = (sa + sb, sa - sb);
                let line = (d1 - d1.round()).abs().min((d2 - d2.round()).abs());
                let body = smooth(0.15, 0.25, u);
                let sc = smooth(0.18, 0.0, line) * body * (1.0 - rim);
                let glint = if hash(d1.floor() + self.seed * 100.0, d2.floor() * 13.0 + (t * 3.0).floor() * 7.0) > 0.975 { 0.9 * body } else { 0.0 };
                let spot = self.patch_at(u, v);
                let sweep = (-((u - p.sweep) / 0.04).powi(2)).exp();
                let mut c = mix3(self.base, self.rim, rim);
                c = mix3(c, self.patch, spot * (1.0 - 0.5 * rim));
                c = mix3(c, WHITE, ((glint + 0.5 * sweep) * 0.7).min(1.0));
                // the gill covers' edge, curving back from the throat
                let gill = smooth(0.9, 0.3, (u - 0.19 - 2.2 * v * v).abs() * l) * smooth(0.95, 0.6, r);
                let mut ba = (0.2 + 0.65 * rim + 0.3 * ridge + 0.22 * sc + glint + 0.5 * sweep + 0.35 * spot + 0.5 * gill).min(1.0) * smooth(1.0, 0.88, r);
                if (u - 0.07).hypot(v.abs() - 0.058) * l < 0.011 * l + 0.5 {
                    (c, ba) = (WHITE, 1.0);
                }
                over(&mut col, &mut a, c, ba);
            }
        }
        (col, a)
    }

    /// The fish into `buf`, wherever the town behind is no nearer than its layer.
    fn draw(&self, p: &Pose, buf: &mut [C3], layer: &[Layer]) {
        let l = self.l;
        let (xa, xb) = if self.dir < 0.0 { (p.hx - 0.06 * l, p.hx + 1.34 * l) } else { (p.hx - 1.34 * l, p.hx + 0.06 * l) };
        let g = p.glitch;
        let m = if g > 0.0 { 32.0 } else { 3.0 };
        let x0 = (xa - m).floor().clamp(0.0, W as f64) as usize;
        let x1 = (xb + m).ceil().clamp(0.0, W as f64) as usize;
        let y0 = p.y_lo.floor().clamp(0.0, ROOF as f64) as usize;
        let y1 = p.y_hi.ceil().clamp(0.0, ROOF as f64) as usize;
        let sd = self.seed * 31.0;
        let chroma = 0.8 + 4.0 * g;
        let scroll = (p.t * 6.0).floor() as usize;
        let bh = 2.0 + (hash(p.slot, sd + 3.0) * 6.0).floor();
        let boff = hash(p.slot, sd + 4.0) * 37.0;
        for py in y0..y1 {
            let y = py as f64 + 0.5;
            let (mut shift, mut drop) = (0.0, 1.0);
            if g > 0.0 {
                let band = ((y + boff) / bh).floor();
                if hash(band, p.slot * 5.0 + sd) < 0.12 + 0.4 * g {
                    shift = (hash(band * 3.0 + 1.0, p.slot * 5.0 + sd) - 0.5) * (6.0 + 36.0 * g);
                }
                if hash(py as f64, p.slot * 11.0 + sd) < 0.12 * g {
                    drop = 0.12;
                }
            }
            let scan = [1.35, 0.8, 0.32][(py + scroll) % 3];
            // a tear: a line of light across part of the fish
            let tear_line = p.tear && hash(py as f64, p.slot * 7.0 + sd) > 0.95;
            let (ta, tb) = (mix(xa, xb, hash(py as f64, p.slot + 2.0)), mix(xa, xb, hash(py as f64, p.slot + 3.0)));
            for px in x0..x1 {
                let k = py * W + px;
                if layer[k] > self.max {
                    continue;
                }
                let x = px as f64 + 0.5 + shift;
                let (cr, ar) = self.eval(p, x - chroma, y);
                let (cg, ag) = self.eval(p, x, y);
                let (cb, ab) = self.eval(p, x + chroma, y);
                let mut a = (ar + ag + ab) / 3.0;
                let mut c = [cr[0], cg[1], cb[2]];
                if tear_line && (ta.min(tb)..ta.max(tb)).contains(&x) {
                    c = [c[0] + 0.3, c[1] + 0.9, c[2] + 1.0];
                    a = a.max(0.6);
                }
                if a <= 0.002 {
                    continue;
                }
                if p.tear {
                    let blk = hash((px / 6) as f64 + (py / 3) as f64 * 97.0, p.slot * 3.0 + sd);
                    if blk > 0.955 {
                        c = if blk > 0.98 { [0.6, 1.0, 0.35] } else { [1.0, 0.28, 0.66] };
                        a = 0.85;
                    }
                }
                let k2 = scan * drop * p.inten * self.haze;
                let b = &mut buf[k];
                for i in 0..3 {
                    b[i] = b[i] * (1.0 - 0.35 * a.min(1.0)) + c[i] * k2;
                }
            }
        }
    }

    /// Where along the fish its light comes from, and how bright.
    fn lights(&self, p: &Pose, at: &[f64], gain: f64, r: f64, out: &mut Vec<Src>) {
        for &u in at {
            let x = p.hx - self.dir * u * self.l;
            let c = self.base.map(|v| v * gain * p.inten * (1.0 - 0.6 * p.glitch));
            out.push(Src { x, y: p.spine_at(u), c, r2: r * r });
        }
    }
}

/// A light: where, what colour, and how far it reaches.
struct Src {
    x: f64,
    y: f64,
    c: C3,
    r2: f64,
}

/// The light all of `srcs` cast on (x, y).
fn light(srcs: &[Src], x: f64, y: f64) -> C3 {
    let mut l = [0.0; 3];
    for s in srcs {
        let d2 = (x - s.x).powi(2) + (y - s.y).powi(2);
        let w = s.r2 / (s.r2 + d2);
        for (li, ci) in l.iter_mut().zip(s.c) {
            *li += ci * w;
        }
    }
    l
}

/// The rain at (x, y): two veils of streaks, near and long, far and fine.
fn rain(x: f64, y: f64, t: f64) -> f64 {
    let mut r = 0.0;
    for (slant, speed, len, dens, gain, seed) in [(0.24, 250.0, 9.0, 0.3, 0.6, 3.0), (0.2, 150.0, 4.0, 0.36, 0.3, 7.0)] {
        let lane = (x - y * slant).floor();
        if hash(lane, seed) > dens {
            continue;
        }
        let period = 110.0 + 90.0 * hash(lane, seed + 1.0);
        let ph = (y + t * speed + hash(lane, seed + 2.0) * period).rem_euclid(period);
        if ph < len {
            r += gain * (ph / len);
        }
    }
    r
}

/// The rooftop and the koi.
pub struct HoloKoi {
    town: Town,
    puddle: Vec<f32>,
    /// The sky's colour just over the parapet, for the wet roof to give back.
    hor: Vec<F3>,
    cover: Vec<f32>,
    lit: Vec<f32>,
    koi: [Koi; 2],
    buf: Vec<C3>,
    pixels: Pixels,
}

/// A holo koi; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let mut town = Town {
        back: vec![[0.0; 3]; n],
        layer: vec![Layer::Sky; n],
        lamp: vec![Lamp::Steady; n],
        seed: vec![0.0; n],
        glow: vec![[0.0; 3]; n],
        rim: vec![0.0; n],
        item: vec![Item::None; n],
        icol: vec![[0.0; 3]; n],
        inorm: vec![[0.0; 2]; n],
        lights: vec![],
    };
    for py in 0..LEDGE {
        for px in 0..W {
            town.back[py * W + px] = f3(sky(px as f64, py as f64));
        }
    }

    // far towers, close-packed in the smog, then the middle ones with gaps between
    let mut towers = vec![];
    let (mut x, mut i) = (-6.0, 0.0);
    while x < W as f64 + 6.0 {
        let w = 8.0 + 16.0 * hash(i, 11.0);
        let step = if hash(i, 13.0) > 0.55 { 2 + (hash(i, 14.0) * 3.0) as i32 } else { 0 };
        let spire = if hash(i, 15.0) > 0.7 { 3 + (hash(i, 16.0) * 10.0) as i32 } else { 0 };
        towers.push(Tower { x0: x as i32, x1: (x + w) as i32, top: (62.0 + 36.0 * hash(i, 12.0)) as i32, layer: Layer::Far, seed: i, step, spire });
        x += w + (hash(i, 17.0) * 5.0 - 1.5).max(0.0);
        i += 1.0;
    }
    let (mut x, mut i) = (-4.0, 0.0);
    while x < W as f64 {
        let w = 14.0 + 20.0 * hash(i, 21.0);
        let step = if hash(i, 23.0) > 0.5 { 2 + (hash(i, 24.0) * 3.0) as i32 } else { 0 };
        let spire = if hash(i, 25.0) > 0.6 { 4 + (hash(i, 26.0) * 12.0) as i32 } else { 0 };
        towers.push(Tower { x0: x as i32, x1: (x + w) as i32, top: (32.0 + 50.0 * hash(i, 22.0)) as i32, layer: Layer::Mid, seed: i + 50.0, step, spire });
        x += w + 4.0 + 18.0 * hash(i, 27.0);
        i += 1.0;
    }
    for tw in &towers {
        town.tower(tw);
    }

    // the signs: a tall magenta one, an amber row, a small green one, a cyan LED band
    town.sign(110, 46, 4, true, [1.0, 0.25, 0.68], 1.0);
    town.sign(150, 90, 3, false, [1.0, 0.66, 0.2], 2.0);
    town.sign(238, 78, 2, true, [0.55, 1.0, 0.35], 3.0);
    if let Some(tw) = towers.iter().rfind(|tw| tw.layer == Layer::Mid && tw.x0 <= 205 && tw.x1 > 205) {
        for y in tw.top + 3..tw.top + 5 {
            for x in tw.x0 + 1..tw.x1 - 1 {
                town.neon(x, y, [0.2, 0.85, 1.0], Lamp::Led, x as f64, Layer::Mid);
            }
        }
    }

    // the two near skyscrapers, either side
    for py in 0..LEDGE {
        for px in (0..24).chain(296..W) {
            let (x, y) = (px as f64, py as f64);
            let right = px >= 296;
            if right && py < 10 && !(306..309).contains(&px) {
                continue;
            }
            let k = py * W + px;
            let s = sky(x, y);
            let pane = (px % 4 != 0) && (py % 7 != 0);
            let mut c = if pane { mix3([0.02, 0.018, 0.035], s, 0.22 + 0.1 * hash(x / 4.0, y / 7.0)) } else { [0.05, 0.045, 0.075] };
            let lamp = Lamp::Steady;
            if pane && hash((px / 4) as f64 + if right { 99.0 } else { 0.0 }, (py / 7) as f64 * 3.0) > 0.9 {
                c = if hash((px / 4) as f64, (py / 7) as f64 + 5.0) > 0.5 { [0.42, 0.28, 0.16] } else { [0.25, 0.3, 0.42] };
            }
            town.back[k] = f3(c);
            town.layer[k] = Layer::Near;
            town.lamp[k] = lamp;
            town.rim[k] = if right { smooth(300.5, 296.0, x) as f32 } else { smooth(19.0, 23.5, x) as f32 };
        }
    }
    town.neon(307, 0, [1.0, 0.15, 0.1], Lamp::Aviation, 0.6, Layer::Near);

    // the parapet: a wet ledge, its face streaked
    for py in LEDGE..ROOF {
        for px in 0..W {
            let (x, y) = (px as f64, py as f64);
            let c = if py < LEDGE + 3 {
                let s = 0.1 + 0.05 * fbm(x * 0.2, y, 2) + if py == LEDGE { 0.04 } else { 0.0 };
                [s, s * 0.92, s * 1.25]
            } else {
                let s = 0.035 + 0.03 * fbm(x * 0.4, y * 0.05, 3);
                [s, s * 0.9, s * 1.3]
            };
            town.back[py * W + px] = f3(c);
            town.layer[py * W + px] = Layer::Roof;
        }
    }

    // the roof's surface: tar, seams running away from us, and puddles
    let mut puddle = vec![0.0f32; n];
    for py in ROOF..H {
        for px in 0..W {
            let (x, y) = (px as f64, py as f64);
            let k = py * W + px;
            let mut s = 0.04 + 0.03 * fbm(x * 0.05, y * 0.2, 3);
            let fan = (x - 160.0) / (y - 96.0) / 1.6;
            if (fan - fan.round()).abs() < 0.02 * (y - 96.0).recip() * 30.0 || py == 136 || py == 149 {
                s *= 0.6;
            }
            town.back[k] = f3([s, s * 0.9, s * 1.35]);
            town.layer[k] = Layer::Roof;
            puddle[k] = smooth(0.5, 0.58, fbm(x * 0.022, y * 0.09 + 5.0, 4) + 0.05 * smooth(ROOF as f64, H as f64, y)) as f32;
        }
    }

    // the water tank, on its stilts, with a ladder
    let wood = [0.075, 0.048, 0.055];
    let steel = [0.05, 0.05, 0.068];
    for y in 54..100 {
        for x in 26..76 {
            let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
            let dx = fx - 50.0;
            if y >= 66 {
                let r = 21.0 + 0.8 * (std::f64::consts::PI * (fy - 66.0) / 34.0).sin();
                if dx.abs() <= r {
                    let nx = dx / r;
                    let hoop = matches!(y, 70 | 71 | 81 | 82 | 92 | 93);
                    let stave = (dx.abs() / r).asin() * 12.0;
                    let mut c = if hoop { steel } else { wood };
                    if !hoop && (stave - stave.round()).abs() < 0.12 {
                        c = c.map(|v| v * 0.6);
                    }
                    town.put(x, y, Item::Solid, c, nx, -0.15);
                }
            } else {
                let hw = 23.0 * (fy - 52.0) / 14.0;
                if dx.abs() <= hw && fy >= 52.0 {
                    town.put(x, y, Item::Solid, [0.06, 0.05, 0.065], dx / 23.0 * 0.7, -0.7);
                }
            }
        }
    }
    town.line((50.0, 47.0), (50.0, 53.0), Item::Solid, steel, 0.0, -1.0);
    for (top, bot, thick) in [(34.0, 31.0, 1), (66.0, 69.0, 1), (44.0, 42.0, 2), (56.0, 58.0, 2)] {
        let end = if thick == 2 { 148.0 } else { 143.0 };
        for d in 0..thick {
            let o = d as f64;
            town.line((top + o, 100.0), (bot + o, end), Item::Solid, steel, if top < 50.0 { -0.6 } else { 0.6 }, -0.2);
        }
    }
    town.line((32.0, 101.0), (68.0, 101.0), Item::Solid, steel, 0.0, -1.0);
    town.line((34.0, 104.0), (67.0, 140.0), Item::Solid, steel.map(|v| v * 0.8), 0.0, -0.3);
    town.line((66.0, 104.0), (33.0, 140.0), Item::Solid, steel.map(|v| v * 0.8), 0.0, -0.3);
    town.line((44.0, 122.0), (57.0, 122.0), Item::Solid, steel, 0.0, -1.0);
    for rail in [73.0, 77.0] {
        town.line((rail, 70.0), (rail, 149.0), Item::Solid, steel, 0.6, -0.1);
    }
    let mut ry = 72.0;
    while ry < 148.0 {
        town.line((73.0, ry), (77.0, ry), Item::Solid, steel, 0.3, -1.0);
        ry += 4.0;
    }

    // the air conditioners, their fans on top, and the vent pipe
    for (x0, x1, ytop, yface) in [(88, 121, 124, 131), (125, 141, 129, 133)] {
        for y in ytop..147 {
            for x in x0..x1 {
                let (fx, fy) = (x as f64, y as f64);
                if y < yface {
                    town.put(x, y, Item::Solid, [0.09, 0.088, 0.11], 0.0, -1.0);
                } else {
                    let louvre = (y - yface) % 2 == 1 && x > x0 + 1 && x < x1 - 2;
                    let s = if louvre { 0.035 } else { 0.065 + 0.01 * hash(fx, fy) };
                    town.put(x, y, Item::Solid, [s, s, s * 1.25], if x == x1 - 1 { 0.8 } else { 0.0 }, 0.3);
                }
            }
        }
    }
    for y in 124..131 {
        for x in 94..115 {
            let (dx, dy) = ((x as f64 + 0.5 - 104.5) / 9.0, (y as f64 + 0.5 - 127.5) / 3.0);
            if dx.hypot(dy) < 1.0 {
                town.put(x, y, Item::Fan, [0.03, 0.03, 0.04], 0.0, -1.0);
            }
        }
    }
    for y in 106..147 {
        for x in 146..150 {
            town.put(x, y, Item::Solid, [0.07, 0.065, 0.085], (x as f64 - 147.5) / 2.0, -0.1);
        }
    }
    for y in 103..106 {
        for x in 143..153 {
            town.put(x, y, Item::Solid, [0.08, 0.075, 0.095], (x as f64 - 147.5) / 5.0, -0.8);
        }
    }

    // the projector: a housing, a turret, the lens, and two status lights
    for y in 132..147 {
        for x in 164..191 {
            let top = y < 135;
            let c = if top { [0.1, 0.1, 0.13] } else { [0.055, 0.055, 0.075] };
            town.put(x, y, Item::Solid, c, if x == 190 { 0.8 } else { 0.0 }, if top { -1.0 } else { 0.25 });
        }
    }
    for y in 125..132 {
        let hw = 5.0 + (y - 125) as f64 * 0.6;
        for x in 165..191 {
            if (x as f64 + 0.5 - 177.5).abs() <= hw {
                town.put(x, y, Item::Solid, [0.07, 0.075, 0.095], (x as f64 + 0.5 - 177.5) / hw, -0.5);
            }
        }
    }
    for y in 124..130 {
        for x in 174..182 {
            if (x as f64 + 0.5 - LENS.0).hypot((y as f64 + 0.5 - LENS.1) * 1.3) < 3.2 {
                town.put(x, y, Item::Lens, [0.4, 0.95, 1.0], 0.0, 0.0);
            }
        }
    }
    town.put(168, 138, Item::Led, [0.4, 1.0, 0.35], 0.0, 0.0);
    town.put(171, 138, Item::Led, [1.0, 0.65, 0.2], 1.0, 0.0);

    // the dish, leaning up toward the sky, on its post
    let (dc, ds) = ((-0.5f64).cos(), (-0.5f64).sin());
    for y in 104..130 {
        for x in 205..240 {
            let (rx, ry) = (x as f64 + 0.5 - 222.0, y as f64 + 0.5 - 116.0);
            let (a, b) = ((rx * dc + ry * ds) / 13.0, (-rx * ds + ry * dc) / 6.5);
            let r = a.hypot(b);
            if r < 1.0 {
                let rimmed = r > 0.82;
                let s = if rimmed { 0.13 } else { 0.07 + 0.05 * (0.5 - 0.5 * a) };
                town.put(x, y, Item::Solid, [s, s, s * 1.2], -0.5 + 0.3 * a, -0.85);
            }
        }
    }
    town.line((222.0, 116.0), (213.0, 103.0), Item::Solid, steel, -0.5, -0.8);
    town.line((211.0, 101.0), (214.0, 103.0), Item::Solid, [0.12, 0.12, 0.15], -0.5, -0.8);
    town.line((224.0, 121.0), (224.0, 137.0), Item::Solid, steel, 0.5, 0.0);
    town.line((225.0, 121.0), (225.0, 137.0), Item::Solid, steel, 0.5, 0.0);
    for y in 137..145 {
        for x in 216..233 {
            town.put(x, y, Item::Solid, if y < 139 { [0.09, 0.09, 0.11] } else { [0.05, 0.05, 0.07] }, 0.0, if y < 139 { -1.0 } else { 0.2 });
        }
    }

    // the mast: a lattice tapering to a pole, panels, and the beacon on top
    let mx = 262.5;
    let hwf = |y: f64| 1.0 + 6.0 * (y - 20.0) / 128.0;
    for y in 20..149 {
        let fy = y as f64 + 0.5;
        let (xl, xr) = (mx - hwf(fy), mx + hwf(fy));
        town.put(xl.floor() as i32, y, Item::Solid, steel, -0.6, -0.2);
        town.put(xr.floor() as i32, y, Item::Solid, steel, 0.6, -0.2);
        let seg = ((fy - 20.0) / 8.0).floor();
        let f = (fy - 20.0) / 8.0 - seg;
        let (a, b) = (mix(xl, xr, f), mix(xr, xl, f));
        town.put(a.floor() as i32, y, Item::Solid, steel.map(|v| v * 0.8), 0.0, -0.3);
        town.put(b.floor() as i32, y, Item::Solid, steel.map(|v| v * 0.8), 0.0, -0.3);
        if f < 0.13 {
            town.line((xl, fy), (xr, fy), Item::Solid, steel, 0.0, -1.0);
        }
    }
    town.line((262.0, 9.0), (262.0, 20.0), Item::Solid, steel, -0.5, 0.0);
    town.line((263.0, 9.0), (263.0, 20.0), Item::Solid, steel, 0.5, 0.0);
    for (x0, x1) in [(253, 257), (268, 272)] {
        for y in 34..46 {
            for x in x0..x1 {
                town.put(x, y, Item::Solid, [0.08, 0.08, 0.1], if x0 < 260 { -0.7 } else { 0.7 }, -0.2);
            }
        }
    }
    for (x, y) in [(262, 7), (263, 7), (262, 8), (263, 8)] {
        town.put(x, y, Item::Beacon, [1.0, 0.15, 0.1], 0.0, 0.0);
    }

    // the cables
    town.cable((258.0, 26.0), (58.0, 60.0), 26.0);
    town.cable((266.0, 31.0), (320.0, 66.0), 7.0);
    town.cable((257.0, 98.0), (190.0, 133.0), 8.0);
    town.cable((24.0, 44.0), (44.0, 57.0), 4.0);
    town.cable((70.0, 96.0), (88.0, 125.0), 5.0);

    // the signs' and lights' glow, spread into the air round them
    for &(lx, ly, c) in &town.lights {
        for dy in -9i32..=9 {
            for dx in -11i32..=11 {
                let (x, y) = (lx as i32 + dx, ly as i32 + dy);
                if x < 0 || y < 0 || x >= W as i32 || y >= LEDGE as i32 {
                    continue;
                }
                let d2 = f64::from(dx * dx + dy * dy);
                let w = 0.02 * (-d2 / 16.0).exp() as f32;
                let g = &mut town.glow[y as usize * W + x as usize];
                for i in 0..3 {
                    g[i] += w * c[i] as f32;
                }
            }
        }
    }

    // smog clouds, heaped overhead, lit from beneath by the city
    let density = |x: f64, y: f64| {
        let q = fbm_wrap(x * 0.006, y * 0.02, 3, CW as f64 * 0.006);
        fbm_wrap(x * 0.014 + q * 2.0, y * 0.04 + q, 4, CW as f64 * 0.014) + 0.05 * smooth(6.0, 30.0, y) - 0.16 * smooth(50.0, 84.0, y)
    };
    let mut cover = vec![0.0f32; CW * CH];
    let mut lit = vec![0.0f32; CW * CH];
    for y in 0..CH {
        for x in 0..CW {
            let (fx, fy) = (x as f64, y as f64);
            let d = density(fx, fy);
            cover[y * CW + x] = smooth(0.5, 0.66, d) as f32;
            lit[y * CW + x] = clamp(0.4 + (d - density(fx, fy + 3.0)) * 9.0) as f32;
        }
    }

    let koi = [
        Koi {
            l: 150.0,
            dir: -1.0,
            period: 34.0,
            t0: 15.3,
            x_start: 304.0,
            travel: 484.0,
            y0: 54.0,
            wave: [9.0, 0.3, 5.0, 1.0],
            swim: 2.4,
            seed: 1.0,
            base: [0.16, 0.8, 1.0],
            rim: [0.7, 1.0, 1.0],
            patch: [1.0, 0.2, 0.62],
            fin: [0.3, 0.75, 1.0],
            haze: 1.0,
            tancho: true,
            max: Layer::Mid,
        },
        Koi {
            l: 62.0,
            dir: 1.0,
            period: 34.0,
            t0: 6.7,
            x_start: 20.0,
            travel: 357.0,
            y0: 24.0,
            wave: [5.0, 2.0, 3.0, 0.4],
            swim: 1.5,
            seed: 2.0,
            base: [1.0, 0.32, 0.72],
            rim: [1.0, 0.82, 0.95],
            patch: [1.0, 0.66, 0.2],
            fin: [0.75, 0.3, 1.0],
            haze: 0.75,
            tancho: false,
            max: Layer::Mid,
        },
    ];
    let hor = (0..W).map(|x| f3(sky(x as f64, 112.0))).collect();
    Box::new(HoloKoi { town, puddle, hor, cover, lit, koi, buf: vec![[0.0; 3]; n], pixels: Pixels::new(&META) })
}

impl Piece for HoloKoi {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let HoloKoi { town, puddle, hor, cover, lit, koi, buf, pixels } = self;
        let big = koi[0].pose(t);
        let small = koi[1].pose(t);

        // the lights: the fish, the beacon blinking twice, the lens
        let mut srcs = Vec::with_capacity(8);
        koi[0].lights(&big, &[0.15, 0.45, 0.85], 0.4, 52.0, &mut srcs);
        koi[1].lights(&small, &[0.4], 0.28, 26.0, &mut srcs);
        let bp = (t / 1.8).rem_euclid(1.0);
        let beacon = if bp < 0.1 || (0.2..0.3).contains(&bp) { 1.0 } else { 0.06 };
        let lens = 0.8 + 0.2 * noise(t * 7.0, 3.0);
        let fixed = srcs.len();
        srcs.push(Src { x: BEACON.0, y: BEACON.1, c: [1.0 * beacon, 0.12 * beacon, 0.08 * beacon], r2: 70.0 });
        srcs.push(Src { x: LENS.0, y: LENS.1, c: [0.25 * lens, 0.75 * lens, 0.85 * lens], r2: 110.0 });

        // 1. the sky and the town
        let drift = t * 2.0;
        for py in 0..ROOF {
            let y = py as f64 + 0.5;
            for px in 0..W {
                let k = py * W + px;
                let x = px as f64 + 0.5;
                let mut c = c3(town.back[k]);
                let lay = town.layer[k];
                let f = match town.lamp[k] {
                    Lamp::Steady => 1.0,
                    Lamp::Flicker => {
                        let s = f64::from(town.seed[k]);
                        if hash((t / (2.5 + s * 4.0)).floor(), s * 9999.0) > 0.3 {
                            1.0
                        } else {
                            0.15
                        }
                    }
                    Lamp::Neon => {
                        let s = f64::from(town.seed[k]);
                        let fr = (t * 15.0).floor();
                        if hash(fr, s * 977.0) > 0.985 || (hash((t / 4.0 + s).floor(), s * 31.0) > 0.85 && hash(fr, s * 13.0) > 0.5) {
                            0.25
                        } else {
                            1.0
                        }
                    }
                    Lamp::Aviation => {
                        let ph = (t / 2.4 + f64::from(town.seed[k])).rem_euclid(1.0);
                        0.2 + 0.8 * smooth(0.0, 0.05, ph) * smooth(0.35, 0.1, ph)
                    }
                    Lamp::Led => 0.2 + 0.8 * smooth(0.0, 0.4, (f64::from(town.seed[k]) * 0.35 - t * 4.0).sin()),
                };
                if f != 1.0 {
                    c = c.map(|v| v * f);
                }
                let g = c3(town.glow[k]);
                match lay {
                    Layer::Sky | Layer::Far | Layer::Mid => {
                        if py < CH && lay == Layer::Sky {
                            // the clouds, drifting
                            let sx = x + drift;
                            let ix = sx.floor();
                            let fx = sx - ix;
                            let i0 = (ix as i64).rem_euclid(CW as i64) as usize;
                            let (i0, i1) = (py * CW + i0, py * CW + (i0 + 1) % CW);
                            let cv = mix(f64::from(cover[i0]), f64::from(cover[i1]), fx);
                            if cv > 0.01 {
                                let lt = mix(f64::from(lit[i0]), f64::from(lit[i1]), fx);
                                let cl = [0.07 + 0.12 * lt, 0.035 + 0.05 * lt, 0.1 + 0.12 * lt].map(|v| v * (0.6 + 0.6 * smooth(10.0, 70.0, y)));
                                c = mix3(c, cl, cv * 0.85);
                                let l = light(&srcs[..fixed], x, y);
                                c = [0, 1, 2].map(|i| c[i] + l[i] * cv * 0.18);
                            }
                        }
                        if lay != Layer::Sky {
                            // low smog, drifting between the towers
                            let fog = smooth(0.45, 0.8, noise(x * 0.03 + t * 0.12, y * 0.08)) * smooth(60.0, 118.0, y) * 0.4;
                            c = mix3(c, sky(x, y), fog);
                        }
                        let l = light(&srcs[..fixed], x, y);
                        let k_halo = if lay == Layer::Sky { 0.09 } else { 0.05 };
                        c = [0, 1, 2].map(|i| c[i] + g[i] + l[i] * k_halo);
                    }
                    Layer::Near => {
                        let l = light(&srcs[..fixed], x, y);
                        let r = 0.05 + 0.7 * f64::from(town.rim[k]);
                        c = [0, 1, 2].map(|i| c[i] + l[i] * r);
                    }
                    Layer::Roof => {
                        let l = light(&srcs, x, y);
                        let r = if py < LEDGE + 3 { 0.45 } else { 0.12 };
                        c = [0, 1, 2].map(|i| c[i] + l[i] * r);
                    }
                }
                buf[k] = c;
            }
        }

        // a drone, far off, crossing now and then
        let dp = (t + 7.0).rem_euclid(23.0);
        if dp < 16.0 {
            let (dx, dy) = ((-8.0 + dp * 22.0).floor() as i32, (28.0 + 2.0 * (t * 1.3).sin()).floor() as i32);
            let strobe = (t * 1.2).rem_euclid(1.0) < 0.08;
            for (ox, oy, c) in [
                (0, 0, [0.02, 0.02, 0.03]),
                (1, 0, [0.02, 0.02, 0.03]),
                (2, 0, [0.02, 0.02, 0.03]),
                (3, 0, [0.02, 0.02, 0.03]),
                (1, 1, [0.02, 0.02, 0.03]),
                (2, 1, [0.02, 0.02, 0.03]),
                (-1, 0, [0.9, 0.12, 0.1]),
                (4, 0, [0.2, 1.0, 0.4]),
                (2, -1, if strobe { [1.0, 1.0, 1.0] } else { [0.05, 0.05, 0.06] }),
            ] {
                let (x, y) = (dx + ox, dy + oy);
                if x >= 0 && (x as usize) < W && y >= 0 {
                    let k = y as usize * W + x as usize;
                    if town.layer[k] <= Layer::Mid {
                        buf[k] = c;
                    }
                }
            }
        }

        // 2. the fish: the small one behind the middle towers, the big one in front
        koi[1].draw(&small, buf, &town.layer);
        koi[0].draw(&big, buf, &town.layer);

        // 3. the wet roof: puddles give back the sky, the rest a sheen
        for py in ROOF..H {
            let y = py as f64 + 0.5;
            let near = smooth(ROOF as f64, H as f64, y);
            for (px, hz) in hor.iter().enumerate() {
                let k = py * W + px;
                let x = px as f64 + 0.5;
                let l = light(&srcs, x, y);
                let base = c3(town.back[k]);
                let tex = noise(x * 0.3, y * 0.6);
                // wet, it gives back the smog's glow, most where we see it glancing
                let hz = c3(*hz);
                let fres = 0.05 + 0.4 * (1.0 - near).powi(2);
                let mut c = [0, 1, 2].map(|i| base[i] + hz[i] * fres * (0.7 + 0.3 * tex) + l[i] * (0.12 + 0.2 * tex) * (1.0 - 0.4 * near));
                let p = f64::from(puddle[k]);
                if p > 0.0 {
                    let wob = 0.8 * (y * 1.3 + t * 5.0).sin() + 0.6 * noise(x * 0.2, y * 0.5 + t * 2.0) - 0.3;
                    let my = (2.0 * MIRROR - y + wob * 0.5).clamp(0.0, ROOF as f64 - 1.0) as usize;
                    let mx = (x + wob).clamp(0.0, W as f64 - 1.0) as usize;
                    let r = buf[my * W + mx];
                    let refl = [r[0] * 0.55 + 0.01, r[1] * 0.58 + 0.01, r[2] * 0.66 + 0.02];
                    c = mix3(c, [0, 1, 2].map(|i| refl[i] + l[i] * 0.12), p);
                }
                buf[k] = c;
            }
        }
        // rings where drops land in the puddles
        let rs = t / 0.3;
        for back in 0..3 {
            let j = rs.floor() - back as f64;
            let age = (rs - j) / 3.0;
            for i in 0..12 {
                let fi = i as f64;
                let (rx, ry) = (hash(j * 3.0, fi) * W as f64, ROOF as f64 + 3.0 + hash(fi, j * 3.0 + 1.0) * (H - ROOF - 4) as f64);
                let rad = 1.0 + age * 6.0;
                let b = (1.0 - age) * 0.3;
                for s in 0..24 {
                    let a = s as f64 / 24.0 * TAU;
                    let (x, y) = (rx + rad * a.cos(), ry + rad * 0.35 * a.sin());
                    if x < 0.0 || y < 0.0 || x >= W as f64 || y >= H as f64 {
                        continue;
                    }
                    let k = y as usize * W + x as usize;
                    if puddle[k] > 0.5 {
                        let l = light(&srcs, x, y);
                        buf[k] = [0, 1, 2].map(|c| buf[k][c] + b * (0.35 + l[c] * 1.5));
                    }
                }
            }
        }

        // 4. steam from the vent, drifting right
        for py in 40..104 {
            let y = py as f64 + 0.5;
            let h = 104.0 - y;
            let cx = 147.5 + h * 0.35 + 3.0 * (h * 0.08 - t * 1.2).sin();
            let wd = 2.0 + h * 0.22;
            let (xa, xb) = ((cx - 2.5 * wd).max(0.0) as usize, ((cx + 2.5 * wd) as usize).min(W));
            for px in xa..xb {
                let x = px as f64 + 0.5;
                let prof = (-((x - cx) / wd).powi(2)).exp() * smooth(0.0, 6.0, h) * smooth(64.0, 18.0, h);
                if prof < 0.02 {
                    continue;
                }
                let d = smooth(0.3, 0.7, fbm(x * 0.09, (y + t * 14.0) * 0.07, 3)) * prof * 1.4;
                let k = py * W + px;
                let l = light(&srcs, x, y);
                buf[k] = [0, 1, 2].map(|i| buf[k][i] * (1.0 - 0.4 * d) + d * ([0.2, 0.18, 0.26][i] * 0.6 + l[i] * 1.2));
            }
        }

        // 5. the roof's furniture, lit by what is round it
        let (bx, by) = (big.hx - koi[0].dir * 0.45 * koi[0].l, big.spine_at(0.45));
        let beam_vis = smooth(-60.0, 0.0, bx) * smooth(380.0, 320.0, bx) * (1.0 - 0.7 * big.glitch);
        for py in 0..H {
            for px in 0..W {
                let k = py * W + px;
                let kind = town.item[k];
                if kind == Item::None {
                    continue;
                }
                let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
                let base = c3(town.icol[k]);
                let c = match kind {
                    Item::Lens => base.map(|v| v * (0.75 + 0.25 * lens) * (0.4 + 0.6 * beam_vis)),
                    Item::Beacon => base.map(|v| v * (0.25 + 0.75 * beacon)),
                    Item::Led => base.map(|v| v * if hash((t * 3.0 + f64::from(town.inorm[k][0]) * 0.5).floor(), 9.0) > 0.4 { 1.0 } else { 0.2 }),
                    _ => {
                        let [nx, ny] = town.inorm[k].map(f64::from);
                        let mut c = base;
                        if kind == Item::Fan {
                            let a = (y - 127.5).atan2((x - 104.5) / 3.0) * 4.0 - t * 9.0;
                            c = c.map(|v| v * (1.0 + 1.6 * smooth(0.2, 0.9, a.sin())));
                        }
                        // the smog's glow catches the edges that turn away from us
                        let edge = 0.9 * nx.abs().powi(3) + 0.6 * (-ny).max(0.0);
                        c = [0, 1, 2].map(|i| c[i] + [0.1, 0.05, 0.12][i] * edge);
                        let bead = kind == Item::Cable && hash(x, y * 5.0) > 0.9;
                        for s in &srcs {
                            let (dx, dy) = (s.x - x, s.y - y);
                            let d2 = dx * dx + dy * dy;
                            let d = d2.sqrt().max(1.0);
                            let face = if bead { 1.0 } else { 0.2 + 0.8 * ((nx * dx + ny * dy) / d).max(0.0) };
                            let w = s.r2 / (s.r2 + d2) * face * if bead { 2.5 } else { 0.75 };
                            for (ci, si) in c.iter_mut().zip(s.c) {
                                *ci += si * w;
                            }
                        }
                        c
                    }
                };
                buf[k] = c;
            }
        }

        // 6. the beam, the glow of the beacon and the lens, the rain, the splashes
        let (vx, vy) = (bx - LENS.0, by - LENS.1);
        let dist = vx.hypot(vy).max(1.0);
        let (nx, ny) = (vx / dist, vy / dist);
        let slot = (t * 12.0).floor();
        for py in 0..H {
            let y = py as f64 + 0.5;
            for px in 0..W {
                let k = py * W + px;
                let x = px as f64 + 0.5;
                let mut c = buf[k];
                let mut beam = 0.0;
                if py < ROOF && beam_vis > 0.0 && town.item[k] == Item::None {
                    let (rx, ry) = (x - LENS.0, y - LENS.1);
                    let al = rx * nx + ry * ny;
                    if al > 0.0 && al < dist * 1.05 {
                        let perp = (rx * ny - ry * nx).abs();
                        let wd = 1.0 + al / dist * 30.0;
                        if perp < wd {
                            let q = perp / wd;
                            let streak = 0.6 + 0.4 * ((rx * ny - ry * nx) / wd * 9.0 + t * 0.7).sin();
                            beam = (1.0 - q).powf(1.5) * streak * smooth(dist * 1.05, dist * 0.7, al) * beam_vis * (0.05 + 0.25 * (-al / 8.0).exp());
                            c = [c[0] + beam * 0.25, c[1] + beam * 0.85, c[2] + beam];
                        }
                    }
                }
                let dl = light(&srcs[fixed..], x, y);
                c = [0, 1, 2].map(|i| c[i] + dl[i] * 0.25);
                let r = rain(x, y, t);
                if r > 0.0 {
                    let l = light(&srcs, x, y);
                    c = [0, 1, 2].map(|i| c[i] + r * ([0.16, 0.15, 0.26][i] + l[i] * 1.6 + beam * [3.0, 9.0, 10.0][i]));
                }
                buf[k] = c;
            }
        }
        for i in 0..14 {
            let fi = i as f64;
            let (sx, sy) = ((hash(slot, fi + 100.0) * W as f64) as usize, ROOF + 3 + (hash(fi + 200.0, slot) * (H - ROOF - 3) as f64) as usize);
            if sy < H && town.item[sy * W + sx] == Item::None {
                let l = light(&srcs, sx as f64, sy as f64);
                let c = [0, 1, 2].map(|i| 0.3 + l[i] * 1.5);
                for (ox, oy, w) in [(0i32, 0i32, 1.0), (-1, -1, 0.5), (1, -1, 0.5)] {
                    let (x, y) = (sx as i32 + ox, sy as i32 + oy);
                    if x >= 0 && (x as usize) < W {
                        let k = y as usize * W + x as usize;
                        buf[k] = [0, 1, 2].map(|j| buf[k][j] + c[j] * w);
                    }
                }
            }
        }

        let px = pixels.pixels_mut();
        for (o, c) in px.iter_mut().zip(buf.iter()) {
            *o = f3(*c);
        }
        pixels.draw(grid, env);
    }
}
