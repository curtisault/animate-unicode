//! owl: a horned owl on a branch at night. It blinks slowly, and now and then
//! turns its head to look off to one side before facing front again. After
//! ascii.rest's owl.ts (MIT, @bas3line).
//!
//! The blinks and the turns are ascii.rest's; the owl is redrawn in octants
//! from round shapes: an egg of a body with wings folded at its sides, a
//! broad head with two ear tufts and a pale facial disc, talons gripping the
//! branch. Its head turns smoothly, the near tuft sliding in and the far one
//! narrowing. In colour it is a great horned owl: brown plumage, darker
//! scalloped wings, a pale breast barred brown, a tawny facial disc ringed in
//! dark, and big amber eyes whose catchlights brighten as they open from a
//! blink, under twinkling stars.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The owl's meta.
pub static META: Meta = Meta {
    name: "owl",
    slug: "owl",
    category: Category::Creatures,
    note: "an octant horned owl on a branch, amber eyes blinking, head turning",
    cols: 41,
    rows: 24,
    fps: 12,
    charset: Charset::Extended,
    // Plumage, wings, breast, bars; the disc and its rim; iris, pupil, the
    // catchlight; beak; talons; branch and its bark; a star.
    palette: Some(&[
        "#8c6240", "#553822", "#ccaa7e", "#6a4a30", "#dcc49c", "#463020", "#ffb428", "#141010", "#ffffff", "#3e3c44", "#c8a050", "#5a4030", "#7c5c40",
        "#9aa0c0",
    ]),
    ground: Some("#07080e"),
    ..Meta::DEFAULT
};

const PLUME: u8 = 0;
const WING: u8 = 1;
const BREAST: u8 = 2;
const BAR: u8 = 3;
const DISC: u8 = 4;
const DISC_RIM: u8 = 5;
const IRIS: u8 = 6;
const PUPIL: u8 = 7;
const GLINT: u8 = 8;
const BEAK: u8 = 9;
const TALON: u8 = 10;
const BRANCH: u8 = 11;
const BARK: u8 = 12;
const STAR: u8 = 13;
/// Seconds a loop; head turns (start, end, from, to, in poses); blinks'
/// start times and how long one takes.
const LOOP: f64 = 12.0;
const TURNS: [(f64, f64, f64, f64); 4] = [(2.0, 2.5, 0.0, -2.0), (4.4, 5.1, -2.0, 0.0), (7.4, 7.9, 0.0, 2.0), (9.8, 10.5, 2.0, 0.0)];
const BLINKS: [f64; 4] = [0.6, 6.1, 6.9, 11.2];
const BLINK: f64 = 0.7;
const STARS: [(i64, i64); 7] = [(2, 1), (5, 0), (1, 5), (37, 0), (39, 3), (36, 6), (39, 11)];

fn ease(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// The head's turn, in poses from -2 to 2, eased.
fn pose_at(u: f64) -> f64 {
    let mut p = 0.0;
    for &(a, b, from, to) in &TURNS {
        if u >= a {
            p = from + (to - from) * ease((u - a) / (b - a));
        }
    }
    p
}

/// How shut the eyes are, 0 open to 1 shut: they close slowly and open slower.
fn lid_at(u: f64) -> f64 {
    for &s in &BLINKS {
        let d = u - s;
        if (0.0..BLINK).contains(&d) {
            return if d < 0.3 { ease(d / 0.25) } else { 1.0 - ease((d - 0.3) / (BLINK - 0.3)) };
        }
    }
    0.0
}

fn ellipse(x: f64, y: f64, cx: f64, cy: f64, rx: f64, ry: f64) -> f64 {
    ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2)
}

