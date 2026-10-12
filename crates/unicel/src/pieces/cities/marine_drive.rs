//! marine drive: the queen's necklace seen from malabar hill at night. A
//! string of sodium lamps sweeps round the bay to the towers at nariman
//! point, cars trail light along the road beneath them, and the bay carries
//! their long wavering reflections. Ships ride at anchor on the horizon.
//! Everything along the curve is placed by distance along it, so buildings,
//! lamps and cars shrink together toward the point. After ascii.rest's
//! marine-drive.ts (MIT, @bas3line).
//!
//! The bay, the drive, the city and the sky, and how each is shaded, are
//! ascii.rest's, shaded here per pixel instead of per cell. Where ascii.rest
//! draws each of 200 × 100 square cells as a dot as big as it is bright,
//! this draws 320 × 80 cells of two square pixels each, an upper half block
//! over a back colour ([`crate::pixels`]): 1.6 times the detail each way,
//! solid colour, and gradients dithered in colour rather than in dot size.
//! So the towers' floors are a pixel apart, rooms show as windows further
//! along the drive, and the ships' lights are single pixels. The palette adds
//! the night's darks, glass, tarmac and dim sodium a solid pixel needs where
//! a small dot was enough. In one ink each pixel is octant dots, as many as
//! it is bright.
//!
//! Added here: the cars' lights are mirrored in the bay too, streaming along
//! under the lamps' reflections.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Marine drive's meta.
pub static META: Meta = Meta {
    name: "marine drive",
    slug: "marine-drive",
    category: Category::Cities,
    note: "mumbai's queen's necklace at night, lamps curving round the bay",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: night blues, violets and plums, the sodium ambers to
    // white, the moon's whites, the aviation reds, sea teals and greys. Then
    // the night's darks, glass, tarmac, deep water, dim sodium and lit rooms.
    palette: Some(&[
        "#18213f", "#202c55", "#2a396d", "#374a88", "#4a60a2", "#6880bc", "#2c2140", "#3f2d58", "#56396b", "#6e4a7a", "#8a5f88", "#5a2e3c", "#7b404c",
        "#9e5858", "#55300f", "#7d4515", "#a95c1b", "#d27a24", "#ee9631", "#ffb246", "#ffcd6a", "#ffe39c", "#fff2cf", "#ffffff", "#dce6ff", "#a9bde8",
        "#6a181b", "#b22a28", "#ff4b3e", "#132c38", "#1c3f4f", "#2a5868", "#29283a", "#3c3a4c", "#565266", "#79738a", "#07080f", "#04050a", "#0b0e1a",
        "#0a0e26", "#111836", "#081022", "#0a1430", "#0e1c40", "#120d0d", "#1a1719", "#2a1808", "#3d2410", "#1e1610", "#141a2a", "#4a3d2a", "#6b5a3e",
        "#8c7650", "#b09466", "#c8a878",
    ]),
    ground: Some("#07080f"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const PX: f64 = 1.6;
/// The sea horizon.
const HZ: f64 = 40.0;
/// Nariman point, where the necklace ends.
const TIP: f64 = 176.0;
const MOON: (f64, f64) = (189.0, 13.0);
const MOON_R: f64 = 4.5;
/// Darker seas on the moon's face: (dx, dy, radius).
const MARIA: [(f64, f64, f64); 3] = [(-1.4, -1.2, 1.6), (1.3, 0.8, 1.4), (-0.6, 2.0, 1.1)];
const SODIUM: [f64; 3] = [1.0, 0.56, 0.18];
/// One lamp near us is failing.
const FLICK: usize = 6;
/// The cloud deck: its width in cells and in pixels (it wraps), its height in pixels.
const CW: f64 = 800.0;
const CWP: usize = 1280;
const CHP: usize = 64;
/// Distance along the drive: the table's step in columns, and a car lane's bin.
const STEP: f64 = 0.25;
const BIN: f64 = 0.2;
const U0: f64 = -6.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Sky,
    Water,
    Far,
    Tower,
    Front,
    Road,
    Wall,
    Pole,
    Ship,
}

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn row_y(py: usize) -> f64 {
    (py as f64 + 0.5) / PX
}

/// The scene's x, in ascii.rest's columns (a cell's left edge, as its `x`),
/// at pixel column `px`.
fn col_x(px: usize) -> f64 {
    (px as f64 + 0.5) / PX - 0.5
}

/// The pixel nearest a cell's centre.
fn cell_px(x: f64, r: f64) -> (usize, usize) {
    (((x + 0.5) * PX - 0.5).round() as usize, ((r + 0.5) * PX - 0.5).round() as usize)
}

/// The shoreline: steep and near at the left, flat and far toward the point.
fn shore(x: f64) -> f64 {
    50.0 + 34.0 * (1.0 - x / TIP).max(0.0).powf(2.2)
}

/// How large one unit of distance along the drive looks at column x.
fn scale(x: f64) -> f64 {
    0.5 + 1.9 * (1.0 - x / TIP).max(0.0).powf(1.5)
}

fn sea_top(x: f64) -> f64 {
    if x <= TIP {
        shore(x)
    } else {
        HZ.max(50.0 - (x - TIP) * 1.6)
    }
}

fn wall_top(x: f64) -> f64 {
    shore(x) - scale(x).max(1.0)
}

fn road_top(x: f64) -> f64 {
    wall_top(x) - (1.5 * scale(x)).max(1.0)
}

/// The city's sodium glow in the sky, weaker out over the open sea.
fn glow(x: f64, y: f64) -> f64 {
    let over = 0.5 + 0.5 * smooth(TIP + 24.0, TIP - 16.0, x);
    let h = (HZ - y).max(0.0);
    over * ((-h / 4.0).exp() * 0.5 + (-h / 12.0).exp() * 0.12)
}

/// A block on the front row, by distance along the drive.
struct Building {
    u0: f64,
    u1: f64,
    floors: f64,
    tone: f64,
    crown: bool,
}

/// A room whose light changes: its light's colour, a television or a switch,
/// how fast, and when.
struct Room {
    w: [f64; 3],
    tv: bool,
    rate: f64,
    ph: f64,
}

/// The drive at night: what each pixel is, its colour before anything
/// moves, the lamps' light and reflections, the cloud deck, the cars' lanes,
/// and the rooms and beacons that blink.
pub struct MarineDrive {
    mat: Vec<Mat>,
    rgb: Vec<[f32; 3]>,
    lane: Vec<u8>,
    sky_glow: Vec<f32>,
    sky_lamp: Vec<[f32; 3]>,
    clear: Vec<f32>,
    flick_i: Vec<f32>,
    flick_w: Vec<f32>,
    refl: Vec<[f32; 3]>,
    refl_fl: Vec<f32>,
    cover: Vec<f32>,
    clit: Vec<f32>,
    room: Vec<u16>,
    rooms: Vec<Room>,
    beacons: Vec<(usize, f64)>,
    cars: Vec<(f64, f64, bool)>,
    span: f64,
    lane_a: Vec<f64>,
    lane_b: Vec<f64>,
    bins: Vec<(usize, usize)>,
    col_a: Vec<f64>,
    col_b: Vec<f64>,
    pixels: Pixels,
}

/// No changing room at this pixel.
const NO_ROOM: u16 = u16::MAX;

/// A night on marine drive; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;

    // --- distance along the drive ---
    let nu = ((TIP + 4.0) / STEP).ceil() as usize + 2;
    let mut utab = vec![0.0; nu];
    for i in 1..nu {
        let x = (i as f64 - 0.5) * STEP;
        let sl = (shore(x + 0.05) - shore(x - 0.05)) / 0.1;
        utab[i] = utab[i - 1] + (1.0 + sl * sl).sqrt() * STEP / scale(x);
    }
    let u_of = |x: f64| {
        let f = (x / STEP).clamp(0.0, nu as f64 - 1.001);
        let i = f as usize;
        utab[i] + (utab[i + 1] - utab[i]) * (f - i as f64)
    };
    let x_of_u = |u: f64| {
        let (mut lo, mut hi) = (0.0, TIP);
        for _ in 0..30 {
            let m = (lo + hi) / 2.0;
            if u_of(m) < u {
                lo = m;
            } else {
                hi = m;
            }
        }
        (lo + hi) / 2.0
    };
    let uend = u_of(TIP);

    // --- the lamps ---
    let mut lamps = vec![];
    let mut u = 0.6;
    while u < uend - 0.4 {
        let lx = x_of_u(u);
        let s = scale(lx);
        lamps.push((lx, wall_top(lx) - 2.4 * s, s));
        u += 3.75;
    }

    // --- the front row: art deco blocks along the drive ---
    let mut blds: Vec<Building> = vec![];
    let mut u = -4.0;
    while u < uend {
        let i = blds.len() as f64;
        let w = 7.0 + 7.0 * hash(i, 21.0);
        // art deco blocks of five to eight storeys, and towers at the near, hilly end
        let tall = (7.0 * smooth(26.0, 2.0, u) * (0.6 + 0.4 * hash(i, 26.0))).round();
        blds.push(Building {
            u0: u,
            u1: u + w,
            floors: 5.0 + (hash(i, 22.0) * 4.0).floor() + tall,
            tone: 0.75 + 0.5 * hash(i, 23.0),
            crown: hash(i, 24.0) > 0.45,
        });
        u += w + 0.4 + 1.8 * hash(i, 25.0);
    }
    let col_u: Vec<f64> = (0..W).map(|px| u_of(col_x(px) + 0.5)).collect();
    let bld_at: Vec<Option<usize>> =
        (0..W).map(|px| if col_x(px) + 0.5 > TIP { None } else { blds.iter().rposition(|b| col_u[px] >= b.u0 && col_u[px] < b.u1) }).collect();

    // --- towers behind, a cluster at the point ---
    // (centre, half width, top row, warm light)
    let towers: [(f64, f64, f64, bool); 12] = [
        (24.0, 4.5, 21.0, false),
        (97.0, 2.5, 27.0, true),
        (107.0, 2.0, 22.0, false),
        (118.0, 3.0, 30.0, false),
        (126.0, 2.0, 25.0, true),
        (136.0, 3.0, 23.0, false),
        (142.5, 2.0, 15.0, true),
        (149.0, 3.5, 27.0, false),
        (156.0, 2.5, 19.0, false),
        (162.0, 3.0, 12.0, true),
        (168.5, 2.5, 21.0, false),
        (173.0, 2.0, 28.0, true),
    ];

    // --- the static picture ---
    let mut mat = vec![Mat::Sky; n];
    let mut rgb = vec![[0.0f32; 3]; n];
    let mut lane = vec![0u8; n];
    let mut sky_glow = vec![0.0f32; n];
    let mut room = vec![NO_ROOM; n];
    let mut rooms: Vec<Room> = vec![];
    let mut room_of = std::collections::HashMap::new();
    let mut beacons = vec![];
    // the drive's rows at each pixel column: the road's top and the wall's
    let road_rows: Vec<(f64, f64)> = (0..W).map(|px| (road_top(col_x(px) + 0.5), wall_top(col_x(px) + 0.5))).collect();

    for py in 0..H {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            let xc = x + 0.5;
            if y >= sea_top(xc) {
                mat[k] = Mat::Water;
                continue;
            }
            let mut m = Mat::Sky;
            let (mut cr, mut cg, mut cb) = (0.0, 0.0, 0.0);

            let blk = ((x + 40.0 * fbm(x * 0.02, 5.0, 2)) / 3.0).floor();
            let far_top = HZ - 0.4 - (0.4 + 5.0 * hash(blk, 7.0).powi(3)) * (0.5 + 0.5 * fbm(x * 0.05, 2.0, 2));
            if xc < TIP + 7.0 && y >= far_top {
                // the far city: low blocks in the haze, pricked with tiny lights
                m = Mat::Far;
                let g = glow(xc, HZ - 2.0);
                (cr, cg, cb) = (0.03 + 0.05 * g, 0.04 + 0.03 * g, 0.08 + 0.02 * g);
                let (fx, fy) = (px as f64, py as f64);
                if hash(fx * 3.0 + 1.0, fy * 7.0 + 2.0) < 0.014 {
                    let v = 0.25 + 0.45 * hash(fx, fy * 5.0 + 9.0);
                    let cool = hash(fx * 5.0, fy) < 0.3;
                    cr += v * if cool { 0.8 } else { 1.0 };
                    cg += v * if cool { 0.85 } else { 0.7 };
                    cb += v * if cool { 1.0 } else { 0.4 };
                }
            }

            for (ti, &(tx, hw, top, warm)) in towers.iter().enumerate() {
                let dx = xc - tx;
                if dx.abs() > hw || y < top {
                    continue;
                }
                m = Mat::Tower;
                // dark glass, its moonward edge catching a little light
                (cr, cg, cb) = (0.015, 0.018, 0.035);
                if dx > hw - 1.0 {
                    (cr, cg, cb) = (cr + 0.04, cg + 0.055, cb + 0.1);
                }
                let g = glow(xc, y) * 0.2;
                (cr, cg, cb) = (cr + g * 0.42, cg + g * 0.21, cb + g * 0.1);
                // floors a pixel apart, counted from the roof's pixel row
                let fr = py as i64 - (top * PX - 0.5).ceil() as i64;
                let ti = ti as f64;
                if fr == 0 {
                    (cr, cg, cb) = (cr + 0.08, cg + 0.1, cb + 0.16); // the rooftop's edge
                } else if fr > 0 && fr % 2 == 1 && dx.abs() < hw - 0.5 {
                    let seg = ((xc - tx + hw) / 3.0).floor();
                    let h = hash(ti * 17.0 + seg, py as f64 * 13.0 + 3.0);
                    if h < 0.5 {
                        let v = 0.4 + 0.3 * hash(ti * 7.0 + seg, py as f64);
                        let cool = if warm { h < 0.1 } else { h > 0.1 };
                        cr += v * if cool { 0.72 } else { 1.0 };
                        cg += v * if cool { 0.84 } else { 0.72 };
                        cb += v * if cool { 1.0 } else { 0.42 };
                    }
                }
                if fr == 0 && top < 20.0 && px == (tx * PX - 0.5).round() as usize {
                    beacons.push((k - W, ti * 1.7));
                }
            }

            if xc <= TIP {
                let s = scale(xc);
                let (rt, wt) = road_rows[px];
                if let Some(bi) = bld_at[px] {
                    if y < rt {
                        let b = &blds[bi];
                        let (uu, bw) = (col_u[px] - b.u0, b.u1 - b.u0);
                        let f = (rt - y) / s; // height above the road, in storeys of 1.7
                        let mut hgt = b.floors * 1.7 + 1.1;
                        if b.crown && uu > bw * 0.32 && uu < bw * 0.68 {
                            hgt += 1.6;
                        }
                        if f < hgt {
                            m = Mat::Front;
                            let edge = uu < 0.5 || uu > bw - 0.5;
                            let t0 = b.tone * if edge { 1.35 } else { 1.0 };
                            // pale art deco plaster, warm where the street lamps reach it
                            let up = (-f / 2.0).exp() * 0.16;
                            (cr, cg, cb) = (0.03 * t0 + up * SODIUM[0], 0.035 * t0 + up * SODIUM[1], 0.06 * t0 + up * SODIUM[2]);
                            let top_band = b.floors * 1.7 + 0.6;
                            if f > hgt - 0.6 / s || (f > top_band && f < top_band + 0.5 / s) {
                                (cr, cg, cb) = (cr + 0.1, cg + 0.13, cb + 0.22);
                                // cornice, moonlit
                            }
                            if uu > bw - 0.7 / s - 0.2 {
                                (cr, cg, cb) = (cr + 0.04, cg + 0.06, cb + 0.12);
                                // the corner toward the moon
                            }
                            let fi = ((f - 0.9) / 1.7).floor();
                            let ff = (f - 0.9) - fi * 1.7;
                            let col = ((uu - 0.3) / 1.5).floor();
                            let cu = uu - 0.3 - col * 1.5;
                            // rooms show once a storey is three pixels tall
                            let fine = s * 1.7 * PX >= 3.2;
                            let in_win =
                                fi >= 0.0 && fi < b.floors && uu > 0.7 && uu < bw - 0.7 && (!fine || (ff > 0.45 && ff < 1.35 && cu > 0.35 && cu < 1.15));
                            let bf = bi as f64;
                            if in_win && !fine {
                                // too far to count rooms: each storey reads as a
                                // band of light, broken where rooms are dark
                                let v = 0.04 + 0.26 * hash(bf * 13.0 + (uu / 3.2).floor(), fi * 7.0 + 1.0).powi(2);
                                (cr, cg, cb) = (cr + v, cg + v * 0.8, cb + v * 0.55);
                                if hash(px as f64 * 7.0 + 3.0, py as f64 * 11.0) < 0.014 {
                                    (cr, cg, cb) = (cr + 0.4, cg + 0.36, cb + 0.28);
                                }
                            } else if in_win {
                                let h = hash(bf * 97.0 + col, fi * 31.0 + 5.0);
                                if h < 0.3 {
                                    let v = 0.35 + 0.3 * hash(bf * 13.0 + col, fi);
                                    let kind = hash(bf * 7.0 + col, fi * 3.0 + 1.0);
                                    // tungsten, paler than the lamps, or a tube light
                                    let w = if kind < 0.25 { [0.82, 0.9, 1.0] } else { [1.0, 0.84, 0.58] }.map(|c| c * v);
                                    (cr, cg, cb) = (cr + w[0], cg + w[1], cb + w[2]);
                                    let fk = hash(bf * 5.0 + col, fi * 11.0 + 7.0);
                                    if fk < 0.07 {
                                        let id = *room_of.entry((bi, col as i64, fi as i64)).or_insert_with(|| {
                                            rooms.push(Room { w, tv: fk < 0.025, rate: 0.3 + fk * 9.0, ph: fk * 400.0 });
                                            rooms.len() - 1
                                        });
                                        room[k] = id as u16;
                                    }
                                } else {
                                    (cr, cg, cb) = (cr * 0.7, cg * 0.7, cb * 0.8);
                                }
                            }
                        }
                    }
                }
                if y >= rt && y < wt {
                    m = Mat::Road;
                    (cr, cg, cb) = (0.07, 0.05, 0.05);
                    let lf = (y - rt) / (wt - rt);
                    // two lanes once the road is two pixels wide
                    lane[k] = if (wt - rt) * PX < 2.0 {
                        3
                    } else if lf < 0.5 {
                        1
                    } else {
                        2
                    };
                } else if y >= wt {
                    m = Mat::Wall; // the sea wall and the promenade along it
                    (cr, cg, cb) = (0.1, 0.09, 0.1);
                }
                for &(lx, ly, ls) in &lamps {
                    if ls > 0.9 && (xc - lx).abs() < 0.32 && y > ly && y < wt {
                        m = Mat::Pole;
                        (cr, cg, cb) = (0.09, 0.08, 0.09);
                    }
                }
            }

            if m == Mat::Sky {
                // navy overhead to a sodium haze on the horizon
                let v = y / HZ;
                // brighter over the point, so its towers stand dark against the haze
                let g = glow(xc, y) * (1.0 + 0.6 * smooth(84.0, 98.0, xc) * smooth(186.0, 174.0, xc))
                    + 0.2 * (-(HZ - y).max(0.0) / 14.0).exp() * smooth(84.0, 104.0, xc) * smooth(188.0, 172.0, xc);
                let veil = 0.88 + 0.24 * fbm(x * 0.05, (y - 0.5) * 0.08, 3);
                // the last of the blue hour, deepening overhead
                let vv = v.powf(1.4);
                cr = (0.03 + 0.07 * vv) * veil + g * 0.42;
                cg = (0.05 + 0.12 * vv) * veil + g * 0.22;
                cb = (0.15 + 0.24 * vv) * veil + g * 0.1;
                // a pale sea haze on the open horizon, for the ships to sit against
                let hzn = (-(HZ - y).max(0.0) / 3.5).exp() * smooth(172.0, 186.0, xc) * 0.16;
                (cr, cg, cb) = (cr + hzn * 0.6, cg + hzn * 0.7, cb + hzn * 0.95);
                // a hazy moon over the open sea
                let (dmx, dmy) = (xc - MOON.0, y - MOON.1);
                let dm = dmx.hypot(dmy);
                let halo = (-dm / 22.0).exp() * 0.13 + (-dm / 6.0).exp() * 0.26;
                (cr, cg, cb) = (cr + halo * 0.9, cg + halo * 0.86, cb + halo * 0.8);
                // a darker ring just off the limb, so the disc's edge pops
                let ring = 0.03 * smooth(4.6, 5.6, dm) * smooth(8.5, 6.5, dm);
                (cr, cg, cb) = (cr - ring, cg - ring, cb - ring * 0.8);
                if dm < MOON_R {
                    let mut face = 0.9 + 0.1 * fbm(x * 0.6, (y - 0.5) * 0.6, 2);
                    for (mx, my, mr) in MARIA {
                        let d2 = ((dmx - mx).powi(2) + (dmy - my).powi(2)) / (mr * mr);
                        if d2 < 1.0 {
                            face -= 0.14 * (1.0 - d2);
                        }
                    }
                    let a = smooth(MOON_R, MOON_R - 0.8, dm);
                    (cr, cg, cb) = (mix(cr, face, a), mix(cg, face * 0.95, a), mix(cb, face * 0.86, a));
                }
                sky_glow[k] = g as f32;
            }
            mat[k] = m;
            rgb[k] = [cr as f32, cg as f32, cb as f32];
        }
    }

    // ships riding at anchor beyond the point: a long freighter with its
    // bridge aft, and a smaller boat further out, drawn on ascii.rest's cells
    let hull = |x: i64, r: i64| {
        (r >= 38 && (182..=192).contains(&x))
            || (r == 37 && (183..=192).contains(&x))
            || ((35..=36).contains(&r) && (189..=191).contains(&x))
            || ((35..=36).contains(&r) && x == 184)
            || (r >= 38 && (195..=199).contains(&x))
            || (r == 37 && (196..=197).contains(&x))
    };
    for py in (34.0 * PX) as usize..(40.0 * PX).ceil() as usize {
        for px in 0..W {
            let (x, r) = ((col_x(px) + 0.5).floor() as i64, row_y(py).floor() as i64);
            if (34..40).contains(&r) && hull(x, r) {
                let k = py * W + px;
                mat[k] = Mat::Ship;
                rgb[k] = [0.002, 0.002, 0.004];
            }
        }
    }
    // white mastheads, warm deck and cabin lights, a red port light: a pixel each
    let ship_lights: [(f64, f64, [f64; 3]); 9] = [
        (184.0, 35.0, [1.0, 1.0, 1.0]),
        (190.0, 34.0, [1.0, 1.0, 1.0]),
        (185.0, 37.0, [1.0, 0.78, 0.45]),
        (187.0, 37.0, [1.0, 0.8, 0.5]),
        (189.0, 36.0, [1.0, 0.82, 0.5]),
        (191.0, 36.0, [1.0, 0.8, 0.5]),
        (197.0, 36.0, [1.0, 1.0, 1.0]),
        (196.0, 38.0, [1.0, 0.78, 0.45]),
        (199.0, 38.0, [0.95, 0.22, 0.18]),
    ];
    for &(x, r, c) in &ship_lights {
        let (px, py) = cell_px(x, r);
        let k = py * W + px;
        mat[k] = Mat::Ship;
        rgb[k] = c.map(|v| v as f32);
    }

    // --- lamp light: a hot core, a halo, and a wide warm spill ---
    let mut lamp_i = vec![0.0f32; n];
    let mut lamp_w = vec![0.0f32; n]; // the white-hot cores
    let mut flick_i = vec![0.0f32; n];
    let mut flick_w = vec![0.0f32; n];
    let mut refl = vec![[0.0f32; 3]; n];
    let mut refl_fl = vec![0.0f32; n];
    for (j, &(lx, ly, s)) in lamps.iter().enumerate() {
        let (core, halo, spill) = (0.34 + 0.3 * s, 0.4 + 0.4 * s, 1.2 + 1.5 * s);
        let rad = (spill * 3.2).ceil();
        let (into, hot) = if j == FLICK { (&mut flick_i, &mut flick_w) } else { (&mut lamp_i, &mut lamp_w) };
        let (py0, py1) = (((ly - rad) * PX).floor().max(0.0) as usize, (((ly + rad) * PX).ceil() as usize).min(H));
        let (px0, px1) = (((lx - rad) * PX).floor().max(0.0) as usize, (((lx + rad) * PX).ceil() as usize).min(W));
        for py in py0..py1 {
            for px in px0..px1 {
                let k = py * W + px;
                if mat[k] == Mat::Water {
                    continue;
                }
                let d = (col_x(px) + 0.5 - lx).hypot(row_y(py) - ly);
                let c = (-(d / core).powi(2)).exp() * 2.2;
                into[k] += (c * 0.7 + (-d / halo).exp() * 0.24 + (-d / spill).exp() * 0.02) as f32;
                hot[k] += (c * 0.3) as f32;
            }
        }
        // its reflection: a long column broken by the swell, reaching toward us
        let sy = shore(lx);
        let (wj, len) = (0.4 + 0.7 * s, 2.0 + 8.0 * s);
        let (px0, px1) = (((lx - 3.0 * wj - 1.0) * PX).floor().max(0.0) as usize, (((lx + 3.0 * wj + 1.0) * PX).ceil() as usize).min(W));
        for py in (sy * PX).floor() as usize..H {
            let dy = row_y(py) - sy;
            if dy < 0.0 {
                continue;
            }
            let a = (-dy / (len * 1.7)).exp() * (0.3 + 0.15 * s) * smooth(-0.5, 1.5, dy);
            if a < 0.004 {
                break;
            }
            for px in px0..px1 {
                let k = py * W + px;
                if mat[k] != Mat::Water {
                    continue;
                }
                let g = a * (-((col_x(px) + 0.5 - lx) / wj).powi(2)).exp();
                if j == FLICK {
                    refl_fl[k] += g as f32;
                } else {
                    for c in 0..3 {
                        refl[k][c] += (g * SODIUM[c]) as f32;
                    }
                }
            }
        }
    }
    // the lamps' light on everything but the water; the sky takes it each
    // frame, over the cloud
    let mut sky_lamp = vec![[0.0f32; 3]; n];
    for k in 0..n {
        let (li, lw) = (f64::from(lamp_i[k]), f64::from(lamp_w[k]));
        let add = [li * SODIUM[0] + lw, li * SODIUM[1] + lw * 0.95, li * SODIUM[2] + lw * 0.8].map(|v| v as f32);
        match mat[k] {
            Mat::Water => {}
            Mat::Sky => sky_lamp[k] = add,
            _ => (0..3).for_each(|c| rgb[k][c] += add[c]),
        }
    }

    // the ships' lights stretch down the water too
    for &(x, _, c) in &ship_lights {
        for py in (HZ * PX) as usize..((HZ + 14.0) * PX) as usize {
            let a = (-(row_y(py) - 0.5 - HZ) / 5.0).exp() * 0.35;
            for px in ((x - 1.0) * PX).floor() as usize..(((x + 2.0) * PX).ceil() as usize).min(W) {
                let k = py * W + px;
                let dx = col_x(px) - x;
                if mat[k] != Mat::Water || dx.abs() > 1.5 {
                    continue;
                }
                let g = a * (-dx * dx * 2.5).exp();
                for ch in 0..3 {
                    refl[k][ch] += (g * c[ch]) as f32;
                }
            }
        }
    }

    // The mirror: each water pixel sees the picture above the shore, flipped
    // and stretched toward us, dimmed.
    for py in 0..H {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            if mat[k] != Mat::Water {
                continue;
            }
            let xc = col_x(px) + 0.5;
            let sy = sea_top(xc);
            let d = y - sy;
            let (mut acc, mut cnt) = ([0.0; 3], 0.0);
            // only the lights carry across: the sky, the towers' lit floors, ships
            for o in 0..3 {
                let ym = sy - d * 0.55 - 0.3 - f64::from(o) * 0.7;
                if ym < 0.0 {
                    continue;
                }
                let q = (ym * PX).floor() as usize * W + px;
                match mat[q] {
                    Mat::Water => continue,
                    Mat::Sky => (0..3).for_each(|c| acc[c] += f64::from(rgb[q][c])),
                    Mat::Tower | Mat::Ship => (0..3).for_each(|c| acc[c] += (f64::from(rgb[q][c]) - 0.25).max(0.0) * 0.6),
                    _ => {}
                }
                cnt += 1.0;
            }
            let a = 0.32 * (-d / 30.0).exp();
            if cnt > 0.0 {
                for c in 0..3 {
                    refl[k][c] += (acc[c] / cnt * a) as f32;
                }
            }
            // and a soft warm sheen off the whole lit shore
            if xc <= TIP + 4.0 {
                let sh = 0.025 * (-d / 5.0).exp() * smooth(TIP + 4.0, TIP - 6.0, xc);
                for c in 0..3 {
                    refl[k][c] += (sh * SODIUM[c]) as f32;
                }
            }
        }
    }

    // --- clouds: low stratus lit from beneath by the city, wrapping ---
    let density = |x: f64, y: f64| {
        let q = fbm_wrap(x * 0.01, y * 0.05, 3, 8.0);
        let d = fbm_wrap(x * 0.025 + q * 2.6, y * 0.07 + q * 1.3, 5, 20.0);
        let yb = y + 14.0 * (fbm_wrap(x * 0.005 + 3.0, 7.0, 2, 4.0) - 0.5); // the bank's edge rises and falls
        d + 0.05 * smooth(2.0, 14.0, yb) - 0.1 * smooth(6.0, 0.0, y) - 0.1 * smooth(22.0, HZ - 4.0, yb)
    };
    let mut cover = vec![0.0f32; CWP * CHP];
    let mut clit = vec![0.0f32; CWP * CHP];
    for py in 0..CHP {
        let y = row_y(py);
        for i in 0..CWP {
            let x = i as f64 / PX;
            let d = density(x, y);
            cover[py * CWP + i] = (smooth(0.47, 0.62, d) * 0.9) as f32;
            clit[py * CWP + i] = clamp(0.5 + (d - density((x - 1.0).rem_euclid(CW), y + 2.5)) * 9.0 - (d - 0.6) * 1.4) as f32;
        }
    }

    // the sky behind the towers at the point is kept clear of low cloud, so
    // the dark glass reads against the city's haze
    let clear = (0..n)
        .map(|k| {
            let (x, y) = (col_x(k % W) + 0.5, row_y(k / W));
            (1.0 - 0.9 * smooth(84.0, 96.0, x) * smooth(186.0, 176.0, x) * smooth(20.0, 28.0, y)) as f32
        })
        .collect();

    // --- cars ---
    let nb = ((uend + 12.0) / BIN).ceil() as usize;
    let cars = (0..26).map(|i| (hash(f64::from(i), 51.0) * (uend + 12.0), 4.2 + 1.6 * hash(f64::from(i), 52.0), i & 1 == 1)).collect();
    // the lane bins each pixel column spans
    let bins = (0..W)
        .map(|px| {
            let lo = (((u_of(px as f64 / PX) - U0) / BIN).floor().max(0.0) as usize).min(nb - 1);
            let hi = (((u_of((px + 1) as f64 / PX) - U0) / BIN).floor() as usize).max(lo).min(nb - 1);
            (lo, hi)
        })
        .collect();

    Box::new(MarineDrive {
        mat,
        rgb,
        lane,
        sky_glow,
        sky_lamp,
        clear,
        flick_i,
        flick_w,
        refl,
        refl_fl,
        cover,
        clit,
        room,
        rooms,
        beacons,
        cars,
        span: uend + 12.0,
        lane_a: vec![0.0; nb],
        lane_b: vec![0.0; nb],
        bins,
        col_a: vec![0.0; W],
        col_b: vec![0.0; W],
        pixels: Pixels::new(&META),
    })
}

