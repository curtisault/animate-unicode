//! fox: a fox asleep, curled up with its chin on its tail. Its flank rises
//! and falls; now and then it opens one eye and looks about, or flicks an
//! ear. After ascii.rest's fox.ts (MIT, @bas3line).
//!
//! The pose and its timings are ascii.rest's; the fox is redrawn in octants
//! from round shapes: a broad head with two pointed ears, the body curled
//! round behind it, and the brush wrapped round the front for the chin to
//! rest on, so the flank swells smoothly as it breathes. In colour it is a
//! red fox: an orange coat darker along the back, white cheeks, chest and
//! tail tip, black-backed ear tips and nose, and an amber eye when it peeks.

use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};

/// The fox's meta.
pub static META: Meta = Meta {
    name: "fox",
    slug: "fox",
    category: Category::Creatures,
    note: "an octant red fox asleep on its white-tipped tail, peeking now and then",
    cols: 47,
    rows: 16,
    fps: 10,
    charset: Charset::Extended,
    // The coat, the darker back, white, black, the eye, the eyelids; the zs.
    palette: Some(&["#e2762c", "#b0521e", "#f2ece2", "#2a1c16", "#ffc23e", "#5a2e14", "#9ab0d8"]),
    ground: Some("#0e0c0b"),
    ..Meta::DEFAULT
};

const COAT: u8 = 0;
const BACK: u8 = 1;
const WHITE: u8 = 2;
const BLACK: u8 = 3;
const AMBER: u8 = 4;
const LID: u8 = 5;
const ZED: u8 = 6;
/// Seconds a loop and a breath (the flank is out for the middle of it); the
/// peek, each step's span and where the eye looks; when the ear flicks; and
/// how often and how long a z drifts up.
const LOOP: f64 = 12.0;
const BREATH: f64 = 4.0;
const PEEK: [(f64, f64, f64, bool); 6] =
    [(2.2, 2.5, 0.0, false), (2.5, 3.0, 0.0, true), (3.0, 3.5, -0.6, true), (3.5, 4.0, 0.6, true), (4.0, 4.3, 0.0, true), (4.3, 4.6, 0.0, false)];
const EAR: [(f64, f64); 2] = [(7.0, 7.2), (7.45, 7.65)];
const Z_EVERY: f64 = 2.0;
const Z_LIFE: f64 = 4.0;

fn ellipse(x: f64, y: f64, cx: f64, cy: f64, rx: f64, ry: f64) -> f64 {
    ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2)
}

