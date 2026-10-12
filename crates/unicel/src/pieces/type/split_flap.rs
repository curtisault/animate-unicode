//! split-flap: an airport departures board. When a flight leaves, every row
//! moves up and each flap riffles through its drum, one card at a time, to
//! land. After ascii.rest's split-flap.ts (MIT, @bas3line).
//!
//! The board and its flaps are ascii.rest's, card for card, falling cards
//! hanging over the split in half blocks. In a departures hall's colours:
//! cream cards on a dark board, and each remark in the colour that tells a
//! traveller what to do, amber for a delay, red for a cancellation, green
//! for boarding, cyan for a gate opening.

use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The board's meta.
pub static META: Meta = Meta {
    name: "split-flap",
    slug: "split-flap",
    category: Category::Type,
    note: "a departures board riffling to each new line, remarks in colour",
    cols: 61,
    rows: 18,
    fps: 15,
    options: Some(
        r#"{"title": "departures", "start": "14:03", "flights": [["14:05", "lisbon", "a4", "on time"], ["14:15", "oslo", "b12", "on time"], ["14:20", "nairobi", "c7", "delayed"], ["14:35", "lima", "a9", "on time"], ["14:40", "reykjavik", "d21", "on time"], ["14:50", "seoul", "b3", "on time"], ["15:00", "marrakesh", "c14", "cancelled"], ["15:05", "helsinki", "a2", "on time"], ["15:20", "auckland", "d8", "on time"], ["15:25", "kyoto", "b16", "delayed"], ["15:35", "cape town", "c11", "on time"], ["15:45", "montreal", "a6", "on time"]]}"#,
    ),
    // The board, labels, cards, the title, a card turning; remarks: delayed,
    // cancelled, boarding, gate open.
    palette: Some(&["#4a4f5a", "#7a808c", "#f4ead0", "#f2b44a", "#9aa0aa", "#f2a640", "#ef5a5a", "#6ad87a", "#5ac0e8"]),
    ground: Some("#111216"),
    ..Meta::DEFAULT
};

const BOARD: u8 = 0;
const LABEL: u8 = 1;
const CARD: u8 = 2;
const TITLE: u8 = 3;
const TURNING: u8 = 4;
/// The cards on each kind of flap's drum, in the order they turn up.
const TEXT: &str = " ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.-/'&";
const DIGITS: &str = " 0123456789";
/// Seconds per board, the first flip after a settled first frame, cards a
/// second, and rows of flights shown.
const STEP: f64 = 3.0;
const LEAD: f64 = 0.8;
const RATE: f64 = 30.0;
const SHOWN: usize = 6;

