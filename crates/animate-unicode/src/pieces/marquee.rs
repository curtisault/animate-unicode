//! marquee: a theatre sign. Its text scrolls across an LED panel a pixel at
//! a time and loops without a seam, while the bulbs round the edge chase.
//! After ascii.rest's marquee.ts (MIT, @bas3line).
//!
//! The font, the scroll and the chase are ascii.rest's. In braille every LED
//! is one dot with a gap round it, so the panel looks like a real dot-matrix
//! sign rather than blocks, lit amber; the bulbs are round, warm yellow when
//! lit and dull when dark, inside a gold frame.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// The marquee's meta.
pub static META: Meta = Meta {
    name: "marquee",
    slug: "marquee",
    category: Category::Type,
    note: "a scrolling sign of amber braille leds, chasing bulbs round it",
    cols: 61,
    rows: 10,
    fps: 24,
    charset: Charset::Extended,
    options: Some(r#"{"text": "now showing · doors open at 7:30 ·", "speed": 12}"#),
    // The leds, a bulb lit, a bulb dark, the frame.
    palette: Some(&["#ffa040", "#ffe08a", "#5a4a2a", "#c8a050"]),
    ground: Some("#120c08"),
    ..Meta::DEFAULT
};

const LED: u8 = 0;
const LIT: u8 = 1;
const DARK: u8 = 2;
const FRAME: u8 = 3;
/// 5×7 glyphs: the character, then one base-32 digit per row (bit 4 is the left pixel).
const PACKED: &str = concat!(
    "0ehjlphe14c4444e2eh1248v3v2421he426aiv225vgu11he668guhhe7v1248888ehhehhe9ehhf12c",
    "AehhvhhhBuhhuhhuCehgggheDuhhhhhuEvgguggvFvggugggGehgnhhfHhhhvhhhIe44444eJ72222ic",
    "KhikokihLggggggvMhrllhhhNhhpljhhOehhhhhePuhhugggQehhhlidRuhhukihSfgge11uTv444444",
    "UhhhhhheVhhhhha4WhhhlllaXhha4ahhYhha4444Zv1248gv.0000004,0000048:0040040!4444404",
    "?eh12404-000e000'4400000\"aa00000/11248gg&cik8lid+044v440·0004000*04lel40#aavavaa",
    "(2488842)8422248@ehnlngf%op248j3=00v0v00_000000v 0000000",
);

/// A glyph's seven rows, from the packed font.
pub(crate) fn font(ch: char) -> Option<[u8; 7]> {
    let chars: Vec<char> = PACKED.chars().collect();
    chars.chunks(8).find(|g| g[0] == ch).map(|g| std::array::from_fn(|i| g[i + 1].to_digit(32).unwrap_or(0) as u8))
}

/// The text as a strip of pixel columns, each a 7-bit mask with bit 0 at the top.
fn strip(text: &str) -> Vec<u8> {
    let mut out = vec![];
    for ch in text.to_uppercase().chars() {
        let g = font(ch).or_else(|| font('?')).unwrap_or([0; 7]);
        let col = |x: usize| g.iter().enumerate().fold(0u8, |m, (y, &b)| if b & (16 >> x) != 0 { m | 1 << y } else { m });
        let cols: Vec<u8> = (0..5).map(col).collect();
        if ch == ' ' {
            out.extend([0, 0, 0]);
        } else {
            // Keep the glyph's inked columns and any gaps between them.
            let first = cols.iter().position(|&m| m != 0);
            let last = cols.iter().rposition(|&m| m != 0);
            if let (Some(a), Some(b)) = (first, last) {
                out.extend(&cols[a..=b]);
            }
        }
        out.push(0);
    }
    out
}

/// The sign: its loop of pixel columns, its speed, the bulbs, the frame drawn once, and the dots.
pub struct Marquee {
    lp: Vec<u8>,
    speed: f64,
    bulbs: Vec<(i64, i64)>,
    base: Grid,
    dots: ColorDots,
}

/// A marquee; reads `text` and `speed` (pixels a second).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let width = (cols - 8) as usize;
    // Pad the loop with a gap, then repeat it until it is wider than the panel.
    let mut unit = strip(options.str("text", "now showing · doors open at 7:30 ·").trim());
    unit.extend([0; 12]);
    let mut lp = unit.clone();
    while lp.len() < width + 1 {
        lp.extend(&unit);
    }
    // Bulbs every other cell along the top and bottom, every row down the
    // sides, numbered clockwise so a chase can run round them.
    let mut bulbs = vec![];
    bulbs.extend((0..cols).step_by(2).map(|c| (0, c)));
    bulbs.extend((1..rows - 1).map(|r| (r, cols - 1)));
    bulbs.extend((0..cols).rev().step_by(2).map(|c| (rows - 1, c)));
    bulbs.extend((1..rows - 1).rev().map(|r| (r, 0)));
    let mut base = Grid::new(META.cols, META.rows);
    base.text_at(2, 1, &format!("╔{}╗", "═".repeat(META.cols - 6)), FRAME);
    base.text_at(2, rows - 2, &format!("╚{}╝", "═".repeat(META.cols - 6)), FRAME);
    for r in 2..rows - 2 {
        base.set(2, r, '║', FRAME);
        base.set(cols - 3, r, '║', FRAME);
    }
    Box::new(Marquee { lp, speed: options.f64("speed", 12.0), bulbs, base, dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Marquee {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (left, width) = (4i64, META.cols as i64 - 8);
        let l = self.lp.len();
        let off = (t * self.speed).floor() as usize % l;
        self.dots.clear();
        // An led a column and every other dot down: each pixel a dot with a
        // gap round it, the letters starting a dot below row 3's top.
        for j in 0..width {
            let m = self.lp[(off + j as usize) % l];
            for y in 0..7 {
                if m >> y & 1 == 1 {
                    self.dots.set((left + j) * 2, 3 * 4 + 1 + 2 * y, LED);
                }
            }
        }
        grid.cells_mut().copy_from_slice(self.base.cells());
        grid.colors_mut().copy_from_slice(self.base.colors());
        self.dots.draw(grid, 0, 0);
        // Two of every three bulbs lit, the dark one walking clockwise.
        let step = (t * 7.0).floor() as i64;
        for (i, &(r, c)) in self.bulbs.iter().enumerate() {
            let dark = (i as i64 - step).rem_euclid(3) == 0;
            grid.set(c, r, if dark { '·' } else { '●' }, if dark { DARK } else { LIT });
        }
    }
}
