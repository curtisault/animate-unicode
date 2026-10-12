//! night coast: a lighthouse on a wooded headland under moonlit clouds. The
//! beam turns every eight seconds, the clouds drift, and the sea carries the
//! moon's road and the lamp's reflection. After ascii.rest's night-coast.ts
//! (MIT, @bas3line).
//!
//! The headland, the clouds, the sea and the beam, and how each is shaded,
//! are ascii.rest's, shaded here per pixel instead of per cell. Where
//! ascii.rest draws each of 200 × 100 square cells as a dot as big as it is
//! bright, this draws 320 × 80 cells of two square pixels each, an upper half
//! block over a back colour ([`crate::pixels`]): 1.6 times the detail each
//! way, solid colour, and gradients dithered in colour rather than in dot
//! size. The palette adds the dark tones a solid
//! pixel needs where a small dot was enough. In one ink each pixel is octant
//! dots, as many as it is bright.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Night coast's meta.
pub static META: Meta = Meta {
    name: "night coast",
    slug: "night-coast",
    category: Category::Nights,
    note: "a lighthouse turning its beam under moonlit clouds over a dark sea",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: moonlit blues to white, the lamp's ambers, the woods,
    // stone and slate, the roof and the buoy. Then the night's darks.
    palette: Some(&[
        "#15203d", "#1b2a50", "#223463", "#2b4077", "#364e8b", "#445f9f", "#5874b2", "#7089c0", "#8a9cc4", "#a3b1cf", "#bec9dc", "#d8dfea", "#eef1f6",
        "#f7f8fc", "#ffe9ae", "#ffd27c", "#f3a64a", "#c46c2d", "#121c1b", "#182724", "#203530", "#2c473d", "#3c5c4c", "#232a36", "#363e4c", "#555d6c",
        "#2a3550", "#3a4868", "#9c4136", "#ff5d4d", "#080b12", "#0b1120", "#0f172c", "#0a1214", "#0d1714", "#14100c", "#2a2418", "#4a3a22",
    ]),
    ground: Some("#080b12"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
const HORIZON: f64 = 60.0;
const MOON: (f64, f64) = (150.0, 19.0);
const LAMP_X: f64 = 30.0;
/// The cloud deck's width in columns: it wraps, so it can drift forever.
const CW: usize = 640;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Air,
    Land,
    Tree,
    Tower,
    Lantern,
    Cap,
    Wall,
    Pane,
    Roof,
    Isle,
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

/// The headland's top at column x.
fn top(x: f64) -> f64 {
    HORIZON - 15.0 * smooth(86.0, 50.0, x) * (1.0 - 0.12 * smooth(24.0, 0.0, x)) - 1.6 * fbm(x * 0.15, 3.7, 3)
}

/// The coast: what each pixel is and its shade, the sky's haze, the cloud
/// deck's cover and light, and the picture.
pub struct NightCoast {
    mat: Vec<Mat>,
    shade: Vec<f32>,
    haze: Vec<f32>,
    cover: Vec<f32>,
    lit: Vec<f32>,
    tower_top: f64,
    pixels: Pixels,
}

/// A night coast; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let mut mat = vec![Mat::Air; n];
    let mut shade = vec![0.0f32; n];
    let base = top(LAMP_X);
    let tower_top = base - 15.0;
    let mut trees = vec![];
    let mut x = 1.0;
    while x < 76.0 {
        // the clearing for the tower and the cottage
        if !(x > 22.0 && x < 49.0) {
            trees.push((x + hash(x, 2.0) * 1.2, (4.0 + hash(x, 3.0) * 7.0) * smooth(78.0, 56.0, x), 2.0 + hash(x, 4.0) * 1.6));
        }
        x += 2.6 + hash(x * 7.0, 1.0) * 2.6;
    }
    let cottage = top(41.0);
    for py in 0..H {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            let t0 = top(x);
            let isle = HORIZON - 4.5 * (1.0 - ((x - 180.0) / 26.0).powi(2)).max(0.0) - 1.4 * fbm(x * 0.2, 9.0, 2);
            let mut m = Mat::Air;
            let mut s = 0.0;
            if y >= t0 && y < HORIZON + 4.0 && x < 90.0 {
                m = Mat::Land;
                // rock and scrub, the brow of the slope catching the moon
                s = 0.2 + 0.45 * fbm(x * 0.35, y * 0.35, 3) + 0.5 * smooth(t0 + 3.0, t0, y) - 0.2 * smooth(HORIZON - 4.0, HORIZON + 4.0, y);
            } else if x > 148.0 && y >= isle && y < HORIZON {
                m = Mat::Isle;
            }
            for &(tx, th, tw) in &trees {
                if th < 1.0 {
                    continue;
                }
                let tb = top(tx);
                let dy = y - (tb - th);
                if dy >= 0.0 && y < tb + 2.0 && (x + 0.5 - tx).abs() <= (dy / th) * tw + 0.4 {
                    m = Mat::Tree;
                    s = 0.1 + 0.35 * hash(x * 13.0 + y.floor(), 5.0) + if x + 0.5 > tx { 0.35 } else { 0.0 };
                }
            }
            // the tower: tapered, white, lit from the moon on its right
            let dx = x + 0.5 - LAMP_X;
            if y >= tower_top && y < base + 2.0 {
                let hw = 3.0 - 1.1 * smooth(base, tower_top, y);
                if dx.abs() <= hw {
                    m = Mat::Tower;
                    s = 0.45 + 0.5 * (dx / hw);
                }
            }
            if y >= tower_top - 1.0 && y < tower_top + 1.0 && dx.abs() <= 3.6 {
                (m, s) = (Mat::Cap, 0.55);
            }
            if y >= tower_top - 7.0 && y < tower_top - 1.0 && dx.abs() <= 2.0 {
                m = Mat::Lantern;
            }
            if y >= tower_top - 10.0 && y < tower_top - 7.0 && dx.abs() <= 2.8 - (tower_top - 7.0 - y) * 0.8 {
                (m, s) = (Mat::Cap, 0.4);
            }
            // the keeper's cottage, lit inside
            if (36.0..=47.0).contains(&x) && y >= cottage - 5.0 && y < cottage + 1.0 {
                m = Mat::Wall;
                s = 0.3 + 0.4 * (x - 36.0) / 11.0;
                if (x.round() == 39.0 || x.round() == 44.0) && y >= cottage - 4.0 && y < cottage - 1.5 {
                    m = Mat::Pane;
                }
            }
            if y >= cottage - 10.0 && y < cottage - 5.0 && (x + 0.5 - 42.0).abs() <= 7.5 - (cottage - 5.0 - y) * 1.2 {
                m = Mat::Roof;
            }
            mat[k] = m;
            shade[k] = s as f32;
        }
    }

    // a faint unevenness in the clear sky, so it is never one flat tone
    let haze = (0..n).map(|k| fbm(col_x(k % W) * 0.05, (row_y(k / W) - 0.5) * 0.08, 3) as f32).collect();

    // The clouds: heaps of noise that wrap, so they can drift forever.
    let hz = (HORIZON * S).ceil() as usize;
    let density = |x: f64, y: f64| {
        let q = fbm_wrap(x * 0.008, y * 0.02, 3, CW as f64 * 0.008);
        let d = fbm_wrap(x * 0.018 + q * 2.4, y * 0.036 + q * 1.1, 5, CW as f64 * 0.018);
        // heaped mid-sky, thinner overhead, wisps at the horizon
        d + 0.05 * smooth(8.0, 22.0, y) - 0.04 * smooth(10.0, 0.0, y) - 0.16 * smooth(36.0, HORIZON - 4.0, y)
    };
    let mut cover = vec![0.0f32; CW * hz];
    let mut lit = vec![0.0f32; CW * hz];
    for py in 0..hz {
        let y = row_y(py);
        for x in 0..CW {
            let d = density(x as f64, y);
            cover[py * CW + x] = smooth(0.52, 0.63, d) as f32;
            // Lit on the side facing the moon (up and right), shadowed
            // underneath, a little darker deep inside.
            let toward = density(x as f64 + 2.5, y - 3.0);
            lit[py * CW + x] = clamp(0.5 + (d - toward) * 11.0 - (d - 0.6) * 1.2) as f32;
        }
    }
    Box::new(NightCoast { mat, shade, haze, cover, lit, tower_top, pixels: Pixels::new(&META) })
}