fn hash(a: i64, b: i64, c: i64) -> f64 {
    let mut h = (a as u32).wrapping_mul(374_761_393).wrapping_add((b as u32).wrapping_mul(668_265_263)).wrapping_add((c as u32).wrapping_mul(2_246_822_519));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn minutes(s: &str) -> Option<i64> {
    let (h, m) = s.trim().split_once(':')?;
    (h.len() <= 2 && m.len() == 2).then_some(())?;
    Some(h.parse::<i64>().ok()? * 60 + m.parse::<i64>().ok()?)
}

fn hhmm(m: i64) -> String {
    let m = m.rem_euclid(1440);
    format!("{:02}{:02}", m / 60, m % 60)
}

/// What a run of flaps shows on a board.
#[derive(Clone, Copy)]
enum Field {
    ClockHours,
    ClockMinutes,
    Hours(usize),
    Minutes(usize),
    Destination(usize),
    Gate(usize),
    Remark(usize),
}

/// The board: its flights, their times, the lap, the opening clock, every
/// run of flaps (row, column, tiles, drum, what it shows), and the fixed parts.
pub struct SplitFlap {
    list: Vec<[String; 4]>,
    at: Vec<i64>,
    lap: i64,
    opening: Option<i64>,
    fields: Vec<(i64, i64, usize, &'static str, Field)>,
    base: Grid,
}

impl SplitFlap {
    fn minute(&self, j: i64) -> i64 {
        let n = self.list.len() as i64;
        self.at[j.rem_euclid(n) as usize] + j.div_euclid(n) * self.lap
    }

    fn flight(&self, j: i64) -> &[String; 4] {
        &self.list[j.rem_euclid(self.list.len() as i64) as usize]
    }

    /// Board s lists flights from s / 2. On even boards one has just left; on
    /// odd ones the clock has moved on to just before the next goes.
    fn first(s: i64) -> i64 {
        s.div_euclid(2)
    }

    fn clock(&self, s: i64) -> i64 {
        match self.opening {
            Some(o) if s == 1 => o,
            _ if s.rem_euclid(2) == 1 => self.minute(Self::first(s)) - 2,
            _ => self.minute(Self::first(s) - 1) + 1,
        }
    }

    fn remark(&self, s: i64, i: usize) -> String {
        let said = self.flight(Self::first(s) + i as i64)[3].to_uppercase().trim().to_string();
        if !said.is_empty() && said != "ON TIME" {
            return said;
        }
        let wait = self.minute(Self::first(s) + i as i64) - self.clock(s);
        (if wait <= 3 {
            "LAST CALL"
        } else if wait <= 12 {
            "BOARDING"
        } else if wait <= 25 {
            "GATE OPEN"
        } else {
            "ON TIME"
        })
        .to_string()
    }

    fn text(&self, f: Field, s: i64) -> String {
        let live = |i: usize| i < self.list.len();
        let row = |i: usize| Self::first(s) + i as i64;
        match f {
            Field::ClockHours => hhmm(self.clock(s))[..2].to_string(),
            Field::ClockMinutes => hhmm(self.clock(s))[2..].to_string(),
            Field::Hours(i) if live(i) => hhmm(self.minute(row(i)))[..2].to_string(),
            Field::Minutes(i) if live(i) => hhmm(self.minute(row(i)))[2..].to_string(),
            Field::Destination(i) if live(i) => self.flight(row(i))[1].to_uppercase(),
            Field::Gate(i) if live(i) => self.flight(row(i))[2].to_uppercase(),
            Field::Remark(i) if live(i) => self.remark(s, i),
            _ => String::new(),
        }
    }
}

fn remark_color(r: &str) -> u8 {
    match r {
        "DELAYED" => 5,
        "CANCELLED" => 6,
        "BOARDING" | "LAST CALL" => 7,
        "GATE OPEN" => 8,
        _ => CARD,
    }
}

/// A split-flap board; reads `title`, `start` (the first board's clock) and
/// `flights` (each `[time, destination, gate, remark]`).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let parse = |v: &serde_json::Value| -> Option<[String; 4]> {
        let a = v.as_array()?;
        let s = |i: usize| a.get(i).and_then(|x| x.as_str()).unwrap_or("").to_string();
        minutes(&s(0))?;
        Some([s(0), s(1), s(2), s(3)])
    };
    let mut list: Vec<[String; 4]> = options.json("flights").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(parse).collect()).unwrap_or_default();
    if list.is_empty() {
        let d: serde_json::Value = serde_json::from_str(META.options.unwrap()).unwrap();
        list = d["flights"].as_array().unwrap().iter().filter_map(parse).collect();
    }
    let n = list.len();
    // Times run forward through the list, past midnight if they wrap, and the
    // list comes round again one lap later.
    let mut at: Vec<i64> = vec![];
    for f in &list {
        let mut m = minutes(&f[0]).unwrap_or(0);
        while at.last().is_some_and(|&l| m < l) {
            m += 1440;
        }
        at.push(m);
    }
    let lap = at[n - 1] - at[0] + if n > 1 { (((at[n - 1] - at[0]) as f64 / (n - 1) as f64).round() as i64).max(5) } else { 30 };
    let cols = META.cols as i64;
    let mut fields = vec![(1, cols - 11, 2, DIGITS, Field::ClockHours), (1, cols - 5, 2, DIGITS, Field::ClockMinutes)];
    for i in 0..SHOWN {
        let r = 5 + i as i64 * 2;
        fields.extend([
            (r, 2, 2, DIGITS, Field::Hours(i)),
            (r, 8, 2, DIGITS, Field::Minutes(i)),
            (r, 14, 9, TEXT, Field::Destination(i)),
            (r, 34, 3, TEXT, Field::Gate(i)),
            (r, 42, 9, TEXT, Field::Remark(i)),
        ]);
    }
    // The fixed parts: frame, labels, colons and the split beneath each tile.
    let rows = META.rows as i64;
    let mut base = Grid::new(META.cols, META.rows);
    for r in 0..rows {
        base.set(0, r, '│', BOARD);
        base.set(cols - 1, r, '│', BOARD);
    }
    for (r, l, rt) in [(0, '╭', '╮'), (3, '├', '┤'), (rows - 1, '╰', '╯')] {
        base.text_at(0, r, &format!("{l}{}{rt}", "─".repeat(META.cols - 2)), BOARD);
    }
    let title: String = options.str("title", "departures").to_uppercase().chars().take(18).map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    base.text_at(2, 1, &title, TITLE);
    for (c, l) in [(2, "time"), (14, "destination"), (34, "gate"), (42, "remarks")] {
        base.text_at(c, 4, l, LABEL);
    }
    for &(r, c, len, _, _) in &fields {
        for i in 0..len {
            base.set(c + 2 * i as i64, r + 1, '▔', BOARD);
        }
    }
    base.set(cols - 7, 1, ':', CARD);
    for i in 0..SHOWN.min(n) {
        base.set(6, 5 + i as i64 * 2, ':', CARD);
    }
    Box::new(SplitFlap { list, at, lap, opening: minutes(options.str("start", "14:03")), fields, base })
}

impl Piece for SplitFlap {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let v = t - LEAD + STEP;
        let k = (v / STEP).floor() as i64;
        let since = v - k as f64 * STEP;
        let s = 1 + k;
        grid.cells_mut().copy_from_slice(self.base.cells());
        grid.colors_mut().copy_from_slice(self.base.colors());
        for &(r, c, len, drum, field) in &self.fields {
            let pad = |x: String| -> Vec<char> { format!("{x:<len$}").chars().take(len).collect() };
            let (to_s, from_s) = (self.text(field, s), self.text(field, s - 1));
            let color = if let Field::Remark(_) = field { remark_color(to_s.trim()) } else { CARD };
            let (to, from) = (pad(to_s), pad(from_s));
            let drum_c: Vec<char> = drum.chars().collect();
            for i in 0..len {
                let pos = |ch: char| drum_c.iter().position(|&d| d == ch).unwrap_or(0);
                let (a, b) = (pos(from[i]), pos(to[i]));
                let mut ch = drum_c[b];
                let x = c + 2 * i as i64;
                if a != b {
                    // Each flap starts a touch after its neighbour and turns forward
                    // only; while it turns, the falling card hangs over the split.
                    let begin = r as f64 * 0.02 + x as f64 * 0.004 + hash(r, c + i as i64, s) * 0.12;
                    let flips = ((since - begin).max(0.0) * RATE).floor() as usize;
                    let need = (b + drum_c.len() - a) % drum_c.len();
                    if flips < need {
                        ch = drum_c[(a + flips) % drum_c.len()];
                        if since >= begin {
                            grid.set(x, r + 1, '▀', TURNING);
                        }
                    }
                }
                grid.set(x, r, ch, color);
            }
        }
    }
}