impl Piece for MarineDrive {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let drift = t * 1.3 * PX;
        // cars: tail lights heading out to the point, headlights coming home
        self.lane_a.fill(0.0);
        self.lane_b.fill(0.0);
        let nb = self.lane_a.len() as i64;
        for &(p0, v, dir) in &self.cars {
            let p = (p0 + if dir { -v } else { v } * t).rem_euclid(self.span);
            let into = if dir { &mut self.lane_b } else { &mut self.lane_a };
            let head = (p / BIN).floor() as i64;
            for i in 0..26 {
                let b = if dir { head + i } else { head - i };
                if b < 0 || b >= nb {
                    continue;
                }
                let e = (-(i as f64) / 7.0).exp() * if i < 2 { 1.3 } else { 0.85 };
                into[b as usize] = into[b as usize].max(e);
            }
        }
        for px in 0..W {
            let (lo, hi) = self.bins[px];
            let a = self.lane_a[lo..=hi].iter().fold(0.0f64, |m, &v| m.max(v));
            let b = self.lane_b[lo..=hi].iter().fold(0.0f64, |m, &v| m.max(v));
            (self.col_a[px], self.col_b[px]) = (a, b);
        }
        // the failing lamp
        let lamp_on = if (t * 0.7).sin() > 0.45 {
            if hash((t * 9.0).floor(), 77.0) > 0.45 {
                1.0
            } else {
                0.15
            }
        } else {
            1.0
        };
        let ground = [0.027, 0.031, 0.059];

