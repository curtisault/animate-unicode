//! morse: a signal lamp keys out a message. The marks run off along a tape
//! from the head, and each letter is written beneath its group once it is
//! complete. After ascii.rest's morse.ts (MIT, @bas3line).
//!
//! The lamp, the tape and the timing are ascii.rest's, glyph for glyph. What
//! it adds is the light: the lens glows from amber at its rim to white at its
//! centre while it is keyed, the rays are yellow, the marks on the tape are
//! the lamp's amber, and each letter is set bright once it is decoded.

use crate::{Category, Env, Grid, Meta, Options, Piece, Ramp};

/// Morse's meta.
pub static META: Meta = Meta {
    name: "morse",
    slug: "morse",
    category: Category::Type,
    note: "a signal lamp glowing amber as it keys morse onto a tape",
    cols: 64,
    rows: 19,
    fps: 15,
    options: Some(r#"{"text": "hello world"}"#),
    // The lens, dark to white; the housing; the rays; the marks; the tape; the letters.
    palette: Some(&["#3a2a14", "#7a4e1a", "#c87a24", "#f0a83a", "#ffd27a", "#fff4dc", "#6a6e78", "#ffe07a", "#f0a83a", "#4a4e58", "#f4f6fa"]),
    ground: Some("#0c0b0a"),
    ..Meta::DEFAULT
};

const LENS: Ramp = Ramp::new(0, 6);
const HOUSING: u8 = 6;
const RAY: u8 = 7;
const MARK: u8 = 8;
const TAPE: u8 = 9;
const LETTER: u8 = 10;
/// Seconds per dot.
const UNIT: f64 = 0.12;
const RAMP: [char; 7] = [' ', '·', ':', '░', '▒', '▓', '█'];

fn code(ch: char) -> Option<&'static str> {
    Some(match ch {
        'A' => ".-",
        'B' => "-...",
        'C' => "-.-.",
        'D' => "-..",
        'E' => ".",
        'F' => "..-.",
        'G' => "--.",
        'H' => "....",
        'I' => "..",
        'J' => ".---",
        'K' => "-.-",
        'L' => ".-..",
        'M' => "--",
        'N' => "-.",
        'O' => "---",
        'P' => ".--.",
        'Q' => "--.-",
        'R' => ".-.",
        'S' => "...",
        'T' => "-",
        'U' => "..-",
        'V' => "...-",
        'W' => ".--",
        'X' => "-..-",
        'Y' => "-.--",
        'Z' => "--..",
        '0' => "-----",
        '1' => ".----",
        '2' => "..---",
        '3' => "...--",
        '4' => "....-",
        '5' => ".....",
        '6' => "-....",
        '7' => "--...",
        '8' => "---..",
        '9' => "----.",
        '.' => ".-.-.-",
        ',' => "--..--",
        '?' => "..--..",
        '!' => "-.-.--",
        '\'' => ".----.",
        '/' => "-..-.",
        '-' => "-....-",
        '&' => ".-...",
        _ => return None,
    })
}

/// The message in dot units: when each mark is on, where each letter sits,
/// the loop's length, and how much is already sent at frame 0.
pub struct Morse {
    on: Vec<bool>,
    letters: Vec<(char, i64, i64)>,
    l: i64,
    t0: f64,
}

impl Morse {
    fn lit(&self, x: i64) -> bool {
        self.on.get(x.rem_euclid(self.l) as usize).copied().unwrap_or(false)
    }
}

/// Morse; reads `text`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (mut on, mut letters, mut at) = (vec![], vec![], 0usize);
    for ch in options.str("text", "hello world").to_uppercase().chars() {
        let Some(c) = code(ch) else {
            at += 4; // a word gap is seven, three already follow the letter before
            continue;
        };
        let start = at;
        for m in c.chars() {
            let len = if m == '.' { 1 } else { 3 };
            if on.len() < at + len {
                on.resize(at + len, false);
            }
            on[at..at + len].fill(true);
            at += len + 1;
        }
        letters.push((ch, start as i64, at as i64 - 1));
        at += 2;
    }
    let l = (at as i64 + 12).max(20); // a long rest before it starts again
    let mut m = Morse { on, letters, l, t0: 0.0 };
    // Play already sent at frame 0, chosen so the lamp is lit and the tape full.
    let mut warm = 45.min(l - 1);
    while !m.lit(warm) && warm < l * 2 {
        warm += 1;
    }
    m.t0 = (warm as f64 + 0.5) * UNIT;
    Box::new(m)
}

