//! bonsai: a tree in a shallow pot through a year. Its leaf pads are shaded
//! domes, lit from the upper left; in autumn they thin leaf by leaf and drop,
//! the branches stand bare, then the pads bud again from the middle out.
//! After ascii.rest's bonsai.ts (MIT, @bas3line).
//!
//! The year, the pads, the trunk and the falling leaves are ascii.rest's,
//! drawn in octants: every dot of a pad is a leaf with its own moment to turn
//! and to bud, so the pads thin and fill a dot at a time, and the trunk
//! tapers smoothly instead of in steps. In colour the leaves go green, then
//! yellow, orange and red through autumn before they drop, the buds come
//! back a pale spring green that deepens into summer, the bark is lit on its
//! left, and the pot is a blue glaze.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// Bonsai's meta.
pub static META: Meta = Meta {
    name: "bonsai",
    slug: "bonsai",
    category: Category::Nature,
    note: "an octant bonsai whose leaves turn gold and red, fall, and bud again",
    cols: 52,
    rows: 19,
    fps: 10,
    charset: Charset::Extended,
    // Leaves: shade, sunlit, spring bud; autumn yellow, orange, red. Bark lit
    // and in shadow; the glaze, its highlight; the soil.
    palette: Some(&["#2e6a30", "#6cb44a", "#c4ee86", "#e8c440", "#e87a2a", "#c83424", "#a0744c", "#5a3c26", "#3c5c80", "#7aa2c6", "#3a2c22"]),
    ground: Some("#0e0f12"),
    ..Meta::DEFAULT
};

const SHADE: u8 = 0;
const SUN: u8 = 1;
const BUD: u8 = 2;
const AUTUMN: [u8; 4] = [1, 3, 4, 5];
const BARK_LIT: u8 = 6;
const BARK: u8 = 7;
const GLAZE: u8 = 8;
const GLOSS: u8 = 9;
const SOIL: u8 = 10;

/// Seconds a year: summer, autumn, bare, spring, summer. A leaf starts to
/// turn in the autumn span and is gone FADE later, having coloured for TINT
/// before; it buds in the spring span and is full GROW later; and a falling
/// leaf takes DROP seconds a row.
const LOOP: f64 = 26.0;
const TURN: (f64, f64) = (3.0, 8.5);
const FADE: f64 = 2.5;
const TINT: f64 = 2.5;
const SPRING: (f64, f64) = (14.0, 18.5);
const GROW: f64 = 2.5;
const DROP: f64 = 0.24;

/// Leaf pads: the middle of the widest row, half-width, the dome's height in
/// rows, and where the branch under it splits into twigs.
type Pad = (f64, f64, f64, f64, (f64, f64));
const PADS: [Pad; 5] = [
    (27.0, 3.0, 7.0, 2.2, (27.0, 2.0)),
    (15.0, 5.0, 7.5, 1.3, (15.0, 6.0)),
    (39.0, 6.0, 7.0, 1.3, (39.0, 7.0)),
    (11.0, 9.0, 6.5, 1.3, (11.0, 10.0)),
    (40.0, 11.0, 6.0, 1.3, (40.0, 12.0)),
];
/// The trunk from the soil up: row, first column, width. It tapers 3, 2, 1
/// and leans right, back left, then right again into the top pad.
const TRUNK: [(f64, f64, f64); 13] = [
    (15.0, 24.0, 3.0),
    (14.0, 24.0, 3.0),
    (13.0, 25.0, 3.0),
    (12.0, 26.0, 2.0),
    (11.0, 26.0, 2.0),
    (10.0, 25.0, 2.0),
    (9.0, 24.0, 2.0),
    (8.0, 24.0, 1.0),
    (7.0, 24.0, 1.0),
    (6.0, 25.0, 1.0),
    (5.0, 26.0, 1.0),
    (4.0, 26.0, 1.0),
    (3.0, 27.0, 1.0),
];
/// The branches, from the trunk out to each pad's fork, in columns and rows:
/// ascii.rest's runs of strokes as polylines.
const BRANCHES: [&[(f64, f64)]; 4] = [
    &[(27.0, 12.8), (39.5, 12.8), (40.5, 12.4)],
    &[(26.0, 11.8), (20.0, 11.8), (19.0, 10.8), (13.0, 10.8), (11.5, 10.4)],
    &[(25.0, 8.8), (30.5, 8.8), (31.5, 7.8), (37.5, 7.8), (39.5, 7.4)],
    &[(24.5, 7.8), (20.0, 7.8), (19.0, 6.8), (17.0, 6.8), (15.5, 6.4)],
];
/// The soil's row and columns, where a falling leaf lands in the pot.
const SOIL_AT: (f64, f64, f64) = (14.0, 7.0, 43.0);