        for py in 0..H {
            let y = row_y(py);
            for px in 0..W {
                let k = py * W + px;
                let x = col_x(px);
                let m = self.mat[k];
                let [mut cr, mut cg, mut cb] = self.rgb[k].map(f64::from);
                let mut fade = 1.0;

                match m {
                    Mat::Sky => {
                        let sx = px as f64 + drift;
                        let ix = sx.floor();
                        let fx = sx - ix;
                        let mut star = false;
                        if py < CHP {
                            let i0 = (ix as usize) % CWP;
                            let (i0, i1) = (py * CWP + i0, py * CWP + (i0 + 1) % CWP);
                            let c = (f64::from(self.cover[i0]) + f64::from(self.cover[i1] - self.cover[i0]) * fx) * f64::from(self.clear[k]);
                            if c > 0.01 {
                                let l = f64::from(self.clit[i0]) + f64::from(self.clit[i1] - self.clit[i0]) * fx;
                                let g = f64::from(self.sky_glow[k]);
                                // grey-violet, warmed underneath by the city
                                let dm = (x + 0.5 - MOON.0).hypot(y - MOON.1);
                                let near = (-dm / 16.0).exp();
                                let b = clamp(0.1 + l * (0.42 + 0.9 * g + 0.5 * near));
                                let ramp = |lo: f64, md: f64, hi: f64| if b < 0.5 { mix(lo, md, b * 2.0) } else { mix(md, hi, b * 2.0 - 1.0) };
                                // low cloud thins into the haze, so the horizon and the ships stay clear
                                let a = (c * 1.1).min(1.0) * smooth(MOON_R, MOON_R + 4.0, dm) * (1.0 - 0.8 * smooth(27.0, 37.0, y));
                                cr = mix(cr, ramp(0.05, 0.22, 0.9), a);
                                cg = mix(cg, ramp(0.06, 0.26, 0.88), a);
                                cb = mix(cb, ramp(0.15, 0.44, 0.92), a);
                            } else {
                                star = true;
                            }
                        }
                        if star && y < HZ - 14.0 && hash(px as f64, py as f64 * 3.0 + 11.0) > 0.9973 {
                            // a star, a pixel big, twinkling
                            let (fx, fy) = (px as f64, py as f64);
                            let tw = 0.3 + 0.2 * (t * (1.3 + hash(fx, fy) * 2.5) + hash(fy, fx) * TAU).sin();
                            (cr, cg, cb) = (cr.max(tw * 0.9), cg.max(tw * 0.9), cb.max(tw));
                        }
                        // the lamps' halos in the air, over the cloud
                        let [lr, lg, lb] = self.sky_lamp[k].map(f64::from);
                        (cr, cg, cb) = (cr + lr, cg + lg, cb + lb);
                    }
                    Mat::Water => {
                        let v = (y - HZ) / (100.0 - HZ);
                        let w = 0.6 * noise(x * 0.06 + t * 0.1, y * 0.5 - t * 0.5) + 0.4 * noise(x * 0.18 - t * 0.2, y - t);
                        let swell = 0.45 + 0.95 * w;
                        // brighter toward the open sea and the moon
                        let open = (0.85 + 0.3 * (x / 200.0)) * swell;
                        (cr, cg, cb) = ((0.05 - 0.01 * v) * open, (0.095 - 0.015 * v) * open, (0.23 - 0.03 * v) * open);
                        // the moon's road on the open water
                        let road = (-((x + 0.5 - MOON.0) / (0.8 + (y - HZ) * 0.12)).powi(2)).exp();
                        if road > 0.002 {
                            let glint = smooth(0.52, 0.8, 0.45 * w + 0.55 * noise(x * 0.3 + t * 0.3, y * 1.4 - t * 1.4)) * road;
                            (cr, cg, cb) = (cr + 0.9 * glint + 0.03 * road, cg + 0.88 * glint + 0.035 * road, cb + 0.8 * glint + 0.05 * road);
                        }
                        // the reflections, pushed sideways by the swell and broken into dashes
                        let wob = (noise(x * 0.05 + 7.0, y * 0.32 - t * 0.7) - 0.5) * (1.4 + 4.5 * v);
                        let sx = (px as f64 + wob * PX).clamp(0.0, W as f64 - 1.001);
                        let i0 = py * W + sx as usize;
                        let fx = sx - sx.floor();
                        let dash = 0.45 + 0.9 * smooth(0.2, 0.55, noise(x * 0.16 + 3.0, y * 0.85 - t * 1.3));
                        let at = |a: f32, b: f32| (f64::from(a) + f64::from(b - a) * fx) * dash;
                        let fl = at(self.refl_fl[i0], self.refl_fl[i0 + 1]) * lamp_on;
                        let mut rr = at(self.refl[i0][0], self.refl[i0 + 1][0]) + fl * SODIUM[0];
                        let mut rg = at(self.refl[i0][1], self.refl[i0 + 1][1]) + fl * SODIUM[1];
                        let mut rb = at(self.refl[i0][2], self.refl[i0 + 1][2]) + fl * SODIUM[2];
                        // the cars' lights, mirrored under the sea wall
                        let sxp = sx.round() as usize;
                        let xs = col_x(sxp) + 0.5;
                        let d = y - sea_top(xs);
                        if xs <= TIP && d < 10.0 {
                            let (a, b) = (self.col_a[sxp], self.col_b[sxp]);
                            let f = (-d / 2.5).exp() * 0.22 * dash;
                            (rr, rg, rb) = (rr + f * (a * 0.8 + b), rg + f * (a * 0.1 + b * 0.9), rb + f * (a * 0.08 + b * 0.72));
                        }
                        // where the warm light lies on the water it replaces the
                        // blue, so the streaks stay gold instead of mixing to pink
                        let kill = 1.0 - 0.9 * clamp((rr - rb) * 3.0);
                        (cr, cg, cb) = (cr * kill + rr, cg * kill + rg, cb * kill + rb);
                        fade = smooth(106.0, 92.0, y);
                    }
                    Mat::Road => {
                        let l = self.lane[k];
                        let a = if l & 1 == 1 { self.col_a[px] } else { 0.0 };
                        let b = if l & 2 == 2 { self.col_b[px] } else { 0.0 };
                        (cr, cg, cb) = (cr + a * 0.8 + b, cg + a * 0.1 + b * 0.9, cb + a * 0.08 + b * 0.72);
                    }
                    _ => {}
                }
                if m != Mat::Water {
                    let (f, fw) = (f64::from(self.flick_i[k]) * lamp_on, f64::from(self.flick_w[k]) * lamp_on);
                    if f > 0.0 {
                        (cr, cg, cb) = (cr + f * SODIUM[0] + fw, cg + f * SODIUM[1] + fw * 0.95, cb + f * SODIUM[2] + fw * 0.8);
                    }
                }
                let id = self.room[k];
                if id != NO_ROOM {
                    // a room whose light changes
                    let r = &self.rooms[id as usize];
                    let [wr, wg, wb] = r.w;
                    if r.tv {
                        // a television: cold light that jumps
                        let f = 0.5 + 0.5 * (t * 11.0 + r.ph).sin() * (t * 4.3 + r.ph * 2.0).sin();
                        (cr, cg, cb) = (cr - wr + f * wr * 0.45, cg - wg + f * wr * 0.6, cb - wb + f * wr);
                    } else if (t * r.rate * 0.5 + r.ph).sin() <= -0.6 {
                        // a light switched off for a while, then on again
                        (cr, cg, cb) = (cr - wr, cg - wg, cb - wb);
                    }
                    (cr, cg, cb) = (cr.max(0.0), cg.max(0.0), cb.max(0.0));
                }
                let c = [mix(ground[0], cr, fade) as f32, mix(ground[1], cg, fade) as f32, mix(ground[2], cb, fade) as f32];
                self.pixels.set(px, py, c);
            }
        }
        // the aviation lights on the tallest towers
        for &(k, ph) in &self.beacons {
            let on = (t * 2.4 + ph).sin() > 0.3;
            self.pixels.set(k % W, k / W, if on { [1.0, 0.2, 0.15] } else { [0.3, 0.06, 0.05] });
        }
        self.pixels.draw(grid, env);
    }
}