impl Piece for Morse {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        let now = (t + self.t0) / UNIT;
        let unit = now.floor() as i64;
        // Full while keyed, with a brief afterglow as the shutter closes.
        let glow = if self.lit(unit) {
            1.0
        } else if self.lit(unit - 1) && now - (unit as f64) < 0.5 {
            0.35
        } else {
            0.0
        };
        let (cx, cy) = (31i64, 6i64); // the lamp's centre
        let head = cols - 7; // where marks land on the tape
        let tape = 16;
        grid.clear();
        let put = |grid: &mut Grid, r: i64, c: i64, s: &str, color: u8| grid.text_at(c, r, s, color);
        // The lamp face: a lens, a dark gap, then the housing ring.
        for r in 0..12 {
            for c in cx - 10..=cx + 10 {
                let (dx, dy) = ((c - cx) as f64 / 2.0, (r - cy) as f64);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= 2.75 {
                    // Lit, the glass is brightest at the centre; dark, only a glint shows.
                    let glint = if dx < -0.3 && dy < -0.3 && (d - 1.9).abs() < 0.45 { 0.34 } else { 0.0 };
                    let b = if glow > 0.0 { glow * (0.62 + 0.38 * (1.0 - (d / 2.75).powi(2))) } else { glint };
                    let i = (b * 6.0).round() as usize;
                    grid.set(c, r, RAMP[if env.paper { 6 - i } else { i }], LENS.at(b));
                } else if d > 3.2 && d <= 4.15 {
                    grid.set(c, r, if d > 3.8 { '▓' } else { '█' }, HOUSING);
                }
            }
        }
        // Rays while it shines.
        if glow > 0.0 {
            for k in 0..if glow == 1.0 { 3 } else { 1 } {
                put(grid, cy, cx - 13 - 2 * k, "─", RAY);
                put(grid, cy, cx + 13 + 2 * k, "─", RAY);
                put(grid, cy - 5 - k, cx, "│", RAY);
                put(grid, cy - 4 - k, cx - 6 - k, "╲", RAY);
                put(grid, cy - 4 - k, cx + 6 + k, "╱", RAY);
            }
        }
        // The yoke it swings in, a post and a foot.
        for r in cy..cy + 5 {
            put(grid, r, cx - 10, "│", HOUSING);
            put(grid, r, cx + 10, "│", HOUSING);
        }
        put(grid, cy, cx - 10, "o", HOUSING);
        put(grid, cy, cx + 10, "o", HOUSING);
        put(grid, cy + 5, cx - 10, &format!("╰{}┬{}╯", "─".repeat(9), "─".repeat(9)), HOUSING);
        put(grid, cy + 6, cx, "│", HOUSING);
        put(grid, cy + 7, cx - 5, "▄▄▄▄█▀█▄▄▄▄", HOUSING);
        // The tape: marks behind the head, blank paper ahead of it.
        put(grid, tape - 1, 0, &"─".repeat(META.cols), TAPE);
        put(grid, tape + 1, 0, &"─".repeat(META.cols), TAPE);
        for c in 0..=head {
            if self.lit(unit - (head - c)) {
                grid.set(c, tape, '█', MARK);
            }
        }
        put(grid, tape - 1, head, "┬", TAPE);
        put(grid, tape + 1, head, "┴", TAPE);
        // Each letter under the middle of its group once its last mark is sent.
        let lap = unit.div_euclid(self.l);
        for k in lap - 1..=lap {
            for &(ch, s, e) in &self.letters {
                if k * self.l + e > unit - 1 {
                    continue;
                }
                let x = head - (unit - (k as f64 * self.l as f64 + (s + e) as f64 / 2.0).round() as i64);
                if (0..cols).contains(&x) && tape + 2 < rows {
                    grid.set(x, tape + 2, ch, LETTER);
                }
            }
        }
    }
}