impl Piece for NightCoast {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let beam = t / 8.0 * TAU - 0.75; // starts out over the sea
        let (cb, sb) = (beam.cos(), beam.sin());
        let flash = sb.max(0.0).powi(14) * 2.2; // pointed at us
        let lamp_y = self.tower_top - 4.0;
        let drift = t * 1.4;
        let ground = [0.031, 0.043, 0.071];
        for py in 0..H {
            let y = row_y(py);
            for px in 0..W {
                let k = py * W + px;
                let x = col_x(px);
                let m = self.mat[k];
                let s = f64::from(self.shade[k]);
                let (mut cr, mut cg, mut cbl, mut fade) = (0.0, 0.0, 0.0, 1.0);
                let (dmx, dmy) = (x + 0.5 - MOON.0, y - MOON.1);
                let dm = dmx.hypot(dmy);

                if y < HORIZON && m == Mat::Air {
                    // sky: deep navy up top to a hazy horizon, brighter round the moon
                    let v = y / HORIZON;
                    let halo = (-dm / 24.0).exp() * 0.35 + (-dm / 9.0).exp() * 0.45;
                    let glow_h = v.powi(3) * 0.24; // the horizon's last light
                    let veil = 0.85 + 0.3 * f64::from(self.haze[k]);
                    cr = (0.04 + 0.1 * v * v + glow_h * 0.9) * veil + 0.3 * halo;
                    cg = (0.07 + 0.13 * v * v + glow_h) * veil + 0.35 * halo;
                    cbl = (0.17 + 0.2 * v * v + glow_h * 1.1) * veil + 0.42 * halo;
                    let sx = x + drift;
                    let ix = sx.floor();
                    let fx = sx - ix;
                    let i0 = (ix as i64).rem_euclid(CW as i64) as usize;
                    let (i0, i1) = (py * CW + i0, py * CW + (i0 + 1) % CW);
                    let (c0, c1) = (f64::from(self.cover[i0]), f64::from(self.cover[i1]));
                    // a break in the cloud round the moon
                    let c = (c0 + (c1 - c0) * fx) * (0.15 + 0.85 * smooth(7.0, 24.0, dm));
                    let l = f64::from(self.lit[i0]) + f64::from(self.lit[i1] - self.lit[i0]) * fx;
                    if dm < 6.0 {
                        // the moon, dimmed where cloud crosses it
                        let face = 0.86 + 0.14 * fbm(x * 0.5, y * 0.5, 2);
                        let a = (1.0 - c * 0.8) * smooth(6.0, 5.0, dm);
                        cr = mix(cr, 0.96 * face, a);
                        cg = mix(cg, 0.97 * face, a);
                        cbl = mix(cbl, face, a);
                    } else if c < 0.05 && hash(px as f64, py as f64 * 3.0 + 11.0) > 0.993 {
                        // a star, a pixel big, twinkling
                        let (fx, fy) = (px as f64, py as f64);
                        let tw = 0.6 + 0.4 * (t * (1.5 + hash(fx, fy) * 3.0) + hash(fy, fx) * TAU).sin();
                        let s = (0.5 + 0.5 * tw) * (1.0 - halo) * smooth(HORIZON, 20.0, y);
                        (cr, cg, cbl) = (cr.max(s * 0.92), cg.max(s * 0.94), cbl.max(s));
                    }
                    if c > 0.01 {
                        // shadow slate, through moonlit grey, to a silver rim near
                        // the moon; low cloud is lit from below by the horizon and
                        // stays hazy
                        let near = (-dm / 30.0).exp();
                        let b = clamp(l * (0.5 + 0.6 * near) + 0.25 * smooth(30.0, HORIZON, y));
                        let ramp = |lo: f64, md: f64, hi: f64| if b < 0.5 { mix(lo, md, b * 2.0) } else { mix(md, hi, (b - 0.5) * 2.0) };
                        let a = (c * 1.1).min(1.0);
                        cr = mix(cr, ramp(0.1, 0.36, 0.95), a);
                        cg = mix(cg, ramp(0.13, 0.42, 0.96), a);
                        cbl = mix(cbl, ramp(0.25, 0.58, 1.0), a);
                    }
                } else if y >= HORIZON && (m == Mat::Air || m == Mat::Land) && !(m == Mat::Land && y < HORIZON + 4.0) {
                    // sea: cold, darker toward us, waves stretched along the swell
                    let v = (y - HORIZON) / (100.0 - HORIZON);
                    let w = 0.6 * noise(x * 0.07 + t * 0.12, y * 0.45 - t * 0.6) + 0.4 * noise(x * 0.2 - t * 0.25, y * 0.9 - t * 1.1);
                    let swell = 0.45 + 0.95 * w;
                    cr = (0.08 - 0.04 * v) * swell;
                    cg = (0.13 - 0.06 * v) * swell;
                    cbl = (0.28 - 0.12 * v) * swell;
                    // the moon's road: wider toward us, broken into glints
                    let road_w = 3.0 + (y - HORIZON) * 0.55;
                    let road = (-((x + 0.5 - MOON.0) / road_w).powi(2)).exp();
                    let glint = smooth(0.5, 0.8, w) * road;
                    cr += 0.95 * glint + 0.07 * road;
                    cg += 0.95 * glint + 0.09 * road;
                    cbl += 0.95 * glint + 0.15 * road;
                    // the lamp's reflection, warm, under the tower
                    let lw = 1.3 + (y - HORIZON) * 0.22;
                    let refl = (-((x + 0.5 - LAMP_X) / lw).powi(2)).exp() * smooth(0.45, 0.8, w) * (0.75 + 0.6 * flash);
                    cr += refl;
                    cg += 0.72 * refl;
                    cbl += 0.32 * refl;
                    // surf where the headland meets the water
                    if y < HORIZON + 8.0 && x < 90.0 {
                        let edge = smooth(90.0, 74.0, x) * smooth(HORIZON + 8.0, HORIZON + 3.0, y);
                        let foam = smooth(0.5, 0.85, noise(x * 0.45 - t * 0.6, y * 0.8 + t * 0.4)) * edge;
                        cr += 0.65 * foam;
                        cg += 0.7 * foam;
                        cbl += 0.75 * foam;
                    }
                    let haze = (-(y - HORIZON) / 2.5).exp() * 0.16;
                    cr += haze * 0.8;
                    cg += haze * 0.9;
                    cbl += haze;
                    fade = smooth(100.0, 74.0, y); // the bottom rows sink into the ground
                } else {
                    (cr, cg, cbl) = match m {
                        Mat::Land => (0.05 + 0.14 * s, 0.09 + 0.2 * s, 0.1 + 0.18 * s),
                        Mat::Tree => (0.03 + 0.07 * s, 0.07 + 0.14 * s, 0.07 + 0.1 * s),
                        Mat::Isle if x.round() == 184.0 && y.floor() == 55.0 => {
                            // a buoy light out on the point, blinking
                            if (t * 2.2).sin() > 0.55 {
                                (1.0, 0.36, 0.3)
                            } else {
                                (0.35, 0.12, 0.1)
                            }
                        }
                        Mat::Isle => (0.06, 0.08, 0.15),
                        Mat::Tower => (0.4 + 0.58 * s, 0.42 + 0.56 * s, 0.48 + 0.52 * s),
                        Mat::Cap => ((0.4 + 0.58 * s) * 0.7, (0.42 + 0.56 * s) * 0.66, (0.48 + 0.52 * s) * 0.68),
                        Mat::Lantern => {
                            let g = 0.85 + 0.15 * flash.min(1.0);
                            (g, 0.84 * g, 0.5 * g)
                        }
                        Mat::Wall => (0.3 + 0.4 * s, 0.32 + 0.4 * s, 0.38 + 0.4 * s),
                        Mat::Pane => {
                            let g = 0.85 + 0.15 * (t * 3.0 + x).sin();
                            (g, 0.72 * g, 0.32 * g)
                        }
                        Mat::Roof => (0.4, 0.17, 0.14),
                        Mat::Air => (cr, cg, cbl),
                    };
                }

                // The beam: a soft cone from the lamp to one side, shortened as
                // it turns toward or away from us, and a glow that flares when
                // it faces us.
                let (bx, by) = (x + 0.5 - LAMP_X, y - lamp_y);
                if !matches!(m, Mat::Tower | Mat::Cap | Mat::Lantern) {
                    if bx * cb > 0.0 {
                        let along = bx.abs() / (cb.abs() * 115.0 + 1.0);
                        if along < 1.0 {
                            let spread = 1.4 + bx.abs() * 0.11;
                            let b = (1.0 - along).powf(1.8) * (-(by / spread).powi(2)).exp() * 0.85;
                            cr += b;
                            cg += b * 0.84;
                            cbl += b * 0.5;
                        }
                    }
                    let dl = bx.hypot(by);
                    let glow = (-dl / (3.5 + 3.0 * flash)).exp() * (0.5 + 0.6 * flash) + (-dl / 14.0).exp() * 0.12;
                    cr += glow;
                    cg += glow * 0.82;
                    cbl += glow * 0.5;
                }
                let px_c = [mix(ground[0], cr, fade) as f32, mix(ground[1], cg, fade) as f32, mix(ground[2], cbl, fade) as f32];
                self.pixels.set(px, py, px_c);
            }
        }
        self.pixels.draw(grid, env);
    }
}