fn hash(a: i64, b: i64) -> f64 {
    let mut h = (a.wrapping_mul(374_761_393).wrapping_add(b.wrapping_mul(668_265_263)) as u32).wrapping_mul(1_274_126_177);
    h = (h ^ (h >> 13)).wrapping_mul(1_103_515_245);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

/// A leaf: its dot, how dense it is lit and in shadow, how sunlit it is,
/// when it turns and buds, and a little hash of its own for the wind.
struct Leaf {
    x: i64,
    y: i64,
    day: f64,
    night: f64,
    lit: f64,
    turn: f64,
    bud: f64,
    h: f64,
}

/// A falling leaf: when it lets go, its path in dots (one point a row), how
/// long it lies in the pot after landing, and its colour.
struct Fall {
    t0: f64,
    path: Vec<(f64, f64)>,
    lie: f64,
    color: u8,
}

/// The tree: its wood and pot drawn once, every leaf, the falling leaves, and the dots.
pub struct Bonsai {
    wood: Vec<(i64, i64, u8)>,
    leaves: Vec<Leaf>,
    falls: Vec<Fall>,
    dots: ColorOctants,
}

/// Every dot along a thick line between two points in cells.
fn stroke(out: &mut Vec<(i64, i64, u8)>, (x0, y0): (f64, f64), (x1, y1): (f64, f64), half: f64, color: u8) {
    let (a, b) = ((x0 * 2.0, y0 * 4.0), (x1 * 2.0, y1 * 4.0));
    let n = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * 2.0).ceil().max(1.0) as i64;
    for s in 0..=n {
        let f = s as f64 / n as f64;
        let (x, y) = (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f);
        for dy in -(half.ceil() as i64)..=half.ceil() as i64 {
            if (dy as f64).abs() <= half {
                out.push((x.floor() as i64, (y + dy as f64).floor() as i64, color));
            }
        }
    }
}

