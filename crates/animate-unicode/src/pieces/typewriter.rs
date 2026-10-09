//! typewriter: types each phrase a key at a time at an uneven human pace,
//! now and then fixing a typo, holds it under a blinking cursor, deletes it,
//! types the next. After ascii.rest's typewriter.ts (MIT, @bas3line).
//!
//! The typing is ascii.rest's, slips and all, from its seeded generator. In
//! colour, as an editor shows it: the fixed prefix dim, the phrase being
//! typed bright, the cursor in the accent colour.

use crate::util::Mulberry32;
use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The typewriter's meta.
pub static META: Meta = Meta {
    name: "typewriter",
    slug: "typewriter",
    category: Category::Type,
    note: "phrases typed and deleted behind a blinking cursor, in colour",
    cols: 44,
    rows: 3,
    fps: 20,
    options: Some(r#"{"prefix": "i make ", "phrases": ["small tools.", "quiet websites.", "things that last.", "notes for later."]}"#),
    // The prefix, the phrase, the cursor.
    palette: Some(&["#8a909c", "#f4f6fa", "#f2a65a"]),
    ground: Some("#0e0f13"),
    ..Meta::DEFAULT
};

const PREFIX: u8 = 0;
const PHRASE: u8 = 1;
const CURSOR: u8 = 2;
/// Seconds a finished phrase stays, and the line waits empty before the next.
const HOLD: f64 = 2.2;
const EMPTY: f64 = 0.5;
/// Keys next to each other on a keyboard, for believable slips.
const ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];

fn slip(ch: char, rand: &mut Mulberry32) -> Option<char> {
    for row in ROWS {
        let r: Vec<char> = row.chars().collect();
        if let Some(i) = r.iter().position(|&c| c == ch) {
            let j = if i == 0 {
                1
            } else if i == r.len() - 1 || rand.next_f64() < 0.5 {
                // Only a key with a neighbour each side draws for which.
                i - 1
            } else {
                i + 1
            };
            return Some(r[j]);
        }
    }
    None
}

/// The typewriter: its prefix, the loop as a list of (time, text on the
/// line), its length, where play begins, and where the line starts.
pub struct Typewriter {
    head: String,
    events: Vec<(f64, String)>,
    period: f64,
    offset: f64,
    x: i64,
}

/// A typewriter; reads `prefix` (never deleted) and `phrases`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let head = options.string("prefix", "i make ");
    let mut list = options.list_str("phrases", &["small tools.", "quiet websites.", "things that last.", "notes for later."]);
    if list.is_empty() {
        list.push(String::new());
    }
    let room = (META.cols as i64 - 4 - head.chars().count() as i64).max(0) as usize;
    let list: Vec<Vec<char>> = list.iter().map(|p| p.chars().take(room).collect()).collect();
    let mut rand = Mulberry32::new(31);
    let (mut events, mut at, mut text, mut done) = (vec![], 0.0, String::new(), vec![]);
    let key = |next: String, wait: f64, at: &mut f64, text: &mut String, events: &mut Vec<(f64, String)>| {
        *at += wait;
        *text = next;
        events.push((*at, text.clone()));
    };
    let delay = |ch: char, r: &mut Mulberry32| {
        let base = 0.06 + r.next_f64() * 0.09;
        let space = if ch == ' ' { 0.05 + r.next_f64() * 0.08 } else { 0.0 };
        let stall = if r.next_f64() < 0.05 { 0.3 } else { 0.0 };
        base + space + stall
    };
    for phrase in &list {
        // Some phrases get one slip: a wrong key, a key or two more, a pause, then backspaces.
        let typo = if rand.next_f64() < 0.5 && phrase.len() > 5 { 3 + (rand.next_f64() * (phrase.len() - 4) as f64) as usize } else { usize::MAX };
        for i in 0..phrase.len() {
            if i == typo {
                if let Some(wrong) = slip(phrase[i], &mut rand) {
                    let more = ((rand.next_f64() * 3.0) as usize).min(phrase.len() - i - 1);
                    let d = delay(phrase[i], &mut rand);
                    key(format!("{text}{wrong}"), d, &mut at, &mut text, &mut events);
                    for k in 1..=more {
                        let d = delay(phrase[i + k], &mut rand);
                        key(format!("{text}{}", phrase[i + k]), d, &mut at, &mut text, &mut events);
                    }
                    at += 0.25 + rand.next_f64() * 0.2;
                    for k in 0..=more {
                        let mut s = text.clone();
                        s.pop();
                        key(s, if k > 0 { 0.07 } else { 0.1 }, &mut at, &mut text, &mut events);
                    }
                }
            }
            let pause = if i > 0 && ".,!?".contains(phrase[i - 1]) { 0.15 } else { 0.0 };
            let d = delay(phrase[i], &mut rand) + pause;
            key(format!("{text}{}", phrase[i]), d, &mut at, &mut text, &mut events);
        }
        done.push(at);
        // Held backspace: one press, a beat, then it repeats.
        at += HOLD;
        for k in 0..phrase.len() {
            let mut s = text.clone();
            s.pop();
            key(
                s,
                if k == 0 {
                    0.0
                } else if k == 1 {
                    0.18
                } else {
                    0.045
                },
                &mut at,
                &mut text,
                &mut events,
            );
        }
        at += EMPTY;
    }
    let width = head.chars().count() + list.iter().map(Vec::len).max().unwrap_or(0) + 1;
    // Frame 0 shows the first phrase finished, cursor lit, shortly before it is deleted.
    Box::new(Typewriter { x: ((META.cols as i64 - width as i64) / 2).max(0), head, events, period: at, offset: done[0] + HOLD - 0.75 })
}

impl Piece for Typewriter {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let now = (t + self.offset).rem_euclid(self.period);
        let (mut line, mut last) = ("", 0.0);
        for (when, s) in &self.events {
            if *when > now {
                break;
            }
            line = s;
            last = *when;
        }
        // Solid while keys land, blinking once the cursor has sat still.
        let idle = now - last;
        let lit = idle < 0.45 || (idle - 0.45).rem_euclid(1.06) > 0.53;
        let y = META.rows as i64 / 2;
        grid.clear();
        grid.text_at(self.x, y, &self.head, PREFIX);
        let x = self.x + self.head.chars().count() as i64;
        grid.text_at(x, y, line, PHRASE);
        if lit {
            grid.text_at(x + line.chars().count() as i64, y, "▌", CURSOR);
        }
    }
}
