//! vinyl: a record turning on a turntable, seen from above. The label and a
//! few specks of dust turn at 33 1/3 rpm, the sheen on the grooves stays
//! put, and a J-shaped tonearm rests in the outer grooves. After
//! ascii.rest's vinyl.ts (MIT, @bas3line).
//!
//! The record, its label and sheen, the plinth and the tonearm are
//! ascii.rest's. The record is braille: its grooves are rings a dot apart,
//! with the smooth bands between tracks left bare, and the two wedges of
//! sheen fill in the dots between the rings, so the light lies across the
//! grooves as the record turns beneath it. In colour the grooves are a deep
//! black-blue and the sheen silver, the label is red with its print cut
//! dark out of it, and the turntable is walnut with a silver arm.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// The record's meta.
pub static META: Meta = Meta {
    name: "vinyl",
    slug: "vinyl",
    category: Category::Objects,
    note: "a braille record turning under a silver arm, a red label, sheen",
    cols: 64,
    rows: 25,
    fps: 24,
    charset: Charset::Extended,
    // The grooves, the sheen, the label; the platter's rim; dust;
    // the plinth; the arm.
    palette: Some(&["#2e3448", "#c8d0e0", "#d8343c", "#5a5e6c", "#b0b4c0", "#7a5a3a", "#c8ccd8"]),
    ground: Some("#0b0a0a"),
    ..Meta::DEFAULT
};

const GROOVE: u8 = 0;
const SHEEN: u8 = 1;
const LABEL_RED: u8 = 2;
const RIM_EDGE: u8 = 3;
const DUST_SPECK: u8 = 4;
const PLINTH: u8 = 5;
const ARM: u8 = 6;
/// 33 1/3 rpm in radians a second; the record, the platter's edge and the
/// label, in rows; the smooth bands between tracks; and the dust (radius, angle).
const SPIN: f64 = std::f64::consts::TAU * 100.0 / 3.0 / 60.0;
const R: f64 = 9.0;
const RIM: f64 = 9.9;
const LABEL: f64 = 4.0;
const GAPS: [f64; 2] = [6.2, 7.5];
const DUST: [(f64, f64); 3] = [(6.8, 0.4), (8.2, 2.9), (5.4, 4.4)];

/// The label's print in its own frame (rows, x across): a title band above
/// the spindle and a round mark off to one side below it.
fn printed(x: f64, y: f64) -> bool {
    (y > -2.9 && y < -1.45 && x.abs() < 2.7) || (x - 1.5).hypot(y - 1.9) < 0.75
}

/// Light bars: two opposite wedges where the grooves catch a lamp to the
/// upper left. They stay put while the record turns under them.
fn sheen(th: f64) -> f64 {
    let d = (th + 0.8).rem_euclid(std::f64::consts::PI) - std::f64::consts::FRAC_PI_2;
    (-(d * 2.4).powi(2)).exp()
}

/// The turntable's dots.
pub struct Vinyl {
    dots: ColorDots,
}

/// A record on a turntable; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Vinyl { dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Vinyl {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        let (cx, cy) = (22.5, 12.5); // the spindle, in cells
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        let a = SPIN * t;
        let (sa, ca) = a.sin_cos();
        self.dots.clear();
        for j in 0..h {
            for i in 0..w {
                let (dx, dy) = (((i as f64 + 0.5) / 2.0 - cx) / 2.0, (j as f64 + 0.5) / 4.0 - cy);
                let d = dx.hypot(dy);
                if !(0.4..=RIM).contains(&d) {
                    continue;
                }
                if d > R {
                    // The platter's rim, a fine ring.
                    if (d - RIM).abs() < 0.13 {
                        self.dots.set(i, j, RIM_EDGE);
                    }
                    continue;
                }
                let th = dy.atan2(dx);
                if d < LABEL {
                    // Turn the dot back into the label's frame: the print is
                    // left out of the red, so it shows dark in colour and in one ink.
                    let (x, y) = (dx * ca + dy * sa, dy * ca - dx * sa);
                    if !printed(x, y) {
                        self.dots.set(i, j, LABEL_RED);
                    }
                    continue;
                }
                // Grooves: rings a dot apart, bare in the bands between tracks.
                if GAPS.iter().any(|&g| (d - g).abs() < 0.22) || (d - LABEL).abs() < 0.3 {
                    continue;
                }
                let s = sheen(th);
                let ring = ((d * 4.0) as i64) % 2 == 0;
                if ring || s > 0.45 {
                    self.dots.set(i, j, if s > 0.45 { SHEEN } else { GROOVE });
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        grid.set(cx as i64, cy as i64, '●', RIM_EDGE);
        // Dust riding round with the record.
        for &(r, p) in &DUST {
            grid.set((cx + 2.0 * r * (p + a).cos()).floor() as i64, (cy + r * (p + a).sin()).floor() as i64, '°', DUST_SPECK);
        }
        // The plinth, its start and speed buttons, and the pitch slider.
        grid.text_at(0, 0, &format!("╭{}╮", "─".repeat(META.cols - 2)), PLINTH);
        grid.text_at(0, rows - 1, &format!("╰{}╯", "─".repeat(META.cols - 2)), PLINTH);
        for r in 1..rows - 1 {
            grid.set(0, r, '│', PLINTH);
            grid.set(cols - 1, r, '│', PLINTH);
        }
        for (c, r, s) in [(2, 21, "┌──┐"), (2, 22, "└──┘"), (47, 21, "┌┐┌┐"), (47, 22, "└┘└┘")] {
            grid.text_at(c, r, s, PLINTH);
        }
        for r in 12..=22 {
            grid.set(59, r, if r == 17 { '═' } else { '┊' }, PLINTH);
        }
        // The tonearm: counterweight behind the pivot, the pivot in its ring, a
        // tube down and round to the headshell, and the cue lever beside it.
        for (c, r, s) in [
            (49, 2, "▗▄▄▄▖"),
            (49, 3, "▝▀█▀▘"),
            (48, 4, "╭──╨──╮"),
            (48, 5, "│  ●  │"),
            (48, 6, "╰──╥──╯"),
            (41, 16, "══════════╝"),
            (36, 16, "▐███▌"),
            (54, 9, "╭╮"),
            (54, 10, "││"),
            (54, 11, "╰╯"),
        ] {
            grid.text_at(c, r, s, ARM);
        }
        for r in 7..16 {
            grid.set(51, r, '║', ARM);
        }
    }
}
