//! snake: a diamondback seen from above, sliding along its own S-shaped
//! track and flicking its tongue. It leaves the right edge and comes back on
//! the left. After ascii.rest's snake.ts (MIT, @bas3line).
//!
//! The track, the body's girth and the tongue's flicks are ascii.rest's; the
//! snake is drawn in octants, sliding a dot at a time, its body as wide as it
//! is at every point along it and its tail thinning to a rattle. In colour
//! it is a diamondback: a tan back with a chain of dark brown diamonds edged
//! in cream, paler flanks, a dark head with bright eyes, a red forked tongue
//! and a pale rattle.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// The snake's meta.
pub static META: Meta = Meta {
    name: "snake",
    slug: "snake",
    category: Category::Creatures,
    note: "an octant diamondback slithering an s-curve, tongue flicking, a rattle",
    cols: 64,
    rows: 15,
    fps: 20,
    charset: Charset::Extended,
    // The back, its diamonds and their edges, the flanks; the head, the eyes,
    // the tongue, the rattle.
    palette: Some(&["#b8925a", "#5a3a20", "#ecdcb4", "#d0b080", "#6a4a2a", "#f2e070", "#e0303a", "#d8c8a0"]),
    ground: Some("#0e0d0a"),
    ..Meta::DEFAULT
};

const BACK: u8 = 0;
const DIAMOND: u8 = 1;
const EDGE: u8 = 2;
const FLANK: u8 = 3;
const HEAD: u8 = 4;
const EYE: u8 = 5;
const TONGUE: u8 = 6;
const RATTLE: u8 = 7;
/// The world wraps at the frame's width. The track: wavelength, height and
/// middle row. Body length behind the snout and columns a second; the head's
/// length; columns of neck over which the body eases onto the track; the
/// last columns of the tail, a thread.
const W: f64 = 64.0;
const LAMBDA: f64 = 32.0;
const AMP: f64 = 3.5;
const MID: f64 = 7.0;
const LEN: f64 = 44.0;
const SPEED: f64 = 6.4;
const HL: f64 = 11.0;
const NECK: f64 = 9.0;
const TIP: f64 = 6.0;
/// Columns from one diamond to the next.
const DIAMONDS: f64 = 3.2;

fn ease(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn track_y(x: f64) -> f64 {
    MID + AMP * (TAU / LAMBDA * x).sin()
}

/// Half the body's thickness in half rows, by distance behind the snout: a
/// neck as wide as the back of the head, a thick middle and a long taper.
fn girth(d: f64) -> f64 {
    if d < HL + 4.0 {
        return 2.1;
    }
    let f = d / LEN;
    if f < 0.45 {
        return 2.1 + 0.9 * ((d - HL - 4.0) / (0.45 * LEN - HL - 4.0) * PI / 2.0).sin();
    }
    3.0 * (1.0 - ((f - 0.45) / 0.55).powf(1.3)).max(0.0)
}

/// The snake's dots.
pub struct Snake {
    dots: ColorOctants,
}

/// A snake; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Snake { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Snake {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let tip = (t * SPEED + 50.0).rem_euclid(W);
        // The head sits level, where the track meets the back of it.
        let r0 = track_y(tip - HL + 0.5) - 0.5;
        // The body's middle line by distance behind the snout: level at the
        // head, easing onto the track down the neck.
        let mid_y = |d: f64| r0 + 0.5 + (track_y(tip - d) - r0 - 0.5) * ease((d - HL) / NECK);
        let behind = |x: f64| (tip - x).rem_euclid(W);
        self.dots.clear();
        for j in 0..h {
            for i in 0..w {
                let (x, y) = ((i as f64 + 0.5) / 2.0, (j as f64 + 0.5) / 4.0);
                let d = behind(x);
                // The head: a broad wedge, level, narrowing to the snout, with
                // an eye either side of the midline.
                if d < HL {
                    let f = d / HL; // 0 at the snout, 1 at the back of the head
                    let half = 0.35 + 1.6 * (f * PI * 0.62).sin();
                    let off = y - (r0 + 0.5);
                    if off.abs() < half {
                        let eye = (d - 3.6).abs() < 0.8 && (off.abs() - 0.95).abs() < 0.3;
                        self.dots.set(i as i64, j as i64, if eye { EYE } else { HEAD });
                    }
                    continue;
                }
                if d >= LEN - TIP {
                    continue;
                }
                let g = girth(d);
                if g < 0.05 {
                    continue;
                }
                // How far across the body this dot is, as a share of its half width.
                let s = (mid_y(d - 0.5) - mid_y(d + 0.5)) * 2.0;
                let v = ((y - mid_y(d)) * 2.0 / 1.0f64.hypot(s) / g).abs();
                if v >= 1.0 {
                    continue;
                }
                // A chain of diamonds down an even back, the flanks paler;
                // toward the tail the pattern gives out.
                let p = ((d / DIAMONDS).fract() - 0.5).abs() * 2.0 + v * 1.4;
                let color = if d > 0.85 * LEN {
                    BACK
                } else if p < 0.55 {
                    DIAMOND
                } else if p < 0.8 {
                    EDGE
                } else if v > 0.62 {
                    FLANK
                } else {
                    BACK
                };
                // The diamonds are sparser than the back, so they show in one ink too.
                if color != DIAMOND || dither(i, j, 0.5) {
                    self.dots.set(i as i64, j as i64, color);
                }
            }
        }
        // The tip of the tail, a thread ending in its rattle.
        let mut d = LEN - TIP;
        while d < LEN {
            let (x, y) = ((tip - d).rem_euclid(W), mid_y(d));
            let rattle = d > LEN - 1.6;
            let (px, py) = ((x * 2.0) as i64, (y * 4.0) as i64);
            self.dots.set(px, py, if rattle { RATTLE } else { BACK });
            if rattle {
                self.dots.set(px, py + 1, RATTLE);
            }
            d += 0.1;
        }
        // The forked tongue leaves the snout's tip, flicking out twice.
        let fl = t.rem_euclid(2.2);
        let len = if fl > 0.2 && fl < 0.55 {
            3.0
        } else if fl > 0.75 && fl < 1.0 {
            2.0
        } else {
            0.0
        };
        if len > 0.0 {
            let y = ((r0 + 0.5) * 4.0) as i64;
            let x0 = (tip * 2.0) as i64;
            for k in 0..(len * 2.0) as i64 {
                self.dots.set((x0 + k).rem_euclid(w as i64), y, TONGUE);
            }
            let end = (x0 + (len * 2.0) as i64).rem_euclid(w as i64);
            self.dots.set(end, y - 1, TONGUE);
            self.dots.set(end, y + 1, TONGUE);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
