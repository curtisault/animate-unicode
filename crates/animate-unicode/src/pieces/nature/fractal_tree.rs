//! fractal tree: a trunk that forks, and forks again, seven times over, each
//! limb a little shorter than its parent, the outer twigs gathered into lobes
//! of leaves lit from the upper left. Wind bends every limb, the tips most.
//! After ascii.rest's fractal-tree.ts (MIT, @bas3line).
//!
//! The forking, the wind and the lobes are ascii.rest's. The trunk is solid
//! octants flaring into its roots, the limbs are braille lines a dot or two
//! thick that thin as they fork, and the leaves are a braille stipple, dense
//! where the light falls and sparse in shadow. In colour the bark is brown,
//! lit on its left; the leaves are a deep green inside the crown and a fresh
//! yellow-green at its tips, where the new growth is.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The fractal tree's meta.
pub static META: Meta = Meta {
    name: "fractal tree",
    slug: "fractal-tree",
    category: Category::Nature,
    note: "a forking braille tree swaying, deep green inside, yellow-green tips",
    cols: 60,
    rows: 24,
    fps: 15,
    charset: Charset::Extended,
    options: Some(r#"{"seed": 4}"#),
    // Bark lit and in shade; leaves in shadow, mid, lit; the tips; the ground.
    palette: Some(&["#9a6c46", "#5a3c26", "#24552a", "#3c8a38", "#6cbc48", "#c4e05a", "#3c3022"]),
    ground: Some("#0b0d0b"),
    ..Meta::DEFAULT
};

const BARK_LIT: u8 = 0;
const BARK: u8 = 1;
const LEAVES: [u8; 3] = [2, 3, 4];
const TIP: u8 = 5;
const EARTH: u8 = 6;
/// Forks, the depth whose limbs carry a lobe of leaves over their twigs,
/// seconds every wind term repeats in, and the furthest any limb leans from
/// upright, in radians.
const DEPTH: u32 = 7;
const LOBE: u32 = 6;
const LOOP: f64 = 12.0;
const MAX_LEAN: f64 = 1.1;

fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

/// A limb: its parent (none for the first two), depth, turn from its parent,
/// and length in columns (a row is two).
struct Limb {
    parent: Option<usize>,
    depth: u32,
    turn: f64,
    len: f64,
}

/// The tree: its limbs (parents before children), each lobe's twigs, phase
/// and bump, the leaves' shade by dot, and the two canvases.
pub struct FractalTree {
    limbs: Vec<Limb>,
    lobes: Vec<(Vec<usize>, f64, f64)>,
    shade: Vec<f32>,
    dots: ColorDots,
    trunk: ColorOctants,
}

/// A fractal tree; reads `seed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let mut rand = Mulberry32::new(options.f64("seed", 4.0) as u32);
    let mut limbs = vec![];
    // The first forks are the widest.
    fn grow(limbs: &mut Vec<Limb>, rand: &mut Mulberry32, parent: Option<usize>, depth: u32, len: f64) {
        for side in [-1.0, 1.0] {
            let spread = match depth {
                1 => 0.58,
                2 => 0.48,
                _ => 0.4 + rand.next_f64() * 0.14,
            } * side
                + (rand.next_f64() - 0.5) * 0.12;
            limbs.push(Limb { parent, depth, turn: spread, len: len * (0.92 + rand.next_f64() * 0.16) });
            let i = limbs.len() - 1;
            if depth < DEPTH {
                grow(limbs, rand, Some(i), depth + 1, len * 0.84);
            }
        }
    }
    grow(&mut limbs, &mut rand, None, 1, 6.0);
    // A lobe is the twigs above one limb: it sits over their ends.
    let mut lobes = vec![];
    for (i, l) in limbs.iter().enumerate() {
        if l.depth == LOBE {
            let mut twigs = vec![i];
            twigs.extend((i + 1..limbs.len()).take_while(|&j| limbs[j].depth > LOBE));
            lobes.push((twigs, rand.next_f64() * TAU, 0.12 + rand.next_f64() * 0.1));
        }
    }
    let dots = ColorDots::new(META.cols, META.rows);
    let shade = vec![0.0; dots.width() * dots.height()];
    Box::new(FractalTree { limbs, lobes, shade, dots, trunk: ColorOctants::new(META.cols, META.rows) })
}

