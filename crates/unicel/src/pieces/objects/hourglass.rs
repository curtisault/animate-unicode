//! hourglass: an hourglass on its stand. Sand runs through the neck a grain
//! at a time, a crater opening above and a heap rising below; when the top
//! runs dry it is lifted, turned over on its side and set down the other way
//! up. After ascii.rest's hourglass.ts (MIT, @bas3line).
//!
//! The glass, the sand's crater and heap, the stream and the turn are
//! ascii.rest's. The sand is braille, a dot a grain, so the crater's slopes
//! and the heap's cone are smooth; the glass is two fine curves of dots with
//! a highlight down the inside of its left wall; the stand keeps its
//! box-drawn plates and posts. In colour the sand is gold, the glass a pale
//! blue with a white highlight, and the stand dark wood.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::FRAC_PI_2;

/// The hourglass's meta.
pub static META: Meta = Meta {
    name: "hourglass",
    slug: "hourglass",
    category: Category::Objects,
    note: "braille sand running grain by grain through glass, then turned over",
    cols: 43,
    rows: 21,
    fps: 30,
    charset: Charset::Extended,
    // Sand; the glass; its highlight; the stand.
    palette: Some(&["#eac46e", "#86aac4", "#eef8ff", "#8a5a34"]),
    ground: Some("#0c0b0a"),
    ..Meta::DEFAULT
};

const SAND: u8 = 0;
const GLASS: u8 = 1;
const SHINE: u8 = 2;
const WOOD: u8 = 3;
/// A cell in em (lengths are in em); a bulb's length, neck to cap; the
/// glass's half width at the neck, at its widest and at the cap; the posts
/// out from the middle; the slope sand comes to rest at; how high the sand
/// stands when it is all on top.
const W: f64 = 0.6;
const H: f64 = 1.2;
const L: f64 = 9.0;
const RN: f64 = 0.36;
const RM: f64 = 5.0;
const RC: f64 = 4.3;
const PX: f64 = 6.0;
const K: f64 = 0.6;
const LT: f64 = 6.2;
/// Seconds for the top to run dry, a pause, the turn (lifted, on its side,
/// the other way up), where frame 0 falls, and between grains through the neck.
const RUN: f64 = 10.5;
const HOLD: f64 = 1.2;
const TURN: [f64; 3] = [0.2, 0.45, 0.2];
const START: f64 = 3.5;
const DT: f64 = 0.045;

/// The glass's half width at a distance u from the neck: a narrow throat, a
/// full bulb, rounded in to the cap.
fn w(u: f64) -> f64 {
    let s = (u.abs() / L).min(1.0);
    if s < 0.76 {
        RN + (RM - RN) * (FRAC_PI_2 * (s / 0.76)).sin().powf(1.5)
    } else {
        RM - (RM - RC) * ((s - 0.76) / 0.24).powi(2)
    }
}

fn hash(n: i64) -> f64 {
    let mut h = ((n as u32) ^ 0x2c1b_3c6d).wrapping_mul(0x297a_2d39);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca77);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

fn solve(f: impl Fn(f64) -> f64, want: f64, mut a: f64, mut b: f64) -> f64 {
    for _ in 0..30 {
        if f((a + b) / 2.0) < want {
            a = (a + b) / 2.0;
        } else {
            b = (a + b) / 2.0;
        }
    }
    (a + b) / 2.0
}

/// The hourglass: the bulb's columns, the crater and heap tables, the level
/// sand slumps to on its side, the stand drawn upright, lifted and on its
/// side, and the dots.
pub struct Hourglass {
    crater: Vec<f64>,
    heap: Vec<f64>,
    side: f64,
    stands: [Vec<(i64, i64, char)>; 3],
    dots: ColorDots,
}