/// A bonsai; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut wood = vec![];
    // The pot: a soil bed, a glazed rim with a highlight, a body that tapers in, two feet.
    for x in 16..88 {
        wood.push((x, 60, SOIL));
        wood.push((x, 61, SOIL));
    }
    for x in 12..92 {
        wood.push((x, 62, GLOSS));
        wood.push((x, 63, GLAZE));
    }
    for y in 64..68 {
        let inset = y - 63;
        for x in 14 + inset..90 - inset {
            wood.push((x, y, GLAZE));
        }
    }
    for y in 68..71 {
        for x in (18..28).chain(76..86) {
            wood.push((x, y, GLAZE));
        }
    }
    // The trunk, a column of dots a row, its middle eased between rows so it
    // leans smoothly; lit on its left.
    for y in 12..64 {
        let r = y as f64 / 4.0;
        let k = TRUNK.iter().position(|&(row, ..)| row <= r.floor()).unwrap_or(TRUNK.len() - 1);
        let (row, c, w) = TRUNK[k];
        let (_, c1, w1) = TRUNK[(k + 1).min(TRUNK.len() - 1)];
        let f = (r - row).clamp(0.0, 1.0);
        let (mid, width) = ((c + w / 2.0) * (1.0 - f) + (c1 + w1 / 2.0) * f, w * (1.0 - f) + w1 * f);
        let (x0, x1) = (((mid - width / 2.0) * 2.0).round() as i64, ((mid + width / 2.0) * 2.0).round() as i64);
        for x in x0..x1.max(x0 + 1) {
            wood.push((x, y, if x < (x0 + x1) / 2 { BARK_LIT } else { BARK }));
        }
    }
    for b in BRANCHES {
        for p in b.windows(2) {
            stroke(&mut wood, p[0], p[1], 0.6, BARK);
        }
    }
    // Under each pad its branch splits into twigs, seen once the leaves are off.
    for &(.., (fx, fy)) in &PADS {
        let fork = (fx + 0.5, fy + 0.6);
        for (dx, dy) in [(-2.0, -1.4), (0.0, -1.5), (2.0, -1.4)] {
            stroke(&mut wood, fork, (fork.0 + dx, fork.1 + dy), 0.0, BARK);
        }
    }

    // Every leaf, a dot of a pad: how dense it is, how lit, and when it turns
    // and buds. Outer leaves turn first; inner ones bud first.
    let mut leaves = vec![];
    for (n, &(px, py, pw, ph, _)) in PADS.iter().enumerate() {
        for y in ((py - ph) * 4.0).floor() as i64..((py + 2.0) * 4.0) as i64 {
            for x in ((px - pw) * 2.0).floor() as i64..=((px + pw + 1.0) * 2.0).ceil() as i64 {
                let (cx, cy) = (x as f64 / 2.0 + 0.25, y as f64 / 4.0 + 0.125);
                let dx = (cx - (px + 0.5)) / pw;
                let dome = cy <= py + 1.0;
                let dy = (cy - (py + 0.5)) / ph;
                // A dome above the widest row; one flat, tucked row below it.
                let rho = if dome { dx.hypot(dy) } else { (dx.abs() / 0.85).max(0.72) };
                if rho > 1.0 {
                    continue;
                }
                let h = hash(y * 97 + n as i64, x);
                let mass = ((1.0 - if dome { dx.hypot(dy * 0.85) } else { rho }) * 2.6).min(1.0) * (0.85 + 0.3 * h);
                let lit = (0.5 - 0.35 * dx - 0.55 * if dome { dy } else { 1.1 }).clamp(0.0, 1.0);
                leaves.push(Leaf {
                    x,
                    y,
                    day: mass * (0.5 + 0.5 * lit),
                    night: mass * (0.5 + 0.5 * (1.0 - lit)),
                    lit,
                    turn: TURN.0 + (TURN.1 - TURN.0) * (0.7 * (1.0 - rho) + 0.3 * h),
                    bud: SPRING.0 + (SPRING.1 - SPRING.0) * (0.75 * rho + 0.25 * h).min(1.0),
                    h,
                });
            }
        }
    }

    // Falling leaves: a few strays in summer, and in autumn some of each
    // pad's underside as it goes. Each drops near where it left, and one that
    // lands in the pot lies there, a stray for a while and an autumn leaf until spring.
    let mut falls = vec![];
    let mut fall = |l: &Leaf, t0: f64, k: i64, until: Option<f64>, color: u8| {
        let (drift, phase) = ((hash(k, 7) - 0.5) * 1.2, hash(k, 9) * 6.3);
        let (c0, r0) = (l.x as f64 / 2.0, l.y as f64 / 4.0);
        let mut path = vec![];
        let mut i = 0.0;
        let mut lands = false;
        while r0 + i + 1.0 < META.rows as f64 {
            let (r, c) = (r0 + i + 1.0, c0 + drift * i * DROP + (i * 0.9 + phase).sin());
            path.push((c * 2.0, r * 4.0));
            if r >= SOIL_AT.0 && (SOIL_AT.1..=SOIL_AT.2).contains(&c) {
                lands = true;
                break;
            }
            i += 1.0;
        }
        let lie = if lands { (until.unwrap_or(t0 + path.len() as f64 * DROP + 3.0) - t0 + LOOP).rem_euclid(LOOP) } else { 0.0 };
        falls.push(Fall { t0, path, lie, color });
    };
    let under: Vec<Vec<&Leaf>> =
        (0..PADS.len()).map(|n| leaves.iter().filter(|l| l.y as f64 / 4.0 > PADS[n].1 + 1.0 && pad_of(l, n) && l.day > 0.15).collect()).collect();
    for (k, &(t0, n)) in [(-1.2, 0), (0.5, 0), (2.0, 1), (3.0, 0), (22.6, 1), (24.4, 3)].iter().enumerate() {
        let from = &under[n];
        if !from.is_empty() {
            fall(from[(hash(k as i64, 1) * from.len() as f64) as usize], (t0 + LOOP).rem_euclid(LOOP), k as i64, None, SUN);
        }
    }
    for (k, l) in under.iter().flatten().filter(|l| hash(l.y, l.x * 3) < 0.05).enumerate() {
        let k = k as i64;
        fall(l, l.turn + FADE * 0.8, k + 10, Some(SPRING.0 + 3.0 * hash(k, 3)), AUTUMN[1 + (hash(k, 5) * 3.0) as usize % 3]);
    }
    Box::new(Bonsai { wood, leaves, falls, dots: ColorOctants::new(META.cols, META.rows) })
}