/// The dots along a line between two points in cells, `thick` dots across.
fn line(dots: &mut ColorDots, (x0, y0): (f64, f64), (x1, y1): (f64, f64), thick: i64, color: u8, over: impl Fn(i64, i64) -> bool) {
    let (a, b) = ((x0 * 2.0, y0 * 4.0), (x1 * 2.0, y1 * 4.0));
    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil().max(1.0) as i64;
    for s in 0..=n {
        let f = s as f64 / n as f64;
        let (x, y) = ((a.0 + (b.0 - a.0) * f).floor() as i64, (a.1 + (b.1 - a.1) * f).floor() as i64);
        for d in 0..thick {
            if over(x + d, y) {
                dots.set(x + d, y, color);
            }
        }
    }
}

impl Piece for FractalTree {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let w = self.dots.width();
        let ground = rows - 1.0; // the row the trunk stands on
        let fork = 15.0; // the row the trunk first forks at
        let x0 = cols / 2.0 - 0.5;
        // The wind: a slow sway, a quicker shiver and a gust, all within LOOP,
        // reaching each limb a moment after its parent.
        let wind = |u: f64| 0.65 * (TAU * u / 6.0).sin() + 0.25 * (TAU * u / 2.4 + 1.0).sin() + 0.2 * (TAU * u / LOOP).sin().powi(3);
        let n = self.limbs.len();
        let (mut px, mut py, mut pa) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        let mut strokes = vec![];
        for (i, l) in self.limbs.iter().enumerate() {
            let bend = (0.02 + 0.035 * f64::from(l.depth)) * (0.25 + wind(t - f64::from(l.depth) * 0.12));
            let (x, y, a) = match l.parent {
                None => (x0 + 0.5, fork + 0.5, 0.0),
                Some(p) => (px[p], py[p], pa[p]),
            };
            // Limbs reach for the light: each turns a little back toward upright.
            let a = (a * if l.depth > 2 { 0.85 } else { 1.0 } + l.turn + bend).clamp(-MAX_LEAN, MAX_LEAN);
            let (x1, y1) = (x + a.sin() * l.len, y - a.cos() * l.len / 2.0);
            if l.depth <= LOBE {
                strokes.push(((x, y), (x1, y1), l.depth));
            }
            (px[i], py[i], pa[i]) = (x1, y1, a);
        }
        // The lobes, highest first, so each lower one stands in front of the
        // shaded underside of the one above it. The light falls on the crown
        // as a whole from the upper left, and on each lobe the same way.
        let (mut gx0, mut gx1, mut gy0, mut gy1) = (cols, 0.0f64, rows, 0.0f64);
        for (twigs, ..) in &self.lobes {
            for &j in twigs {
                (gx0, gx1, gy0, gy1) = (gx0.min(px[j]), gx1.max(px[j]), gy0.min(py[j]), gy1.max(py[j]));
            }
        }
        let (gx, gy, grx, gry) = ((gx0 + gx1) / 2.0, (gy0 + gy1) / 2.0, (gx1 - gx0) / 2.0 + 3.0, (gy1 - gy0) / 2.0 + 1.5);
        let mut order: Vec<(f64, f64, f64, f64, f64, f64)> = self
            .lobes
            .iter()
            .map(|(twigs, ph, bump)| {
                // Over the twigs' ends and the fork they spring from.
                let p = self.limbs[twigs[0]].parent.unwrap_or(0);
                let ends: Vec<(f64, f64)> = twigs.iter().map(|&j| (px[j], py[j])).chain([(px[p], py[p])]).collect();
                let (cx, cy) = (ends.iter().map(|e| e.0).sum::<f64>() / ends.len() as f64, ends.iter().map(|e| e.1).sum::<f64>() / ends.len() as f64);
                let (wx, wy) = ends.iter().fold((0.0f64, 0.0f64), |(wx, wy), e| (wx.max((e.0 - cx).abs()), wy.max((e.1 - cy).abs())));
                (cx, cy - 0.4, 1.8 + wx * 0.9, 1.0 + wy * 0.9, *ph, *bump)
            })
            .collect();
        order.sort_by(|p, q| p.1.total_cmp(&q.1));
        self.dots.clear();
        self.shade.fill(-1.0);
        for &(cx, cy, rx, ry, ph, bump) in &order {
            for j in (((cy - ry - 1.0) * 4.0).floor() as i64).max(0)..=(((cy + ry + 1.0) * 4.0) as i64).min(self.dots.height() as i64 - 1) {
                for i in (((cx - rx - 1.0) * 2.0).floor() as i64).max(0)..=(((cx + rx + 1.0) * 2.0) as i64).min(w as i64 - 1) {
                    let (x, y) = ((i as f64 + 0.5) / 2.0, (j as f64 + 0.5) / 4.0);
                    let (nx, ny) = ((x - cx) / rx, (y - cy) / ry);
                    let edge = 1.0 + bump * (ny.atan2(nx) * 3.0 + ph + TAU * t / LOOP).sin();
                    let d2 = (nx * nx + ny * ny) / (edge * edge);
                    if d2 >= 1.0 {
                        continue;
                    }
                    let (gnx, gny) = ((x - gx) / grx, (y - gy) / gry);
                    let v = (0.36 - 0.12 * gnx - 0.16 * gny - 0.24 * nx - 0.4 * ny + 0.16 * (hash(i, j) - 0.5)).clamp(0.0, 1.0);
                    let k = j as usize * w + i as usize;
                    self.shade[k] = v as f32;
                    // At the rim of each lobe light shows between the leaves; the
                    // fresh growth there is yellow-green.
                    let rim = 1.0 - d2 < 0.22;
                    let ink = if env.paper && !rim { 1.0 - v } else { v };
                    if dither(i as usize, j as usize, 0.2 + 0.75 * ink) {
                        self.dots.set(i, j, if rim { TIP } else { LEAVES[((v * 3.0) as usize).min(2)] });
                    } else {
                        self.dots.unset(i, j);
                    }
                }
            }
        }
        // Leaves hide the twigs; the main limbs show where the leaves are dark.
        let shade = &self.shade;
        for &(a, b, depth) in &strokes {
            let thick = if depth <= 2 { 2 } else { 1 };
            line(&mut self.dots, a, b, thick, if depth <= 2 { BARK_LIT } else { BARK }, |x, y| {
                let s = shade.get(y.max(0) as usize * w + x.max(0) as usize).copied().unwrap_or(-1.0);
                depth <= 3 || s < 0.0 || (depth == 4 && s < 0.45)
            });
        }
        // The ground either side of the roots.
        for x in 6..(cols as i64 - 3) * 2 {
            if x % 2 == 0 || (12..(cols as i64 - 6) * 2).contains(&x) {
                self.dots.set(x, (ground * 4.0) as i64 + 3, EARTH);
            }
        }
        // The trunk: solid wood, lit on its left, flaring into roots at the foot.
        self.trunk.clear();
        for y in (fork * 4.0) as i64..((ground + 1.0) * 4.0) as i64 {
            let r = y as f64 / 4.0;
            let flare = (r - ground + 1.6).max(0.0) * 1.6;
            let (l, rt) = (((x0 - 1.0 - flare) * 2.0).round() as i64, ((x0 + 2.0 + flare) * 2.0).round() as i64);
            for x in l..rt {
                self.trunk.set(x, y, if x < (l + rt) / 2 { BARK_LIT } else { BARK });
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        self.trunk.draw(grid, 0, 0);
    }
}
