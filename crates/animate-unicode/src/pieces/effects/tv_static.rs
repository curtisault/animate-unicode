//! tv static: an old set showing snow, with a hum bar rolling through it.
//! The dial clicks over, a test card rolls into place and holds, then is
//! lost. After ascii.rest's tv-static.ts (MIT, @bas3line).
//!
//! The set, the snow, the hum bar, the roll, the tear and the card are
//! ascii.rest's. The snow is octant noise, about half the dots of the screen
//! lit at random and streaked along the scan lines, new every frame; the
//! test card is the colour bars of a real one, each bar dithered at its own
//! level so they still step in one ink, with the circle and crosshair drawn
//! over them. In colour the snow is grey and white, the card has SMPTE's
//! seven bars, its reversed strip and its bottom strip, and the set is a
//! walnut cabinet with silver knobs and ears.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};

/// The set's meta.
pub static META: Meta = Meta {
    name: "tv static",
    slug: "tv-static",
    category: Category::Effects,
    note: "octant snow on an old set tuning into a colour test card, then losing it",
    cols: 58,
    rows: 26,
    fps: 20,
    charset: Charset::Extended,
    options: Some(r#"{"set": true}"#),
    // Snow dim and bright; the bars: grey, yellow, cyan, green, magenta,
    // red, blue; the bottom strip's dark blue, white, purple, black; the card's
    // lines; the cabinet; its silver.
    palette: Some(&[
        "#6a6c74", "#e8eaf0", "#c0c0c0", "#c8c800", "#00c4c8", "#00c000", "#c800c8", "#c80000", "#0000c8", "#00214c", "#f4f4f4", "#32006a", "#101012",
        "#ffffff", "#7a5236", "#b8bcc6",
    ]),
    ground: Some("#08080a"),
    ..Meta::DEFAULT
};

const SNOW_DIM: u8 = 0;
const SNOW_BRIGHT: u8 = 1;
const BAR_COLORS: [u8; 7] = [2, 3, 4, 5, 6, 7, 8];
const NAVY: u8 = 9;
const WHITE: u8 = 10;
const PURPLE: u8 = 11;
const BLACK: u8 = 12;
const LINES: u8 = 13;
const WOOD: u8 = 14;
const SILVER: u8 = 15;
/// Seconds a loop: snow, tuning in, the card, losing it, snow.
const LOOP: f64 = 10.0;

fn hash(a: i64, b: i64, c: i64) -> f64 {
    let mut h = (a as u32).wrapping_mul(0x27d4_eb2d) ^ (b as u32).wrapping_mul(0x1656_67b1) ^ (c as u32).wrapping_mul(0x2545_f491);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn smooth(a: f64, b: f64, v: f64) -> f64 {
    let x = ((v - a) / (b - a)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// A cell of the card: a block of colour at a level (0 to 7), or a line of
/// the circle or crosshair.
#[derive(Clone, Copy)]
enum Card {
    Block(u8, u8),
    Line(char),
}

/// The set: cabinet, rounded screen bezel, two knobs, a grille, legs and ears.
fn draw_set() -> Vec<(i64, i64, char, u8)> {
    let mut g = vec![];
    let mut boxed = |x0: i64, y0: i64, x1: i64, y1: i64, c: [char; 6], color: u8| {
        for x in x0 + 1..x1 {
            g.push((x, y0, c[4], color));
            g.push((x, y1, c[4], color));
        }
        for y in y0 + 1..y1 {
            g.push((x0, y, c[5], color));
            g.push((x1, y, c[5], color));
        }
        g.extend([(x0, y0, c[0], color), (x1, y0, c[1], color), (x0, y1, c[2], color), (x1, y1, c[3], color)]);
    };
    let round = ['╭', '╮', '╰', '╯', '─', '│'];
    boxed(1, 5, 56, 23, round, WOOD);
    boxed(3, 6, 43, 22, round, WOOD);
    boxed(46, 8, 52, 10, round, SILVER); // the channel knob
    boxed(46, 12, 52, 14, round, SILVER); // the volume knob
    boxed(45, 16, 53, 21, ['┌', '┐', '└', '┘', '─', '│'], WOOD); // the speaker
    for y in 17..=20 {
        for x in 46..=52 {
            g.push((x, y, '═', WOOD));
        }
    }
    g.push((49, 13, '╲', SILVER));
    for (i, ch) in "▄▄███▄▄".chars().enumerate() {
        g.push((25 + i as i64, 4, ch, WOOD));
    }
    for k in 1..=4 {
        g.push((25 - k, 4 - k, '╲', SILVER));
        g.push((31 + k, 4 - k, '╱', SILVER));
    }
    g.extend([(21, 0, '●', SILVER), (35, 0, '●', SILVER), (5, 24, '╱', WOOD), (4, 25, '╱', WOOD), (52, 24, '╲', WOOD), (53, 25, '╲', WOOD)]);
    g
}

/// The test card: bars over a reversed strip and a bottom strip of
/// hard-edged blocks, with a circle and a crosshair across the middle.
fn draw_card(sw: usize, sh: usize) -> Vec<Vec<Card>> {
    let bars = [7u8, 6, 5, 4, 3, 2, 1];
    let castle = [(8, 1), (12, 0), (6, 3), (12, 0), (4, 5), (12, 0), (2, 7)];
    let bottom =
        [(0.17, NAVY, 5), (0.34, WHITE, 7), (0.51, PURPLE, 3), (0.68, BLACK, 0), (0.73, BLACK, 1), (0.78, BLACK, 0), (0.83, BLACK, 2), (1.0, BLACK, 0)];
    let mut card: Vec<Vec<Card>> = (0..sh)
        .map(|r| {
            (0..sw)
                .map(|c| {
                    let (u, v) = ((c as f64 + 0.5) / sw as f64, (r as f64 + 0.5) / sh as f64);
                    let bar = ((u * 7.0) as usize).min(6);
                    if v < 0.67 {
                        Card::Block(BAR_COLORS[bar], bars[bar])
                    } else if v < 0.75 {
                        let (color, level) = castle[bar];
                        Card::Block(color, level)
                    } else {
                        let &(_, color, level) = bottom.iter().find(|&&(end, ..)| u < end).unwrap_or(&bottom[7]);
                        Card::Block(color, level)
                    }
                })
                .collect()
        })
        .collect();
    // The circle, row by row: its span in each row, outlined in box drawing.
    let rr = (sh as f64 * 0.43).min(sw as f64 * 0.22) * 2.0;
    let (cx, cy) = (sw as f64 / 2.0, sh as f64 / 2.0);
    let span: Vec<Option<(usize, usize)>> = (0..sh)
        .map(|r| {
            let y = (r as f64 + 0.5 - cy) * 2.0;
            let w = rr * rr - y * y;
            (w >= 0.0).then(|| ((cx - w.sqrt() - 0.5).ceil() as usize, (cx + w.sqrt() - 0.5).floor() as usize))
        })
        .collect();
    let (mid, mc) = (sh / 2, sw / 2);
    for r in 0..sh {
        let Some((a, b)) = span[r] else { continue };
        let up = r < mid;
        let near = if up { r.checked_sub(1).and_then(|n| span[n]) } else { span.get(r + 1).copied().flatten() };
        let (na, nb) = near.unwrap_or((mc + 1, mc - 1));
        #[allow(clippy::needless_range_loop)] // the index is compared as well as read
        for c in a..=b {
            if (c > a && c < na && c < nb) || (c < b && c > nb && c > na) {
                card[r][c] = Card::Line('─');
            }
        }
        if a < na {
            card[r][a] = Card::Line(if up { '╭' } else { '╰' });
            card[r][na] = Card::Line(if up { '╯' } else { '╮' });
            card[r][b] = Card::Line(if up { '╮' } else { '╯' });
            card[r][nb] = Card::Line(if up { '╰' } else { '╭' });
        } else {
            card[r][a] = Card::Line('│');
            card[r][b] = Card::Line('│');
        }
        if near.is_none() {
            for cell in &mut card[r][a + 1..b] {
                *cell = Card::Line('─');
            }
        }
    }
    // The crosshair, meeting the circle in tees.
    if let Some((a, b)) = span[mid] {
        for cell in &mut card[mid][a + 1..b] {
            *cell = Card::Line('─');
        }
        for r in 0..sh {
            if span[r].is_some() {
                card[r][mc] = Card::Line('│');
            }
        }
        card[mid][a] = Card::Line('├');
        card[mid][b] = Card::Line('┤');
        card[mid][mc] = Card::Line('┼');
        if let Some(top) = span.iter().position(Option::is_some) {
            card[top][mc] = Card::Line('┬');
        }
        if let Some(bot) = span.iter().rposition(Option::is_some) {
            card[bot][mc] = Card::Line('┴');
        }
    }
    card
}

/// The set: its cabinet, the screen's place and size, the card, and the dots.
pub struct TvStatic {
    set: Option<Vec<(i64, i64, char, u8)>>,
    screen: (i64, i64, usize, usize),
    card: Vec<Vec<Card>>,
    dots: ColorOctants,
}

/// A set of static; reads `set` (whether to draw the cabinet round the screen).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let with_set = options.bool("set", true);
    let screen = if with_set { (4, 7, 39, 15) } else { (0, 0, META.cols, META.rows) };
    Box::new(TvStatic { set: with_set.then(draw_set), screen, card: draw_card(screen.2, screen.3), dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for TvStatic {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (x0, y0, sw, sh) = self.screen;
        let set = self.set.is_some();
        let u = t.rem_euclid(LOOP);
        let k = (t * f64::from(META.fps)).floor() as i64; // the snow is new every frame
                                                          // How firmly the set holds the signal. Just before it locks, and just
                                                          // before it is lost, the dial clicks through three stops.
        let lock = smooth(1.6, 3.4, u) * (1.0 - smooth(6.6, 7.8, u));
        let dial = (((u - 1.1) * 8.0).floor()).min(3.0 - ((u - 6.4) * 8.0).floor()).clamp(0.0, 3.0) as usize;
        let snow = 1.0 - 0.97 * lock;
        // Unlocked, the picture rolls (with its blanking bar) and its lines tear.
        let roll = (1.0 - lock).powi(2) * (sh as f64 + 2.0) * 2.5;
        let tear = (1.0 - lock) * 9.0;
        let hum = (u / LOOP + 0.45).rem_euclid(1.0) * (sh as f64 + 6.0) - 3.0; // the hum bar's middle row
        let inside = |c: usize, r: usize| {
            if !set {
                return true;
            }
            let (x, y) = ((c as f64 + 0.5 - sw as f64 / 2.0) / (sw as f64 / 2.0), (r as f64 + 0.5 - sh as f64 / 2.0) / (sh as f64 / 2.0));
            x.powi(6) + y.powi(6) <= 1.02
        };
        self.dots.clear();
        let mut lines = vec![];
        for r in 0..sh {
            let dim = 1.0 - 0.45 * (-((r as f64 - hum) / 2.2).powi(2)).exp();
            let line = 0.9 + 0.2 * hash(k, r as i64, 9); // each scan line a little brighter or darker
            let shift = (tear * (r as f64 * 0.8 + u * 9.0).sin() * hash(k >> 2, r as i64, 3)).round() as i64;
            let pr = ((r as f64 + roll) % (sh as f64 + 2.0)).floor() as usize; // the card row shown here
            for c in 0..sw {
                if !inside(c, r) {
                    continue;
                }
                let id = (r * 97 + c) as i64;
                let noisy = hash(k, id, 5) < snow;
                let (cx, cy) = (x0 + c as i64, y0 + r as i64);
                if noisy && hash(k, id, 6) >= lock.powi(6) {
                    // Snow: about half the dots lit, streaked along the line.
                    let mut prev = hash(k, id, 1);
                    for q in 0..8i64 {
                        let (dx, dy) = (q % 2, q / 2);
                        let n = hash(k, id * 8 + q, 2);
                        prev = 0.6 * n + 0.4 * prev;
                        let v = prev * line * dim;
                        if v > 0.5 {
                            self.dots.set(cx * 2 + dx, cy * 4 + dy, if v > 0.8 { SNOW_BRIGHT } else { SNOW_DIM });
                        }
                    }
                    continue;
                }
                let pc = (c as i64 + shift).rem_euclid(sw as i64) as usize;
                match if pr < sh { self.card[pr][pc] } else { Card::Block(BLACK, 0) } {
                    Card::Line(ch) => lines.push((cx, cy, ch)),
                    Card::Block(color, level) => {
                        // Once locked, what noise is left only nudges the card a shade.
                        let level = f64::from(level)
                            + if noisy {
                                if hash(k, id, 2) < 0.5 {
                                    -1.0
                                } else {
                                    1.0
                                }
                            } else {
                                0.0
                            };
                        let ink = (level / 7.0).clamp(0.0, 1.0);
                        for q in 0..8i64 {
                            let (dx, dy) = (cx * 2 + q % 2, cy * 4 + q / 2);
                            if dither(dx as usize, dy as usize, if env.paper { 1.0 - ink } else { ink }) {
                                self.dots.set(dx, dy, color);
                            }
                        }
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        for (c, r, ch) in lines {
            grid.set(c, r, ch, LINES);
        }
        if let Some(cabinet) = &self.set {
            for &(c, r, ch, color) in cabinet {
                grid.set(c, r, ch, color);
            }
            grid.set(49, 9, ['╱', '─', '╲', '│'][dial], SILVER);
        }
    }
}