/// Whether a leaf belongs to pad n: it lies within that pad's bounds.
fn pad_of(l: &Leaf, n: usize) -> bool {
    let (px, py, pw, ph, _) = PADS[n];
    let (x, y) = (l.x as f64 / 2.0, l.y as f64 / 4.0);
    (x - px - 0.5).abs() <= pw + 0.5 && y >= py - ph - 0.5 && y < py + 2.0
}

impl Piece for Bonsai {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let u = t.rem_euclid(LOOP);
        self.dots.clear();
        for &(x, y, color) in &self.wood {
            self.dots.set(x, y, color);
        }
        // A gust runs through the crown now and then, stirring the leaves.
        let gust = (u / LOOP * PI * 8.0).sin().max(0.0).powi(3);
        for l in &self.leaves {
            let (f, color) = if u >= l.turn - TINT && u < SPRING.0 {
                // Turning: green, yellow, orange, red, then thinning away.
                let s = ((u - (l.turn - TINT)) / TINT).clamp(0.0, 1.0);
                let f = if u >= l.turn { (1.0 - (u - l.turn) / FADE).max(0.0) } else { 1.0 };
                (f, AUTUMN[((s * 3.0).round() as usize).min(3)])
            } else if u >= SPRING.0 && u < l.bud {
                (0.0, BUD)
            } else if u >= l.bud && u < l.bud + GROW + 2.0 {
                // A bud, pale at first, deepening into summer's green.
                (
                    (u - l.bud) / GROW,
                    if u < l.bud + GROW {
                        BUD
                    } else if l.lit > 0.45 {
                        SUN
                    } else {
                        SHADE
                    },
                )
            } else {
                (1.0, if l.lit > 0.45 { SUN } else { SHADE })
            };
            if f <= 0.0 {
                continue;
            }
            let stir = gust * 0.12 * (u * 5.0 - l.x as f64 * 0.25 + l.h * 3.0).sin();
            let v = ((if env.paper { l.night } else { l.day }) * f.min(1.0) + stir).clamp(0.0, 1.0);
            if v > 0.06 && dither(l.x as usize, l.y as usize, 0.15 + v) {
                self.dots.set(l.x, l.y, color);
            }
        }
        for f in &self.falls {
            let age = (u - f.t0 + LOOP).rem_euclid(LOOP);
            let step = age / DROP;
            let n = f.path.len();
            if step < n as f64 {
                // Between rows, eased along its path, fluttering flat then edge on.
                let k = step.floor() as usize;
                let (a, b) = (f.path[k], f.path[(k + 1).min(n - 1)]);
                let s = step - k as f64;
                let (x, y) = ((a.0 + (b.0 - a.0) * s).floor() as i64, (a.1 + (b.1 - a.1) * s).floor() as i64);
                if self.dots.get(x, y).is_none() {
                    self.dots.set(x, y, f.color);
                    if k.is_multiple_of(2) {
                        self.dots.set(x + 1, y, f.color);
                    } else {
                        self.dots.set(x, y + 1, f.color);
                    }
                }
            } else if age < f.lie {
                // Lying on the soil.
                let (x, _) = f.path[n - 1];
                self.dots.set(x.floor() as i64, 59, f.color);
                self.dots.set(x.floor() as i64 + 1, 59, f.color);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