/// An hourglass; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    // One bulb column by column (v, lo, hi): how far from the neck its inside
    // runs, so sand can be weighed.
    let mut bulb = vec![];
    let mut v = -RM + 0.05;
    while v < RM {
        let (mut lo, mut hi) = (L, 0.0f64);
        let mut u = 0.0;
        while u <= L {
            if v.abs() < w(u) - 0.15 {
                lo = lo.min(u);
                hi = u;
            }
            u += 0.02;
        }
        if hi > lo {
            bulb.push((v, lo, hi));
        }
        v += 0.1;
    }
    let area = |f: &dyn Fn(f64, f64, f64) -> f64| bulb.iter().map(|&(v, lo, hi)| f(v, lo, hi)).sum::<f64>();
    let top_area = |c: f64| area(&|v, lo, hi| (hi.min(LT).min(c + K * v.abs()) - lo).max(0.0));
    let heap_area = |p: f64| area(&|v, lo, hi| (hi - lo.max(L - p + K * v.abs())).max(0.0));
    let all = top_area(LT);
    let crater = (0..=50).map(|i| solve(top_area, f64::from(i) / 50.0 * all, -K * RM - 1.0, LT)).collect();
    let heap = (0..=50).map(|i| solve(heap_area, f64::from(i) / 50.0 * all, 0.0, L + K * RM)).collect();
    // On its side, the sand slumps against the lower wall of its bulb.
    let side = solve(|lv| area(&|v, lo, hi| if v > lv { hi - lo } else { 0.0 }), all, RM, -RM);
    // The stand: plates with turned ends and the posts let into them; on its
    // side the plates stand up at either end and the posts run across.
    let (mid, pc) = (cols / 2, (PX / W).round() as i64);
    let n0 = (rows as f64 / 2.0 - 0.5).round() as i64; // the neck's row, upright
    let upright = |lift: i64| {
        let mut g = vec![];
        let (top, bot, hw) = (n0 - lift - 8, n0 - lift + 8, pc + 2);
        for c in mid - hw..=mid + hw {
            let end = if c == mid - hw {
                0
            } else if c == mid + hw {
                1
            } else {
                -1
            };
            let post = (c - mid).abs() == pc;
            let pick = |s: [char; 2], other: char| if end < 0 { other } else { s[end as usize] };
            g.push((top - 1, c, pick(['╒', '╕'], '═')));
            g.push((top, c, pick(['╘', '╛'], if post { '╤' } else { '═' })));
            g.push((bot, c, pick(['╒', '╕'], if post { '╧' } else { '═' })));
            g.push((bot + 1, c, pick(['╘', '╛'], '═')));
        }
        for r in top + 1..bot {
            g.push((r, mid - pc, '│'));
            g.push((r, mid + pc, '│'));
        }
        g
    };
    let mut lying = vec![];
    let hr = (PX / H).round() as i64;
    let ends = [mid - 16, mid + 16];
    for (k, &c) in ends.iter().enumerate() {
        for r in n0 - hr - 1..=n0 + hr + 1 {
            let edge = if r == n0 - hr - 1 {
                0
            } else if r == n0 + hr + 1 {
                1
            } else {
                -1
            };
            let post = (r - n0).abs() == hr;
            if edge < 0 {
                lying.push((r, c - 1, if post && k == 1 { '╢' } else { '║' }));
                lying.push((r, c + 1, if post && k == 0 { '╟' } else { '║' }));
            } else {
                lying.push((r, c - 1, ['╓', '╙'][edge as usize]));
                lying.push((r, c, '─'));
                lying.push((r, c + 1, ['╖', '╜'][edge as usize]));
            }
        }
    }
    for c in ends[0] + 2..ends[1] - 1 {
        lying.push((n0 - hr, c, '─'));
        lying.push((n0 + hr, c, '─'));
    }
    Box::new(Hourglass { crater, heap, side, stands: [upright(0), upright(1), lying], dots: ColorDots::new(META.cols, META.rows) })
}

