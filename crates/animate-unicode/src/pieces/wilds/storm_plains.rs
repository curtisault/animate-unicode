//! storm plains: an anvil thunderhead at dusk over open wheat country. The
//! sun has just set behind the farmhouse, so the storm's top and western flank
//! still catch its light while the base sinks into slate shadow. Lightning
//! flickers inside the cloud, now and then a bolt reaches the ground, rain
//! curtains drift under the base and the wheat moves in gusts. After
//! ascii.rest's storm-plains.ts (MIT, @bas3line).
//!
//! The cloud is ascii.rest's height field (heaped domes, a flat anvil,
//! pouches hanging under it) lit from its surface normals, and the farm, the
//! wheat, the rain and the lightning are its too, shaded here per pixel
//! instead of per cell. Where ascii.rest draws each of 200 × 100 square cells
//! as a dot as big as it is bright, this draws 320 × 80 cells of two square
//! pixels each, an upper half block over a back colour ([`crate::pixels`]):
//! 1.6 times the detail each way, solid colour, and gradients dithered in
//! colour rather than in dot size. The height field is built at pixel
//! resolution too, so the billows and the anvil's streaks are finer, and the
//! rain falls in streaks a pixel wide. The near wheat ears are drawn in
//! pixels, their stalks a dark umber line. The palette adds the dark tones a
//! solid pixel needs where a small dot was enough. In one ink each pixel is
//! octant dots, as many as it is bright.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// Storm plains' meta.
pub static META: Meta = Meta {
    name: "storm plains",
    slug: "storm-plains",
    category: Category::Wilds,
    note: "an anvil thunderhead at dusk flickering over a wheat field",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: night violets, zenith to dusk; dusty mauve, the shaded
    // flank; peach and gold, the sunlit tops; amber afterglow; wheat, gold to
    // umber; rain and slate; lightning; lamplight. Then the dusk's darks.
    palette: Some(&[
        "#0f0c20", "#141029", "#1c1638", "#261c4a", "#32235a", "#40306c", "#53407e", "#6a5590", "#8670a6", "#7a6288", "#9a7890", "#b98e94", "#e6a890",
        "#f2bca8", "#f7d6a8", "#fbe8c6", "#f8b070", "#e88a4c", "#c8643a", "#9a4630", "#6a3226", "#f0c868", "#d8a447", "#b7843a", "#8c6230", "#5e4226",
        "#3a2a1c", "#261c14", "#181a2c", "#2a3048", "#3a4260", "#4f5878", "#6a7392", "#8890ae", "#a2aabd", "#5d5878", "#7c7598", "#a6a2c8", "#d6d2fa",
        "#f4f2ff", "#ffe08a", "#ffb84a", "#0b0912", "#07061a", "#0b0b28", "#100d1c", "#17121f", "#1e1828", "#2a2034", "#100c0a", "#18110c", "#22180f",
        "#2e2014", "#0f1020", "#1c1a30", "#2c2240", "#4a2a26", "#3a2030",
    ]),
    ground: Some("#0b0912"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame, in its cells.
const SW: f64 = 200.0;
const SH: f64 = 100.0;
/// The horizon.
const HZ: f64 = 64.0;
/// Just below the horizon, behind the farmhouse.
const SUN: (f64, f64) = (30.0, 70.0);
/// The storm's flat base.
const BASE: f64 = 45.0;
/// The height field's margin either side, in pixels, and its size.
const MX: usize = 3;
const SXP: usize = W + 2 * MX;
const SYP: usize = 85;
/// The first pixel row of the land's textures: everything below the sky.
const LAND: usize = 102;
/// The wheat's and the gusts' texture widths, wrapping.
const TW: usize = 512;
const GW: usize = 512;
/// The altostratus deck's width.
const CW: usize = 400;
/// The rain shafts' envelope's width.
const RW: usize = 256;
/// Where the wheat's columns converge: the farmhouse.
const VX: f64 = 57.0;
/// The farmhouse's walls run from x 40 to 54, eaves at row 58.
const HX0: f64 = 40.0;
const HX1: f64 = 54.0;
const HW0: f64 = 58.0;
const PUMP_X: f64 = 66.0;
/// Toward the light: west, a touch high, out of the page.
const LIGHT: (f64, f64, f64) = (-0.72, -0.3, 0.62);
/// The range of a twinkle's or a sway's phase: ascii.rest's 6.28, kept so
/// every phase is the TypeScript's.
#[allow(clippy::approx_constant)]
const PHASE: f64 = 6.28;
const GROUND: [f64; 3] = [11.0 / 255.0, 9.0 / 255.0, 18.0 / 255.0];
/// How much the cloud's middle tones are deepened: a power on each channel.
const DEEPEN: f64 = 1.4;
/// Seconds between big flashes, and between the flickers inside the cloud.
const PERIOD: f64 = 3.1;
const FLICKER: f64 = 1.6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Sky,
    Plain,
    Field,
    Road,
    House,
    Roof,
    Pane,
    Tree,
    Pump,
    Belt,
}

/// The scene's x, in ascii.rest's columns, at the middle of pixel column `px`.
fn xc(px: usize) -> f64 {
    (px as f64 + 0.5) / S
}

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn yc(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

fn dome(dx: f64, dy: f64, r: f64) -> f64 {
    let q = 1.0 - dx * dx - dy * dy;
    if q > 0.0 {
        r * q.sqrt()
    } else {
        0.0
    }
}

/// Distance from (px, py) to the segment a-b.
fn seg_dist(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    let (dx, dy) = (bx - ax, by - ay);
    let l = dx * dx + dy * dy;
    let k = clamp(((px - ax) * dx + (py - ay) * dy) / if l == 0.0 { 1.0 } else { l });
    (px - ax - dx * k).hypot(py - ay - dy * k)
}

fn tower_c(y: f64) -> f64 {
    138.0 + (BASE - y) * 0.14
}
fn tower_hw(y: f64) -> f64 {
    23.0 - (BASE - y) * 0.08
}
fn anvil_top(x: f64) -> f64 {
    7.5 + 2.5 * smooth(150.0, 210.0, x) + 3.0 * smooth(112.0, 58.0, x)
}
fn anvil_bot(x: f64) -> f64 {
    21.0 - 9.0 * smooth(124.0, 62.0, x) - 7.0 * smooth(152.0, 210.0, x)
}
fn glow_sun(x: f64, y: f64) -> f64 {
    let (dx, dy) = (x - SUN.0, y - SUN.1);
    (-(dx / 48.0).powi(2) - (dy / 10.0).powi(2)).exp() * 0.9 + (-(dx / 95.0).powi(2) - (dy / 20.0).powi(2)).exp() * 0.26
}
/// The land's line at x: the farmhouse stands on a low rise.
fn ground(x: f64) -> f64 {
    HZ + 3.0 - 1.2 * smooth(20.0, 70.0, x) * smooth(110.0, 70.0, x)
}
/// The shelf: a flat dark lip of cloud along the storm's leading edge.
fn shelf_top(x: f64) -> f64 {
    BASE - 2.4 + 1.2 * (noise(x * 0.2, 8.1) - 0.5)
}
fn shelf_bot(x: f64) -> f64 {
    BASE + 2.4 + 1.8 * (fbm(x * 0.12, 8.7, 2) - 0.5) - 2.0 * smooth(184.0, 199.0, x) - 2.0 * smooth(114.0, 104.0, x)
}
fn shelf_on(x: f64) -> f64 {
    smooth(103.0, 112.0, x) * smooth(199.0, 190.0, x)
}
/// The road, from the yard to the bottom edge: its middle and half width `p`
/// of the way down the land.
fn road_c(p: f64) -> f64 {
    57.0 + 62.0 * p.powf(1.25)
}
fn road_w(p: f64) -> f64 {
    0.6 + 12.0 * p
}

/// A flash of lightning or a flicker inside the cloud.
struct Flash {
    i: f64,
    bolt: Option<usize>,
    cx: f64,
    cy: f64,
    bolt_on: f64,
    inside: bool,
}

/// The plains: the storm, the sky and the land, built once, and the picture.
pub struct StormPlains {
    mat: Vec<Mat>,
    /// Every pixel's colour before anything moves.
    base: Vec<[f64; 3]>,
    /// Storm cover, for the lightning.
    cloud: Vec<f32>,
    /// The base and the shelf, lit from behind by a flash.
    dark: Vec<f32>,
    /// The wheat's texture, a row for each pixel row of the land, and each
    /// pixel's column in it.
    wheat: Vec<f32>,
    persp: Vec<f32>,
    /// How much light the field holds.
    field_light: Vec<f32>,
    /// Gusts: soft patches of bent, brighter wheat, a row for each pixel row of the land.
    gust: Vec<f32>,
    /// Thin glowing streaks of altostratus, a row for each pixel row of the sky.
    streak: Vec<f32>,
    /// [pixel, phase, rate, brightness]
    stars: Vec<(usize, f64, f64, f64)>,
    /// The wheat ears right in front of us: [x, stalk height, phase, foot
    /// below the frame, head length].
    ears: Vec<[f64; 5]>,
    /// This frame's ears: > 0 a lit grain head, < 0 a dark stalk.
    ear: Vec<f32>,
    touched: Vec<usize>,
    shafts: Vec<f32>,
    col_phase: Vec<f32>,
    col_speed: Vec<f32>,
    rain_top: Vec<f32>,
    rain_on: Vec<f32>,
    /// The four bolts: where each starts, and every pixel's distance from it.
    bolts: Vec<(f64, Vec<f32>)>,
    pixels: Pixels,
}

/// A storm over the plains; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    // --- the thunderhead, as a height field ----------------------------------
    // The towers are heaps of puffs: [x, y, radius, bulge toward us, cut flat].
    // Each puff is a dome in the height field, so it gets its own lit side and
    // shadow.
    let mut puffs: Vec<[f64; 5]> = vec![];
    let mut seed = 1.0;
    let mut rnd = || {
        seed += 1.0;
        hash(seed - 1.0, 911.0)
    };
    let mut y = BASE - 4.5;
    while y > 14.0 {
        let (cx, hw) = (tower_c(y), tower_hw(y));
        let n = ((hw * 2.0) / 11.0).round().max(2.0) as usize;
        for i in 0..n {
            let u = ((i as f64 + 0.5) / n as f64) * 2.0 - 1.0;
            let rr = 7.0 + rnd() * 4.0;
            let px = cx + u * (hw - rr * 0.55) + (rnd() - 0.5) * 3.0;
            puffs.push([px, y + (rnd() - 0.5) * 2.0, rr, 4.0 * (1.0 - u * u * 0.85).sqrt(), 1.0]);
        }
        y -= 4.4;
    }
    // the overshooting top, bulging out of the anvil
    puffs.extend([[141.0, 8.5, 9.0, 2.0, 0.0], [133.0, 9.5, 6.0, 1.0, 0.0], [149.0, 10.0, 6.0, 1.0, 0.0]]);
    // the flanking line: younger towers stepping down to the west
    for [cx, top, w] in [[104.0, 27.0, 9.0], [89.0, 34.0, 6.5], [77.0, 40.5, 3.8]] {
        let mut y = BASE - w * 0.4;
        while y > top + w * 0.6 {
            let px = cx + (rnd() - 0.5) * w * 0.6;
            puffs.push([px, y, w * (0.75 + rnd() * 0.2), 1.0, 1.0]);
            y -= w * 0.75;
        }
        puffs.extend([
            [cx - w * 0.35, top + w * 0.75, w * 0.62, 1.0, 1.0],
            [cx + w * 0.3, top + w * 0.6, w * 0.7, 1.5, 1.0],
            [cx, top + w * 0.45, w * 0.55, 2.0, 1.0],
        ]);
    }
    // one continuous low base under the line, so the towers stand on something
    let mut x = 66.0;
    while x < 126.0 {
        let py = BASE - 1.8 - rnd() * 1.2;
        let r = 3.0 + rnd() * 1.6 + 1.6 * smooth(70.0, 110.0, x);
        puffs.push([x, py, r, 0.4, 1.0]);
        x += 3.2 + rnd() * 1.6;
    }

    let mut sh = vec![0.0f32; SXP * SYP];
    let mut anv = vec![0.0f32; SXP * SYP]; // how much of the height is anvil
    let mut lip = vec![0.0f32; SXP * SYP]; // the anvil's sunlit top edge
    for r in 0..SYP {
        for i in 0..SXP {
            let (x, y) = ((i as f64 - MX as f64 + 0.5) / S, yc(r));
            // the towers, cut flat along the base
            let mut tower: f64 = 0.0;
            let cut = smooth(BASE + 1.0, BASE - 1.5, y + 1.2 * (noise(x * 0.15, 3.3) - 0.5));
            for &[px, py, pr, pz, flat] in &puffs {
                let (dx, dy) = ((x - px) / pr, (y - py) / pr);
                if dx * dx + dy * dy >= 1.0 {
                    continue;
                }
                let v = (pr * 0.6 + pz) * (1.0 - dx * dx - dy * dy).sqrt() * if flat > 0.0 { cut } else { 1.0 };
                tower = tower.max(v);
            }
            // the anvil: flat on top, a lens in section, combed by the wind
            let fib = fbm(x * 0.035, y * 0.2, 3) - 0.5;
            let top = anvil_top(x) + 2.2 * fib;
            let bot = anvil_bot(x) - 1.5 * fib - 3.0 * (fbm(x * 0.2, 7.0, 2) - 0.5);
            let (mid, half) = ((top + bot) / 2.0, ((bot - top) / 2.0).max(0.5));
            let mut anvil = dome(0.0, (y - mid) / half, (half * 0.9).min(5.0)) * smooth(58.0, 72.0, x + 6.0 * fib);
            let on_top = if anvil > 0.0 { smooth(top + 3.2, top + 0.6, y) } else { 0.0 };
            // pouches of mammatus hanging under its western half
            if x > 66.0 && x < 128.0 && y > bot - 2.0 && y < bot + 5.0 {
                let row = ((x - 66.0) / 5.2).floor();
                let off = row * 5.2 + 66.0 + 2.6 + (hash(row, 3.0) - 0.5) * 1.2;
                let py = bot + 0.6 + hash(row, 4.0) * 1.2;
                anvil = anvil.max(dome((x - off) / 2.7, (y - py) / 2.3, 2.2) * smooth(64.0, 76.0, x) * smooth(130.0, 116.0, x));
            }
            let mut h = tower.max(anvil);
            let a = if anvil > tower { smooth(0.0, 2.0, anvil - tower) } else { 0.0 };
            // small billows on the towers; long streaks on the anvil, combed
            // at a slight slant so they do not line up with the rows
            let billow = fbm(x * 0.22, y * 0.24, 3) - 0.5;
            let sy = y * 0.18 + 0.9 * (noise(x * 0.04, 5.5) - 0.5) + x * 0.012;
            let streaky = fbm(x * 0.03 + 0.6 * noise(x * 0.08, y * 0.1), sy, 3) - 0.5;
            h += (h / 2.5).min(1.0) * mix(3.0 * billow, 2.2 * streaky, a);
            sh[r * SXP + i] = h.max(0.0) as f32;
            anv[r * SXP + i] = a as f32;
            lip[r * SXP + i] = (on_top * a) as f32;
        }
    }

    // --- static colour of every pixel: sky, storm, land ---------------------
    let n = W * H;
    let mut base = vec![[0.0; 3]; n];
    let mut mat = vec![Mat::Sky; n];
    let mut cloud = vec![0.0f32; n];
    let mut dark = vec![0.0f32; n];
    let hb = ground(47.0);
    let pump_y = hb - 15.0;
    for py in 0..H {
        let y = yc(py);
        for px in 0..W {
            let k = py * W + px;
            let x = xc(px) - 0.5;
            let xi = x.round();
            if y < ground(x) {
                // sky: indigo overhead, dusky rose lower down, amber where the sun went
                let v = y / HZ;
                let g = glow_sun(x, y);
                let east = smooth(60.0, 190.0, x); // the storm side is cooler and darker
                let v2 = v.powf(2.2);
                let veil = 0.9 + 0.2 * fbm(x * 0.05, y * 0.09, 3);
                let mut cr = (0.045 + 0.2 * v2 * (1.0 - 0.6 * east)) * veil + g;
                let mut cg = (0.045 + 0.09 * v2 * (1.0 - 0.5 * east)) * veil + 0.56 * g;
                let mut cb = (0.16 + 0.13 * v2 * (1.0 - 0.35 * east)) * veil + 0.22 * g;
                // the last light along the horizon, gone where the storm stands
                let be = smooth(45.0, 118.0, x);
                let band = (-(HZ + 2.0 - y) / 11.0).exp() * (1.0 - 0.85 * be) * (1.0 - 0.6 * g);
                cr += band * mix(0.5, 0.34, be);
                cg += band * mix(0.26, 0.21, be);
                cb += band * mix(0.26, 0.3, be);
                // under the storm the air itself goes dark slate, with only a
                // thread of light left along the ground behind the rain
                let under = smooth(98.0, 128.0, x + 8.0 * (noise(y * 0.15, 4.4) - 0.5)) * smooth(BASE - 14.0, BASE - 1.0, y);
                // black under the base, opening to a dim violet glow at the
                // ground: the clear sky far beyond the storm, for the rain to
                // hang against
                let slot = smooth(BASE + 1.5, HZ - 6.0, y).sqrt() * (0.85 + 0.3 * noise(x * 0.06, 6.6));
                cr = mix(cr, mix(0.035, 0.38, slot) * veil, under * 0.92);
                cg = mix(cg, mix(0.032, 0.3, slot) * veil, under * 0.92);
                cb = mix(cb, mix(0.075, 0.56, slot) * veil, under * 0.92);
                // the storm
                if py < SYP - 1 {
                    let j = py * SXP + px + MX;
                    let h = f64::from(sh[j]);
                    let c = smooth(0.15, 1.6, h);
                    if c > 0.0 {
                        let up = if py > 0 { f64::from(sh[j - SXP]) } else { 0.0 };
                        let dn = f64::from(sh[j + SXP]);
                        // the slopes per cell of ascii.rest's, the height field being in pixels
                        let gx = f64::from(sh[j + 1] - sh[j - 1]) / 2.0 * S;
                        let gy = (dn - up) / 2.0 * S;
                        let nl = (gx * gx + gy * gy + 1.0).sqrt();
                        let (nx, ny, nz) = (-gx / nl, -gy / nl, 1.0 / nl);
                        let an = f64::from(anv[j]);
                        let lam = clamp(nx * LIGHT.0 + ny * LIGHT.1 + nz * LIGHT.2).powf(1.5);
                        // earth's shadow climbing the tower: the tops still
                        // lit, the lower bulk already in dusk
                        let vis = 1.0 - 0.4 * smooth(22.0, BASE - 4.0, y + 5.0 * (noise(x * 0.12, 2.2) - 0.5));
                        let a = smooth(8.0, BASE, y);
                        let near = (-((x - SUN.0) / 110.0).powi(2)).exp();
                        let flank = smooth(122.0, 108.0, x) * (1.0 - an); // the younger towers to the west
                                                                          // sunlight: cream-gold up high, peach lower down
                        let (s_r, s_g, s_b) = (1.0, mix(0.84, 0.58, a), mix(0.58, 0.45, a));
                        // the tower's own bulk shades its eastern side, along a ragged edge
                        let ej = 8.0 * (fbm(y * 0.14 + x * 0.02, 21.3, 3) - 0.5);
                        let lee = 1.0
                            - 0.65
                                * smooth(tower_c(y) - 14.0 + ej, tower_c(y) + tower_hw(y) + 16.0 + ej, x)
                                * (1.0 - an)
                                * smooth(anvil_bot(x) - 3.0, anvil_bot(x) + 9.0, y + 5.0 * (noise(x * 0.15, 21.0) - 0.5))
                                * if y < BASE { 1.0 } else { 0.0 };
                        let sun = lam * vis * lee * (0.95 + 0.4 * near) * (1.0 - 0.3 * flank) * (1.0 - 0.2 * an);
                        // skylight from above, violet; afterglow bounced up from the west
                        let sky = 0.55 + 0.45 * (-ny).max(0.0);
                        let bounce = ny.max(0.0) * (0.2 + 0.45 * near) * (1.0 - 0.6 * flank) * (1.0 - 0.5 * an);
                        let occ = 1.0 - 0.35 * smooth(4.0, 22.0, h) * (1.0 - lam);
                        let mut kr = (0.27 * sky + s_r * sun + 0.36 * bounce) * occ;
                        let mut kg = (0.2 * sky + s_g * sun + 0.16 * bounce) * occ;
                        let mut kb = (0.45 * sky + s_b * sun + 0.16 * bounce) * occ;
                        // the anvil's underside sinks into mauve shadow; its
                        // top edge catches the last direct light, cream toward
                        // the west
                        let below = clamp(ny * 2.2) * an;
                        kr *= 1.0 - 0.42 * below;
                        kg *= 1.0 - 0.46 * below;
                        kb *= 1.0 - 0.3 * below;
                        let rim = f64::from(lip[j]) * smooth(196.0, 120.0, x) * (0.6 + 0.4 * lam);
                        (kr, kg, kb) = (mix(kr, 0.98, rim * 0.75), mix(kg, 0.86, rim * 0.75), mix(kb, 0.64, rim * 0.75));
                        // the flankers' lower halves turn violet-grey
                        let low = clamp(ny * 1.6 + 0.2) * flank;
                        (kr, kg, kb) = (mix(kr, 0.14, low * 0.65), mix(kg, 0.11, low * 0.65), mix(kb, 0.24, low * 0.65));
                        // the base: dark slate where the rain hangs
                        let bs = smooth(BASE - 4.5, BASE - 0.5, y) * (1.0 - an) * (1.0 - 0.3 * near);
                        (kr, kg, kb) = (mix(kr, 0.03, bs * 0.9), mix(kg, 0.026, bs * 0.9), mix(kb, 0.06, bs * 0.9));
                        // Solid colour shows a cloud's middle tones brighter than
                        // ascii.rest's dots did, so they are deepened, and the
                        // lit flank stands out of the dusk as it does there.
                        let (kr, kg, kb) = (kr.max(0.0).powf(DEEPEN), kg.max(0.0).powf(DEEPEN), kb.max(0.0).powf(DEEPEN));
                        (cr, cg, cb) = (mix(cr, kr, c), mix(cg, kg, c), mix(cb, kb, c));
                        cloud[k] = c as f32;
                        dark[k] = (bs * c) as f32;
                    }
                }
                // the shelf, hung along the base, a touch lighter on its leading lip
                let on = shelf_on(x);
                if on > 0.0 && y > BASE - 5.0 && y < BASE + 5.0 {
                    let (st, sbt) = (shelf_top(x), shelf_bot(x));
                    let s = smooth(st - 1.0, st + 0.6, y) * smooth(sbt + 0.6, sbt - 0.8, y) * on;
                    if s > 0.0 {
                        let n = fbm(x * 0.15, y * 0.5, 2);
                        // striations along its face, and a faint lip of light underneath
                        let lit = 0.55 + 0.9 * n * (0.7 + 0.6 * noise(x * 0.05, y * 1.3)) + 0.9 * smooth(sbt - 1.6, sbt - 0.2, y);
                        (cr, cg, cb) = (mix(cr, 0.06 * lit, s), mix(cg, 0.056 * lit, s), mix(cb, 0.12 * lit, s));
                        cloud[k] = cloud[k].max(s as f32);
                        dark[k] = dark[k].max(s as f32);
                    }
                }
                base[k] = [cr, cg, cb];
            } else {
                // land: the afterglow caught only in the far rows near the sun
                let g = glow_sun(x, HZ + 1.0) * (-(y - HZ) / 3.2).exp();
                mat[k] = if y < HZ + 4.0 { Mat::Plain } else { Mat::Field };
                base[k] = [0.06 + 0.55 * g, 0.045 + 0.3 * g, 0.08 + 0.1 * g];
            }

            // a shelterbelt of trees on the far horizon, half lost in the rain
            let belt = HZ + 2.0 - (1.5 + 2.2 * fbm(x * 0.3, 2.0, 2)) * smooth(140.0, 150.0, x) * smooth(196.0, 186.0, x);
            let belt2 = HZ + 2.0 - (1.0 + 1.8 * fbm(x * 0.35, 9.0, 2)) * smooth(0.0, 4.0, x) * smooth(18.0, 10.0, x);
            if y >= belt.min(belt2) && y < HZ + 3.0 {
                mat[k] = Mat::Belt;
            }

            // the cottonwood by the house
            let (tx, ty) = ((x + 0.5 - 31.0) / 8.5, (y - (hb - 10.0)) / 7.0);
            let crown = tx * tx + ty * ty < 1.0 + 0.35 * (fbm(x * 0.4, y * 0.4, 2) - 0.5);
            if crown || ((x + 0.5 - 31.5).abs() < 1.0 && y > hb - 6.0 && y < hb + 1.0) {
                mat[k] = Mat::Tree;
            }

            // the farmhouse: two storeys, a gable roof, a chimney, a porch
            if (HX0..=HX1).contains(&xi) && y >= HW0 && y < hb + 1.0 {
                mat[k] = Mat::House;
            }
            if (HW0 - 7.0..HW0).contains(&y) && (x + 0.5 - 47.5).abs() <= 8.6 - (HW0 - y) * 1.15 {
                mat[k] = Mat::Roof;
            }
            if (50.0..=51.0).contains(&xi) && (HW0 - 8.0..HW0 - 3.0).contains(&y) {
                mat[k] = Mat::Roof;
            }
            if (36.0..HX0).contains(&xi) && y >= hb - 4.0 && y < hb - 3.0 {
                mat[k] = Mat::Roof;
            }
            if xi == 36.0 && y >= hb - 3.0 && y < hb {
                mat[k] = Mat::House;
            }
            if (49.0..=51.0).contains(&xi) && (HW0 + 2.0..HW0 + 5.0).contains(&y) {
                mat[k] = Mat::Pane;
            }
            if (43.0..=44.0).contains(&xi) && (HW0 + 2.0..HW0 + 4.0).contains(&y) {
                mat[k] = Mat::Pane;
            }

            // the windpump: a tapering lattice tower
            let wy = y - pump_y;
            if wy > 2.0 && y < hb + 0.5 {
                let half = 0.4 + wy * 0.11;
                let dx = x + 0.5 - PUMP_X;
                if (dx.abs() - half).abs() < 0.55 || (dx.abs() < half && wy.floor() % 4.0 == 0.0) {
                    mat[k] = Mat::Pump;
                }
            }
        }
    }

    // the road: from the yard to the bottom edge, a leading line
    for py in LAND..H {
        let y = yc(py);
        if y < HZ + 3.0 {
            continue;
        }
        let p = (y - HZ) / (SH - HZ);
        for px in 0..W {
            let k = py * W + px;
            let x = xc(px) - 0.5;
            if mat[k] == Mat::Field && (x + 0.5 - road_c(p)).abs() < road_w(p) + 0.6 * (noise(x * 0.3, (y - 0.5) * 0.3) - 0.5) {
                mat[k] = Mat::Road;
            }
        }
    }

    // wheat: stalks in perspective, their columns converging on the
    // farmhouse, tall strokes up close and a fine grain toward the horizon;
    // and how much light the field holds: the sun side, the far rows, not
    // the east or the foreground, which sink into the storm's shadow
    let lr = H - LAND;
    let mut wheat = vec![0.0f32; lr * TW];
    let mut persp = vec![0.0f32; lr * W];
    let mut field_light = vec![0.0f32; lr * W];
    let mut gust = vec![0.0f32; lr * GW];
    for py in LAND..H {
        let (j, y) = (py - LAND, yc(py));
        let (r, p) = (y - 0.5, (y - HZ) / (SH - HZ));
        for u in 0..TW {
            wheat[j * TW + u] = fbm_wrap(u as f64 * 0.55, r * (0.5 - 0.42 * p), 3, TW as f64 * 0.55) as f32;
        }
        for u in 0..GW {
            gust[j * GW + u] = smooth(0.4, 0.7, fbm_wrap(u as f64 * 0.025, r * 0.16, 3, GW as f64 * 0.025)) as f32;
        }
        for px in 0..W {
            let x = xc(px) - 0.5;
            persp[j * W + px] = (((x + 0.5 - VX) * 24.0) / (r + 2.0 - HZ) + 256.0) as f32;
            let sun_w = (-((x - SUN.0) / 100.0).powi(2)).exp();
            let east = smooth(80.0, 130.0, x + 10.0 * p);
            let fore = 1.0 - 0.6 * smooth(SH - 22.0, SH - 2.0, y);
            field_light[j * W + px] = ((0.72 + 0.45 * sun_w) * (1.0 - 0.35 * east) * fore) as f32;
        }
    }

    // thin glowing streaks of altostratus in the clear western sky
    let mut streak = vec![0.0f32; LAND * CW];
    for py in 0..LAND {
        let r = yc(py) - 0.5;
        for u in 0..CW {
            let n = fbm_wrap(u as f64 * 0.02, r * 0.2, 4, CW as f64 * 0.02);
            streak[py * CW + u] = (smooth(0.64, 0.8, n) * smooth(40.0, 46.0, r) * smooth(HZ - 3.0, 52.0, r)) as f32;
        }
    }

    // stars, a pixel big, in the darkest part of the sky
    let mut stars = vec![];
    for py in 0..LAND {
        let y = yc(py);
        for px in 0..W {
            let x = xc(px) - 0.5;
            let k = py * W + px;
            if y < 28.0 && x < 100.0 && hash(px as f64, py as f64 * 5.0 + 3.0) > 0.9945 && cloud[k] < 0.05 {
                stars.push((k, hash(px as f64, py as f64) * PHASE, 1.0 + hash(py as f64, px as f64) * 2.5, 0.85 * (1.0 - y / 28.0) * (1.0 - x / 130.0)));
            }
        }
    }

    // wheat ears right in front of us, bowing with the wind
    let mut ears = vec![];
    let mut x = 2.0f64;
    while x < SW {
        let xi = x.trunc();
        ears.push([
            x,
            8.0 + hash(x * 3.0, 61.0) * 12.0,
            hash(x * 7.0, 62.0) * PHASE,
            hash(x * 5.0, 63.0) * 6.0,
            5.0 + if hash(x, 64.0) > 0.5 { 1.0 } else { 0.0 },
        ]);
        x += 5.0 + hash(xi, 60.0) * 4.0;
    }

    // rain: one envelope of shafts that drifts, and falling streaks inside it
    let shafts = (0..RW).map(|u| smooth(0.47, 0.57, fbm_wrap(u as f64 * 0.093_75, 0.5, 2, 24.0)) as f32).collect();
    let col_phase = (0..W).map(|x| (hash(x as f64, 77.0) * 40.0) as f32).collect();
    let col_speed = (0..W).map(|x| (22.0 + hash(x as f64, 78.0) * 14.0) as f32).collect();
    let rain_top = (0..W).map(|px| (shelf_bot(xc(px) - 0.5) - 1.5) as f32).collect();
    let rain_on = (0..W).map(|px| (smooth(106.0, 118.0, xc(px) - 0.5) * smooth(198.0, 186.0, xc(px) - 0.5)) as f32).collect();

    // lightning: four bolts, each a jagged walk down from the base with a branch or two
    let mut bolts = vec![];
    for i in 0..4 {
        let fi = f64::from(i);
        let mut segs: Vec<[f64; 5]> = vec![];
        fn walk(segs: &mut Vec<[f64; 5]>, i: f64, mut x: f64, mut y: f64, len: f64, lean: f64, depth: f64) {
            let mut s = 0.0;
            while s < len && y < HZ + 2.0 {
                let nx = x + (hash(i * 97.0 + s, depth * 13.0 + 41.0) - 0.5) * 3.2 + lean;
                let ny = y + 0.9 + hash(i * 31.0 + s, depth + 42.0) * 1.3;
                segs.push([x, y, nx, ny, depth]);
                if depth == 0.0 && hash(i * 7.0 + s, 43.0) > 0.84 {
                    walk(segs, i, nx, ny, 3.0 + hash(s, i) * 5.0, (hash(s, i + 9.0) - 0.5) * 2.4, 1.0);
                }
                (x, y) = (nx, ny);
                s += 1.0;
            }
        }
        walk(&mut segs, fi, 122.0 + hash(fi, 40.0) * 34.0, BASE - 4.0, 40.0, (hash(fi, 44.0) - 0.5) * 0.8, 0.0);
        let mut field = vec![99.0f32; n];
        for py in 0..H {
            let y = yc(py);
            if !(25.0..HZ + 4.0).contains(&(y - 0.5).floor()) {
                continue;
            }
            for px in 0..W {
                let x = xc(px) - 0.5;
                if x < 80.0 {
                    continue;
                }
                let mut m: f64 = 99.0;
                for &[ax, ay, bx, by, dep] in &segs {
                    if (x - ax).abs() > 14.0 && (x - bx).abs() > 14.0 {
                        continue;
                    }
                    m = m.min(seg_dist(x + 0.5, y, ax, ay, bx, by) + dep * 0.5);
                }
                field[py * W + px] = m as f32;
            }
        }
        bolts.push((segs[0][0], field));
    }

    Box::new(StormPlains {
        mat,
        base,
        cloud,
        dark,
        wheat,
        persp,
        field_light,
        gust,
        streak,
        stars,
        ears,
        ear: vec![0.0; n],
        touched: vec![],
        shafts,
        col_phase,
        col_speed,
        rain_top,
        rain_on,
        bolts,
        pixels: Pixels::new(&META),
    })
}

/// A fixed schedule of flashes, some carrying a bolt.
fn flash_at(t: f64, bolts: &[(f64, Vec<f32>)]) -> Option<Flash> {
    let n = (t / PERIOD).floor();
    if n > 0.0 && hash(n, 50.0) < 0.25 {
        return None; // some periods stay dark
    }
    let start = if n == 0.0 { -0.08 } else { n * PERIOD + 0.2 + hash(n, 51.0) * 1.6 };
    let l = t - start;
    if !(0.0..=0.9).contains(&l) {
        return None;
    }
    // a stroke and its restrikes, each a sharp rise and a fast decay
    let strikes = [0.0, 0.09 + hash(n, 52.0) * 0.06, 0.32 + hash(n, 53.0) * 0.2];
    let mut i = 0.0;
    for (j, &s) in strikes.iter().enumerate() {
        if l >= s {
            i += (-(l - s) / if j == 0 { 0.09 } else { 0.06 }).exp() * if j == 0 { 1.0 } else { 0.7 - j as f64 * 0.15 };
        }
    }
    let bolt = (n == 0.0 || hash(n, 54.0) > 0.55).then_some(n as usize % 4);
    let cx = bolt.map_or(115.0 + hash(n, 55.0) * 45.0, |b| bolts[b].0);
    let cy = if bolt.is_some() { BASE - 8.0 } else { 14.0 + hash(n, 56.0) * 24.0 };
    let bolt_on = if bolt.is_some() && l < 0.5 { (i * 1.4).min(1.0) } else { 0.0 };
    Some(Flash { i: i.min(1.3), bolt, cx, cy, bolt_on, inside: false })
}

/// Between the big flashes, the storm keeps flickering inside itself.
fn flicker_at(t: f64) -> Option<Flash> {
    let n0 = (t / FLICKER).floor();
    let mut n = n0;
    while n >= n0 - 1.0 && n >= 0.0 {
        let l = t - (n * FLICKER + 0.35 + hash(n, 70.0) * 0.75);
        if (0.0..=0.6).contains(&l) {
            let pulses = [0.0, 0.08 + hash(n, 71.0) * 0.1, 0.26 + hash(n, 72.0) * 0.18];
            let mut i = 0.0;
            for (j, &p) in pulses.iter().enumerate() {
                if l >= p {
                    i += (-(l - p) / 0.07).exp() * [1.0, 0.7, 0.45][j];
                }
            }
            let (cx, cy) = (120.0 + hash(n, 74.0) * 38.0, 15.0 + hash(n, 75.0) * 25.0);
            return Some(Flash { i: i.min(1.0) * (0.25 + 0.2 * hash(n, 73.0)), bolt: None, cx, cy, bolt_on: 0.0, inside: true });
        }
        n -= 1.0;
    }
    None
}

impl StormPlains {
    /// The near ears: each stalk bends from its foot, more at the tip, and the
    /// gusts that roll across the field push them further.
    fn ears(&mut self, t: f64, g_off: f64) {
        for &k in &self.touched {
            self.ear[k] = 0.0;
        }
        self.touched.clear();
        let last = (H - 1 - LAND) * GW;
        for &[ex, eh, ph, foot, hl] in &self.ears {
            let gu = f64::from(self.gust[last + (ex.trunc() as i64 - (g_off * 1.5).floor() as i64).rem_euclid(GW as i64) as usize]);
            // the inflow blows toward the storm, so every ear bows a little east
            let lean = 0.7 + 0.7 * (t * 1.7 + ph).sin() + 1.8 * gu;
            let tall = ((eh + hl) * S).round() as usize;
            let stalk = (eh * S).round() as usize;
            let foot_py = ((SH + foot) * S).round() as i64;
            for ip in 0..tall {
                let py = foot_py - ip as i64;
                if py >= H as i64 {
                    continue;
                }
                let py = py as usize;
                let q = ip as f64 / tall as f64;
                let px = ((ex + lean * q * q + 0.5) * S).floor() as i64;
                if px < 0 || px + 1 >= W as i64 {
                    continue;
                }
                let k = py * W + px as usize;
                if ip < stalk {
                    // the stalk: a thin dark line against the field
                    if self.ear[k] == 0.0 {
                        self.ear[k] = -1.0;
                        self.touched.push(k);
                    }
                    continue;
                }
                // the head: plump, two grains wide in the middle, tapering at
                // both ends, the grains alternating side to side
                let j = ip - stalk;
                let tip = j == tall - stalk - 1;
                let v = if tip {
                    0.55
                } else if j == 0 {
                    0.7
                } else {
                    1.0 - 0.18 * (j & 1) as f32
                };
                let mut put = |k: usize, v: f32| {
                    if v > self.ear[k] {
                        self.ear[k] = v;
                        self.touched.push(k);
                    }
                };
                put(k, v);
                if !tip && j > 0 {
                    put(k + 1, v * 0.82);
                }
                // a whisker of awns past the tip
                if tip && py > 0 {
                    put(k - W + usize::from(lean > 0.5), 0.3);
                }
            }
        }
    }
}

impl Piece for StormPlains {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let mut f = flash_at(t, &self.bolts);
        if let Some(fl) = flicker_at(t) {
            if f.as_ref().is_none_or(|f| f.i < fl.i) {
                f = Some(fl);
            }
        }
        let i_f = f.as_ref().map_or(0.0, |f| f.i);
        let drift = t * 1.2;
        let shaft_off = t * 0.9;
        let g_off = t * 9.0;
        self.ears(t, g_off);
        let hb = ground(47.0);
        let pump_y = hb - 15.0;
        let stalk = [0.2, 0.135, 0.07];

        for py in 0..H {
            let y = yc(py);
            let p = (y - HZ) / (SH - HZ);
            let r = y - 0.5;
            for px in 0..W {
                let k = py * W + px;
                let x = xc(px) - 0.5;
                let m = self.mat[k];
                let [mut cr, mut cg, mut cb] = self.base[k];
                let (mut fade, mut rain) = (1.0, 0.0);
                let c = f64::from(self.cloud[k]);

                if m == Mat::Sky {
                    if c < 0.98 && py < LAND {
                        // the altostratus, lit gold toward the sun
                        let sx = x + drift;
                        let ix = sx.floor();
                        let fx = sx - ix;
                        let ix = ix as usize;
                        let (s0, s1) = (f64::from(self.streak[py * CW + ix % CW]), f64::from(self.streak[py * CW + (ix + 1) % CW]));
                        let s = (s0 + (s1 - s0) * fx) * (1.0 - c) * smooth(105.0, 55.0, x) * 0.75;
                        if s > 0.005 {
                            let sun = (-((x - SUN.0) / 80.0).powi(2)).exp();
                            cr = mix(cr, 0.5 + 0.5 * sun, s);
                            cg = mix(cg, 0.24 + 0.36 * sun, s);
                            cb = mix(cb, 0.34 + 0.06 * sun, s);
                        }
                    }
                    // rain curtains hanging from the shelf to the ground
                    let on = f64::from(self.rain_on[px]);
                    let top = f64::from(self.rain_top[px]);
                    if on > 0.0 && y > top {
                        let u = x + (y - BASE) * 0.42 - shaft_off;
                        let (ui, uf) = (u.floor(), u - u.floor());
                        let ui = (ui as i64).rem_euclid(RW as i64) as usize;
                        let sh = mix(f64::from(self.shafts[ui]), f64::from(self.shafts[(ui + 1) % RW]), uf);
                        let e = sh * on * smooth(top, top + 3.0, y) * (1.0 - c * 0.7);
                        if e > 0.01 {
                            // falling streaks, a pixel wide, slanted with the shafts
                            let col = (((x + (y - BASE) * 0.42 + 0.5) * S).floor() as i64).rem_euclid(W as i64) as usize;
                            let fall = (y * 0.4 - t * f64::from(self.col_speed[col]) * 0.1 + f64::from(self.col_phase[col]) + 1000.0).rem_euclid(4.0);
                            let lit = if fall < 1.4 { 1.0 } else { 0.0 };
                            rain = e;
                            cr = mix(cr, 0.06 + 0.07 * lit, e * 0.88);
                            cg = mix(cg, 0.065 + 0.075 * lit, e * 0.88);
                            cb = mix(cb, 0.12 + 0.1 * lit, e * 0.88);
                        }
                    }
                } else if m == Mat::Field || m == Mat::Plain {
                    let j = py - LAND;
                    let gu = f64::from(self.gust[j * GW + (x.round() as i64 - (g_off * (0.5 + p)).floor() as i64).rem_euclid(GW as i64) as usize]);
                    let g = cr - 0.06; // the gold glint just under the horizon
                    if m == Mat::Field {
                        // stalks lean with the gust and spring back
                        let sway = 0.5 * (t * 2.1 + x * 0.05 + r * 0.17).sin() + 2.2 * gu;
                        let u = f64::from(self.persp[j * W + px]) + sway;
                        let (ui, uf) = (u.floor(), u - u.floor());
                        let ui = (ui as i64).rem_euclid(TW as i64) as usize;
                        let (a0, a1) = (f64::from(self.wheat[j * TW + ui]), f64::from(self.wheat[j * TW + (ui + 1) % TW]));
                        let tex = a0 + (a1 - a0) * uf;
                        let v = clamp(0.55 + (tex - 0.5) * (0.9 + 1.4 * p));
                        let l = (v * 0.7 + 0.4 * gu) * f64::from(self.field_light[j * W + px]);
                        cr = 0.04 + 0.7 * l + 0.9 * g;
                        cg = 0.03 + 0.48 * l + 0.48 * g;
                        cb = 0.035 + 0.18 * l + 0.12 * g;
                        fade = smooth(SH + 10.0, SH - 8.0, y);
                    } else {
                        let v = 0.75 + 0.45 * gu;
                        (cr, cg, cb) = (cr * v, cg * v, cb * v);
                    }
                } else {
                    (cr, cg, cb) = match m {
                        Mat::Road => {
                            let sun = 0.5 + 0.5 * (-((x - SUN.0) / 110.0).powi(2)).exp();
                            let rut = if ((x + 0.5 - road_c(p)).abs() - road_w(p) * 0.45).abs() < 0.4 + p * 0.8 { 0.6 } else { 1.0 };
                            let l = (0.6 + 0.3 * noise(x * 0.5, r * 0.5)) * sun * rut * (1.0 - 0.65 * p);
                            fade = smooth(SH + 10.0, SH - 8.0, y);
                            (0.08 + 0.42 * l, 0.06 + 0.3 * l, 0.07 + 0.26 * l)
                        }
                        Mat::Belt => {
                            // far trees: dark, hazed toward the rain
                            let h = smooth(130.0, 190.0, x);
                            (mix(0.07, 0.1, h), mix(0.05, 0.1, h), mix(0.1, 0.17, h))
                        }
                        Mat::Tree => {
                            let rim = noise(x * 0.6, y * 0.6);
                            (0.06 + 0.06 * rim, 0.04 + 0.03 * rim, 0.07 + 0.04 * rim)
                        }
                        Mat::House | Mat::Pump => {
                            // back-lit by the afterglow: a dark silhouette with a warm western rim
                            let rim = if m == Mat::House && x.round() == HX0 { 0.18 } else { 0.0 };
                            (0.07 + rim, 0.05 + rim * 0.55, 0.09 + rim * 0.3)
                        }
                        Mat::Roof => (0.1, 0.06, 0.1),
                        Mat::Pane => {
                            let g = 0.88 + 0.12 * (t * 2.3 + x.round()).sin() * (t * 3.7).sin();
                            let small = if x.round() < 46.0 { 0.6 } else { 1.0 };
                            (g * small, 0.72 * g * small, 0.3 * g * small)
                        }
                        _ => (cr, cg, cb),
                    };
                }

                let e = f64::from(self.ear[k]);
                if e > 0.0 {
                    // a grain head, warm and brightest on its sunward edge
                    let sun_w = (-((x - SUN.0) / 120.0).powi(2)).exp();
                    let l = e * (0.5 + 0.5 * sun_w);
                    (cr, cg, cb, fade) = (0.95 * l, 0.68 * l, 0.3 * l, 1.0);
                }

                // the windpump's wheel, turning slowly, and its tail vane
                let (wx, wy) = (x + 0.5 - PUMP_X, y - pump_y);
                if wx > -4.0 && wx < 7.0 && wy > -4.0 && wy < 4.0 {
                    let wd = wx.hypot(wy);
                    let vane = wy > -1.0 && wy < 0.6 && wx > 2.0;
                    if (wd < 3.6 && wd > 0.4 && (((wy.atan2(wx) - t * 1.6) * 8.0).cos() > 0.2 || wd < 1.0)) || vane {
                        (cr, cg, cb) = (0.07, 0.05, 0.09);
                    }
                }

                // the lit pane's glow on the yard and the wall around it
                let (lx, ly) = (x + 0.5 - 50.5, y - (HW0 + 3.5));
                if m != Mat::Pane && lx * lx + ly * ly < 200.0 {
                    let lg = (-(lx * lx + ly * ly * 2.0).sqrt() / 3.0).exp() * 0.5;
                    cr += lg;
                    cg += lg * 0.66;
                    cb += lg * 0.25;
                }

                // lightning: the cloud glows from inside; the base and the
                // rain are lit from behind; a full flash reaches the land as well
                if let Some(f) = f.as_ref().filter(|_| i_f > 0.01) {
                    let (dx, dy) = (x - f.cx, (y - f.cy) * 1.3);
                    let dist = dx.hypot(dy);
                    let sky = m == Mat::Sky;
                    let c = if sky { c } else { 0.0 };
                    let inner = if f.inside { (-dist / 14.0).exp() * c * i_f * 2.0 } else { (-dist / 16.0).exp() * (0.3 + 0.7 * c) * i_f * 1.3 };
                    let back = if sky {
                        (f64::from(self.dark[k]) * 0.5 + rain * 0.8) * (-dx.abs() / 34.0).exp() * i_f * if f.inside { 0.5 } else { 1.0 }
                    } else {
                        0.0
                    };
                    let amb = if f.inside { 0.0 } else { i_f * if sky { 0.08 } else { 0.05 } };
                    let lw = inner + back;
                    cr += 0.75 * lw + amb * 0.8;
                    cg += 0.72 * lw + amb * 0.8;
                    cb += lw + amb;
                    if let Some(b) = f.bolt.filter(|_| f.bolt_on > 0.0) {
                        let d = f64::from(self.bolts[b].1[k]);
                        if d < 12.0 && (c < 0.6 || y > BASE - 1.0) {
                            let core = smooth(1.1, 0.35, d) * f.bolt_on;
                            let glow = ((-d / 2.0).exp() * 0.55 + (-d / 7.0).exp() * 0.2) * f.bolt_on;
                            cr += core + glow * 0.75;
                            cg += core + glow * 0.72;
                            cb += core + glow;
                        }
                    }
                }

                if e < 0.0 {
                    // a stalk: one unbroken dark umber line
                    (cr, cg, cb, fade) = (stalk[0], stalk[1], stalk[2], 1.0);
                }
                self.pixels.set(px, py, [mix(GROUND[0], cr, fade) as f32, mix(GROUND[1], cg, fade) as f32, mix(GROUND[2], cb, fade) as f32]);
            }
        }
        // stars, where the cloud leaves the sky clear
        for &(k, ph, sp, a) in &self.stars {
            let v = (a * (0.55 + 0.45 * (t * sp + ph).sin()) * (1.0 - f64::from(self.cloud[k]))) as f32;
            let (px, py) = (k % W, k / W);
            let o = self.pixels.get(px, py);
            self.pixels.set(px, py, [o[0].max(v * 0.9), o[1].max(v * 0.88), o[2].max(v)]);
        }
        self.pixels.draw(grid, env);
    }
}
