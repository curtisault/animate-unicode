//! terminal: a shell session in a rounded frame. Each command is typed a key
//! at a time, its output prints line by line and scrolls, and a clear starts
//! over. After ascii.rest's terminal.ts (MIT, @bas3line).
//!
//! The session in a shell's colours: the prompt's path green and its sign
//! blue, the typed commands bright, their output grey, ticked and unticked
//! to-dos green and dim, the ping's figures cyan, the frame and its title
//! dim around it all.

use crate::util::Mulberry32;
use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The terminal's meta.
pub static META: Meta = Meta {
    name: "terminal",
    slug: "terminal",
    category: Category::Ui,
    note: "a shell session in colour: typed commands, output, scrolling",
    cols: 62,
    rows: 18,
    fps: 20,
    options: Some(r#"{"prompt": "~/notes $", "title": "~/notes"}"#),
    // Frame, path, prompt sign, command, output, done, not done, figures, cursor.
    palette: Some(&["#4a505c", "#7ad87a", "#5aa0f4", "#f0f2f6", "#a8aebb", "#7ad87a", "#6a707c", "#4ac8e8", "#ffffff"]),
    ground: Some("#0c0d10"),
    ..Meta::DEFAULT
};

const FRAME: u8 = 0;
const PATH: u8 = 1;
const SIGN: u8 = 2;
const COMMAND: u8 = 3;
const OUTPUT: u8 = 4;
const DONE: u8 = 5;
const UNDONE: u8 = 6;
const FIGURE: u8 = 7;
const CURSOR: u8 = 8;

/// A line of output, or a pause before the next.
enum Out {
    Line(&'static str),
    Pause(f64),
}

/// Each command: what is typed, seconds before its output, then the output.
const SESSION: [(&str, f64, &[Out]); 7] = [
    ("ls", 0.08, &[Out::Line("drafts   ideas.txt   photos   reading.txt   todo.txt")]),
    (
        "cat todo.txt",
        0.06,
        &[Out::Line("[x] water the plants"), Out::Line("[x] back up the photos"), Out::Line("[ ] fix the bike light"), Out::Line("[ ] call home on sunday")],
    ),
    ("echo \"[ ] buy stamps\" >> todo.txt", 0.05, &[]),
    ("wc -l todo.txt", 0.08, &[Out::Line("5 todo.txt")]),
    ("du -sh photos", 0.7, &[Out::Line("2.4G    photos")]),
    (
        "ping -c 3 localhost",
        0.15,
        &[
            Out::Line("PING localhost (127.0.0.1): 56 data bytes"),
            Out::Pause(0.8),
            Out::Line("64 bytes from 127.0.0.1: icmp_seq=0 ttl=64 time=0.041 ms"),
            Out::Pause(0.9),
            Out::Line("64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.067 ms"),
            Out::Pause(0.9),
            Out::Line("64 bytes from 127.0.0.1: icmp_seq=2 ttl=64 time=0.052 ms"),
            Out::Pause(0.3),
            Out::Line(""),
            Out::Line("--- localhost ping statistics ---"),
            Out::Line("3 packets transmitted, 3 received, 0% packet loss"),
            Out::Line("round-trip min/avg/max = 0.041/0.053/0.067 ms"),
        ],
    ),
    ("clear", 0.0, &[]),
];

/// A command laid out in time: when its prompt shows, each key lands, it
/// runs, and each output line prints.
struct Step {
    cmd: &'static str,
    shown: f64,
    keys: Vec<f64>,
    enter: f64,
    lines: Vec<(f64, &'static str)>,
}

/// A line on screen: a prompt and command, or a line of output.
enum Line {
    Typed(&'static str, usize),
    Output(&'static str),
}

/// The terminal: its prompt and title, the session in time, and its loop.
pub struct Terminal {
    prompt: String,
    title: String,
    steps: Vec<Step>,
    period: f64,
    start: f64,
}

/// A terminal; reads `prompt` and `title`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let mut rand = Mulberry32::new(5);
    let mut steps = vec![];
    let mut at = 0.0;
    for (cmd, wait, out) in SESSION.iter() {
        let shown = at;
        at += 0.9 + rand.next_f64() * 0.6;
        let mut keys = vec![];
        for ch in cmd.chars() {
            keys.push(at);
            let space = if ch == ' ' { rand.next_f64() * 0.12 } else { 0.0 };
            let stall = if rand.next_f64() < 0.04 { 0.35 } else { 0.0 };
            at += 0.06 + rand.next_f64() * 0.07 + space + stall;
        }
        let enter = at + 0.12;
        at = enter + wait;
        let mut lines = vec![];
        for o in out.iter() {
            match o {
                Out::Pause(p) => at += p,
                Out::Line(l) => {
                    at += 0.035;
                    lines.push((at, *l));
                }
            }
        }
        at += 0.05;
        steps.push(Step { cmd, shown, keys, enter, lines });
    }
    let period = steps.last().unwrap().enter + 0.1;
    // Frame 0 lands part way through typing the ping, with the screen most of the way full.
    let start = steps[5].keys[9];
    Box::new(Terminal { prompt: options.string("prompt", "~/notes $"), title: options.string("title", "~/notes"), steps, period, start })
}

impl Terminal {
    /// Draws a line of the session at row y, cut to the frame.
    fn line(&self, grid: &mut Grid, y: i64, line: &Line) {
        let inner = META.cols as i64 - 4;
        let mut x = 2;
        let mut put = |s: &str, c: u8, grid: &mut Grid| {
            for ch in s.chars() {
                if x - 2 < inner {
                    grid.set(x, y, ch, c);
                }
                x += 1;
            }
        };
        match *line {
            Line::Typed(cmd, n) => {
                // The prompt's path, then its sign.
                let (path, sign) = self.prompt.rsplit_once(' ').unwrap_or(("", &self.prompt));
                if !path.is_empty() {
                    put(path, PATH, grid);
                    put(" ", PATH, grid);
                }
                put(sign, SIGN, grid);
                put(" ", SIGN, grid);
                put(&cmd.chars().take(n).collect::<String>(), COMMAND, grid);
            }
            Line::Output(text) => {
                if let Some(rest) = text.strip_prefix("[x]") {
                    put("[x]", DONE, grid);
                    put(rest, OUTPUT, grid);
                } else if let Some(rest) = text.strip_prefix("[ ]") {
                    put("[ ]", UNDONE, grid);
                    put(rest, OUTPUT, grid);
                } else {
                    // Numbers in the output in their own colour.
                    for ch in text.chars() {
                        let c = if ch.is_ascii_digit() || ch == '.' && text.contains("ms") { FIGURE } else { OUTPUT };
                        put(&ch.to_string(), c, grid);
                    }
                }
            }
        }
    }
}

impl Piece for Terminal {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let now = (t + self.start).rem_euclid(self.period);
        let mut lines: Vec<Line> = vec![];
        let mut cursor: Option<(usize, usize)> = None; // line index, column
        let mut moved = 0.0; // the moment the cursor last moved, for the blink
        let head = self.prompt.chars().count() + 1;
        for s in &self.steps {
            if s.shown > now {
                break;
            }
            let typed = s.keys.iter().filter(|&&k| k <= now).count();
            lines.push(Line::Typed(s.cmd, typed));
            moved = if typed > 0 { s.keys[typed - 1] } else { s.shown };
            if now < s.enter {
                cursor = Some((lines.len() - 1, head + typed));
                break;
            }
            moved = s.enter;
            for &(k, l) in &s.lines {
                if k > now {
                    break;
                }
                lines.push(Line::Output(l));
                moved = k;
            }
            // While the command runs the cursor waits at the start of the next line.
            cursor = Some((lines.len(), 0));
        }
        let view = META.rows - 2;
        let extra = usize::from(cursor.is_some_and(|c| c.0 == lines.len()));
        let top = (lines.len() + extra).saturating_sub(view);
        // Solid while keys land, blinking once the cursor has sat still.
        let blink = (now - moved).rem_euclid(1.0) < 0.55;
        grid.clear();
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        for c in 1..cols - 1 {
            grid.set(c, 0, '─', FRAME);
            grid.set(c, rows - 1, '─', FRAME);
        }
        for r in 1..rows - 1 {
            grid.set(0, r, '│', FRAME);
            grid.set(cols - 1, r, '│', FRAME);
        }
        for (c, r, ch) in [(0, 0, '╭'), (cols - 1, 0, '╮'), (0, rows - 1, '╰'), (cols - 1, rows - 1, '╯')] {
            grid.set(c, r, ch, FRAME);
        }
        if !self.title.is_empty() {
            grid.text_at(2, 0, &format!(" {} ", self.title.chars().take(META.cols - 6).collect::<String>()), PATH);
        }
        for r in 0..view {
            if let Some(line) = lines.get(top + r) {
                self.line(grid, r as i64 + 1, line);
            }
            if let Some((i, c)) = cursor {
                if i == top + r && blink && (c as i64) < cols - 4 {
                    grid.set(2 + c as i64, r as i64 + 1, '█', CURSOR);
                }
            }
        }
    }
}