fn table(tab: &[f64], x: f64) -> f64 {
    let f = x.clamp(0.0, 1.0) * 50.0;
    let i = (f.floor() as usize).min(49);
    tab[i] + (tab[i + 1] - tab[i]) * (f - i as f64)
}

impl Piece for Hourglass {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (dw, dh) = (self.dots.width() as i64, self.dots.height() as i64);
        let (cx, cy) = (cols / 2.0 * W, rows / 2.0 * H); // the neck
        let cycle = RUN + HOLD + TURN.iter().sum::<f64>();
        let k = (START + t).rem_euclid(cycle);
        let turn = k - RUN - HOLD;
        let phase = if turn < 0.0 {
            -1
        } else if turn < TURN[0] {
            0
        } else if turn < TURN[0] + TURN[1] {
            1
        } else {
            2
        };
        let side = phase == 1;
        let lift = if phase == 0 || phase == 2 { 1.0 } else { 0.0 };
        let left = if phase == 2 { 1.0 } else { (1.0 - k / RUN).clamp(0.0, 1.0) }; // the share of sand still on top
        let (c_floor, h_peak) = (table(&self.crater, left), table(&self.heap, 1.0 - left));
        let sand = |u: f64, v: f64| {
            if side {
                u < 0.0 && v > self.side
            } else if u > 0.0 {
                u < LT.min(c_floor + K * v.abs())
            } else {
                L + u < h_peak - K * v.abs()
            }
        };
        // Where a point of the glass (u from the neck along it, v across) is on screen, in dots.
        let at = |u: f64, v: f64| {
            let (x, y) = if side { (cx + u, cy + v) } else { (cx + v, cy - lift * H - u) };
            ((x / W * 2.0).floor() as i64, (y / H * 4.0).floor() as i64)
        };
        self.dots.clear();
        // Sand: a grain at every dot inside the glass that sits in the sand.
        for j in 0..dh {
            for i in 0..dw {
                let (x, y) = ((i as f64 + 0.5) / 2.0 * W - cx, cy - lift * H - (j as f64 + 0.5) / 4.0 * H);
                let (u, v) = if side { (x, -y) } else { (y, x) };
                if u.abs() < L && v.abs() < w(u) - 0.15 && sand(u, v) {
                    self.dots.set(i, j, SAND);
                }
            }
        }
        // The stream: grains leave the neck one by one and speed up as they fall onto the heap.
        if phase < 0 {
            let peak = -L + h_peak;
            let mut i = ((k - 0.8) / DT).floor().max(0.0) as i64;
            while i as f64 * DT <= k.min(RUN) {
                let a = k - i as f64 * DT - hash(i + 7919) * DT * 0.7;
                let u = 0.1 - 3.0 * a - 18.0 * a * a;
                if hash(i) >= 0.18 && a >= 0.0 && u >= peak {
                    let (x, y) = at(u, 0.0);
                    self.dots.set(x, y, SAND);
                }
                i += 1;
            }
        }
        // The glass: both walls as fine curves, and a highlight down the inside of the left.
        let mut u = -L;
        while u <= L {
            for sv in [-1.0, 1.0] {
                let (x, y) = at(u, sv * w(u));
                self.dots.set(x, y, GLASS);
            }
            if u.abs() > 1.6 && u.abs() < L - 0.8 {
                let (x, y) = at(u, -(w(u) - 0.55));
                if (u * 2.0).rem_euclid(2.0) < 1.4 {
                    self.dots.set(x, y, SHINE);
                }
            }
            u += 0.04;
        }
        // The caps.
        for sv in [-L, L] {
            let mut v = -RC;
            while v <= RC {
                let (x, y) = at(sv, v);
                self.dots.set(x, y, GLASS);
                v += 0.15;
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        let stand = &self.stands[if side { 2 } else { lift as usize }];
        for &(r, c, ch) in stand {
            grid.set(c, r, ch, WOOD);
        }
    }
}