/// The fox's paint at (x, y) and the part it is on (head, ears, brush,
/// body), given how far its flank is out, the eye (shut, half open or open,
/// and where it looks), and whether the ear is flicked.
fn paint(x: f64, y: f64, flank: f64, eye: Option<(f64, bool)>, flick: bool) -> Option<(u8, u8)> {
    // Eyes: the left one may open; both otherwise are shut, a dark crescent each.
    if let Some((look, open)) = eye {
        let d = ellipse(x, y, 17.0, 5.3, 1.3, 0.55);
        if d < 1.0 {
            return Some((
                if open && ellipse(x, y, 17.0 + look, 5.3, 0.55, 0.4) < 1.0 {
                    BLACK
                } else if open {
                    AMBER
                } else {
                    LID
                },
                1,
            ));
        }
    }
    for ex in [17.0, 24.0] {
        if (eye.is_none() || ex > 20.0) && ellipse(x, y, ex, 5.5, 1.5, 0.45) < 1.0 && y > 5.5 {
            return Some((LID, 1));
        }
    }
    if ellipse(x, y, 20.5, 7.3, 0.9, 0.4) < 1.0 {
        return Some((BLACK, 1));
    }
    // Ears, black at their tips; the right one folds when it flicks.
    let ear = |apex: (f64, f64), half: f64, base: f64| y >= apex.1 && y <= base && (x - apex.0).abs() < half * (y - apex.1) / (base - apex.1);
    let right = if flick { ear((27.5, 2.2), 2.6, 4.0) } else { ear((27.0, 0.6), 2.4, 4.0) };
    if ear((14.0, 0.6), 2.4, 4.0) || right {
        return Some((if y < 1.9 + f64::from(u8::from(flick && x > 20.0)) * 0.8 { BLACK } else { COAT }, 2));
    }
    // The head, white at the cheeks and chin.
    if ellipse(x, y, 20.5, 5.4, 9.6, 2.9) < 1.0 {
        let cheek = y > 6.1 && (x - 20.5).abs() < 7.5;
        return Some((if cheek { WHITE } else { COAT }, 1));
    }
    if ellipse(x, y, 20.0, 8.6, 5.0, 1.2) < 1.0 {
        return Some((WHITE, 1));
    }
    // The brush, wrapped round the front, white at its tip under the chin.
    let tail = [(41.0, 12.0), (34.0, 13.5), (24.0, 13.9), (13.0, 13.4), (6.0, 12.0), (4.0, 10.3), (6.5, 9.0)];
    for (k, s) in tail.windows(2).enumerate() {
        let ((ax, ay), (bx, by)) = (s[0], s[1]);
        let (dx, dy) = ((bx - ax) / 2.0, by - ay);
        let (px, py) = ((x - ax) / 2.0, y - ay);
        let f = ((px * dx + py * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        let thick = 1.3 - 0.12 * k as f64;
        if (px - f * dx).hypot(py - f * dy) < thick {
            return Some((
                if k >= 4 {
                    WHITE
                } else if y < ay + f * dy - 0.4 {
                    BACK
                } else {
                    COAT
                },
                3,
            ));
        }
    }
    // The body, curled round behind the head; the flank swells as it breathes.
    if ellipse(x, y, 25.0 + flank * 0.4, 10.0, 17.0 + flank, 4.3) < 1.0 {
        return Some((if y < 8.4 { BACK } else { COAT }, 4));
    }
    None
}

/// The fox's dots.
pub struct Fox {
    dots: ColorOctants,
}

/// A fox; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Fox { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Fox {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let u = t.rem_euclid(LOOP);
        let b = u % BREATH;
        // Out for the middle of the breath, easing in and out.
        let flank = ((b - 0.3) / 0.4).clamp(0.0, 1.0).min(((2.7 - b) / 0.4).clamp(0.0, 1.0));
        let eye = PEEK.iter().find(|&&(a, z, ..)| u >= a && u < z).map(|&(_, _, look, open)| (look, open));
        let flick = EAR.iter().any(|&(a, z)| u >= a && u < z);
        let at: Vec<Option<(u8, u8)>> = (0..w * h).map(|k| paint((k % w) as f64 / 2.0 + 0.25, (k / w) as f64 / 4.0 + 0.125, flank, eye, flick)).collect();
        draw(&mut self.dots, &at, w);
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // A z is only let out while both eyes are shut, drifting up and away.
        let awake = |u: f64| u >= PEEK[0].0 - 0.3 && u < PEEK[PEEK.len() - 1].1 + 0.6;
        for k in -2..=(LOOP / Z_EVERY).ceil() as i64 {
            let born = k as f64 * Z_EVERY;
            let age = u - born;
            if !(0.0..Z_LIFE).contains(&age) || awake(born.rem_euclid(LOOP)) {
                continue;
            }
            let f = age / Z_LIFE;
            let (c, r) = ((33.0 + f * 9.0).floor() as i64, (5.0 - f * 4.0).floor() as i64);
            grid.set(c, r, if f < 0.6 { 'z' } else { 'Z' }, ZED);
        }
    }
}

/// Draws a creature from its paint and parts by dot: the coat dense, the
/// shut eyes gaps in it, and a dot left out wherever one part meets another,
/// so the parts keep their outlines in one ink too.
fn draw(dots: &mut ColorOctants, at: &[Option<(u8, u8)>], w: usize) {
    dots.clear();
    for (k, a) in at.iter().enumerate() {
        let Some((c, part)) = *a else { continue };
        let other = |n: usize| at.get(n).copied().flatten().is_some_and(|(_, p)| p != part);
        if c == LID || ((k + 1) % w != 0 && other(k + 1)) || other(k + w) {
            continue;
        }
        dots.set((k % w) as i64, (k / w) as i64, c);
    }
}