/// The owl's paint at (x, y) and the part it is on, given its head's turn,
/// how shut its eyes are, and how bright the catchlights are.
fn paint(x: f64, y: f64, pose: f64, lid: f64, glint: f64) -> Option<(u8, u8)> {
    let cx = 20.5;
    let shift = pose * 1.1;
    // The eyes, big and round, under lids that close from the top.
    for (k, ex) in [cx - 5.2 + shift, cx + 5.2 + shift].into_iter().enumerate() {
        // The far eye narrows as the head turns away from it.
        let far = if (k == 0) == (pose > 0.0) { 1.0 - pose.abs() * 0.12 } else { 1.0 };
        let d = ellipse(x, y, ex, 6.1, 2.6 * far, 1.35);
        if d < 1.0 {
            let lid_line = 6.1 - 1.35 + 2.7 * lid;
            if y < lid_line {
                return Some((PLUME, 3));
            }
            if ellipse(x, y, ex - 0.6, 5.6, 0.5, 0.25) < 1.0 && glint > 0.0 {
                return Some((GLINT, 2));
            }
            return Some((if ellipse(x, y, ex, 6.1, 1.1 * far, 0.7) < 1.0 { PUPIL } else { IRIS }, 2));
        }
    }
    // The beak between them.
    let bx = cx + shift;
    if y > 6.7 && y < 8.4 && (x - bx).abs() < 0.9 * (8.4 - y) / 1.7 + 0.15 {
        return Some((BEAK, 3));
    }
    // The ear tufts: the near one slides in, the far one narrows.
    let tuft = |ax: f64, half: f64| (0.8..=3.2).contains(&y) && (x - ax).abs() < half * (y - 0.8) / 2.4;
    if tuft(cx - 9.6 + shift * 1.4, 1.6 - pose.max(0.0) * 0.4) || tuft(cx + 9.6 + shift * 1.4, 1.6 + pose.min(0.0) * 0.4) {
        return Some((WING, 3));
    }
    // The facial disc, ringed in dark, on a broad head.
    let disc = ellipse(x, y, cx + shift, 6.0, 10.6, 2.9);
    if disc < 1.0 {
        return Some((if disc > 0.8 { DISC_RIM } else { DISC }, 3));
    }
    if ellipse(x, y, cx + shift * 0.6, 5.6, 13.0, 3.9) < 1.0 && y > 2.0 {
        return Some((PLUME, 3));
    }
    // The talons, gripping the branch.
    for fx in [cx - 6.0, cx + 6.0] {
        if y > 19.4 && y < 20.4 && (x - fx).abs() < 1.6 {
            return Some((TALON, 5));
        }
    }
    // The body, wings folded at its sides, a barred breast down its front.
    let body = ellipse(x, y, cx, 14.4, 12.2, 5.8);
    if body < 1.0 && y > 8.6 {
        let side = ((x - cx) / 12.2).abs();
        if side > 0.62 {
            // Scalloped feathers on the wings.
            let scallop = ((x * 1.3).sin() + (y * 2.0).sin()) > 1.2;
            return Some((if scallop { PLUME } else { WING }, 4));
        }
        let bar = (y * 2.1 + ((x - cx) * 0.9).abs() * 0.6).sin() > 0.55;
        return Some((if bar { BAR } else { BREAST }, 1));
    }
    // The branch: a log with bark.
    if y > 20.0 && y < 22.0 {
        let bark = ((x * 0.7).sin() + (y * 4.0).sin() * 0.6) > 0.7;
        return Some((if bark { BARK } else { BRANCH }, 6));
    }
    None
}

/// The owl's dots.
pub struct Owl {
    dots: ColorOctants,
}

/// An owl; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Owl { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Owl {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let u = t.rem_euclid(LOOP);
        let (pose, lid) = (pose_at(u), lid_at(u));
        // The catchlights brighten as the eyes open again.
        let glint = if lid < 0.4 { 1.0 } else { 0.0 };
        self.dots.clear();
        let at: Vec<Option<(u8, u8)>> = (0..w * h).map(|k| paint((k % w) as f64 / 2.0 + 0.25, (k / w) as f64 / 4.0 + 0.125, pose, lid, glint)).collect();
        // Dense plumage, the bars and scallops sparser, and a dot left out
        // wherever one part meets another, so the parts keep their outlines.
        for (k, a) in at.iter().enumerate() {
            let Some((c, part)) = *a else { continue };
            let other = |n: usize| at.get(n).copied().flatten().is_some_and(|(_, p)| p != part);
            let (i, j) = (k % w, k / w);
            if (i + 1 < w && other(k + 1)) || other(k + w) || (matches!(c, BAR | PLUME) && part != 3 && !dither(i, j, 0.55)) {
                continue;
            }
            self.dots.set(i as i64, j as i64, c);
        }
        for (n, &(c, r)) in STARS.iter().enumerate() {
            let on = (u * TAU / LOOP * (n % 3 + 1) as f64 + n as f64 * 2.0).sin() > -0.4;
            if on {
                self.dots.set(c * 2, r * 4 + 1, STAR);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
