//! form controls: a settings form drawn the way a terminal app draws one. A
//! focus marker tabs down through a text input, a select, radio buttons, a
//! checkbox, a switch and a slider, changes each, saves, then changes them
//! back. After ascii.rest's form-controls.ts (MIT, @bas3line).
//!
//! The controls are the ones a terminal toolkit draws with Unicode: radio
//! buttons `○ ◉`, checkboxes `□ ▣`, a slider with a round knob `●`. The
//! field with focus is amber, whatever is chosen is cyan, and the save
//! button turns green when it has saved.

use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// The form's meta.
pub static META: Meta = Meta {
    name: "form controls",
    slug: "form-controls",
    category: Category::Ui,
    note: "a terminal form with ○ ◉ □ ▣ controls, focus amber, choices cyan",
    cols: 54,
    rows: 16,
    fps: 15,
    charset: Charset::Extended,
    options: Some(r#"{"title": "preferences"}"#),
    // Border, labels, values, focus, chosen, saved, the title.
    palette: Some(&["#4a505c", "#9aa0aa", "#e5e9f0", "#ffb44a", "#4ac8e8", "#7ad87a", "#ffffff"]),
    ground: Some("#0e0f13"),
    ..Meta::DEFAULT
};

const BORDER: u8 = 0;
const LABEL: u8 = 1;
const VALUE: u8 = 2;
const FOCUS: u8 = 3;
const CHOSEN: u8 = 4;
const SAVED: u8 = 5;
const TITLE: u8 = 6;
const EMAILS: [&str; 2] = ["you@example.com", "team@example.com"];
const UNITS: [&str; 2] = ["metric", "imperial"];
const THEMES: [&str; 3] = ["light", "dark", "system"];
const VOLUME: [f64; 2] = [40.0, 75.0];
/// Columns: labels, controls, input width inside its brackets; one past the
/// right edge every control lines up on; slider cells.
const LABEL_X: i64 = 4;
const FIELD: i64 = 14;
const WIDE: usize = 32;
const RIGHT: i64 = FIELD + WIDE as i64 + 4;
const TRACK: usize = 30;

/// The form's state at a moment.
#[derive(Clone)]
struct State {
    focus: usize,
    email: String,
    typed: f64,
    unit: usize,
    open: bool,
    hi: usize,
    theme: usize,
    mentions: bool,
    knob: usize,
    volume: f64,
    pressed: bool,
    saved: bool,
}

/// A timed change to the state.
enum Change {
    Focus(usize),
    Email(String, f64),
    Open(usize),
    Hi(usize),
    Unit(usize),
    Theme(usize),
    Mentions(bool),
    Knob(usize),
    Volume(f64),
    Press,
    Save,
    Unsave,
}

/// The form: its title, the tour as timed changes, its length and where play begins.
pub struct FormControls {
    title: String,
    ev: Vec<(f64, Change)>,
    total: f64,
    start: f64,
}

/// Form controls; reads `title`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    // The tour of the form as timed changes to its state. Two passes, the
    // second undoing the first, so the loop closes on the starting state.
    let (mut ev, mut t, mut start) = (vec![], 0.0, 0.0);
    let mut at = |dt: f64, c: Change, ev: &mut Vec<(f64, Change)>| {
        t += dt;
        ev.push((t, c));
        t
    };
    for b in [1usize, 0] {
        let a = 1 - b;
        at(0.35, Change::Focus(0), &mut ev);
        let now = at(0.3, Change::Email(String::new(), 0.0), &mut ev);
        if let Some(last) = ev.last_mut() {
            last.1 = Change::Email(String::new(), now);
        }
        for i in 1..=EMAILS[b].len() {
            let when = ev.last().unwrap().0 + 0.07;
            at(0.07, Change::Email(EMAILS[b][..i].to_string(), when), &mut ev);
        }
        if b == 1 {
            start = ev.last().unwrap().0 + 0.2;
        }
        at(0.9, Change::Focus(1), &mut ev);
        at(0.3, Change::Open(a), &mut ev);
        at(0.4, Change::Hi(b), &mut ev);
        at(0.35, Change::Unit(b), &mut ev);
        at(0.45, Change::Focus(2), &mut ev);
        at(0.35, Change::Theme(if b == 1 { 1 } else { 0 }), &mut ev);
        at(0.45, Change::Focus(3), &mut ev);
        at(0.35, Change::Mentions(b == 1), &mut ev);
        at(0.45, Change::Focus(4), &mut ev);
        for k in 1..=4 {
            at(if k == 1 { 0.35 } else { 0.07 }, Change::Knob(if b == 1 { k } else { 4 - k }), &mut ev);
        }
        at(0.45, Change::Focus(5), &mut ev);
        for k in 1..=7 {
            let v = VOLUME[a] + (VOLUME[b] - VOLUME[a]) * k as f64 / 7.0;
            at(if k == 1 { 0.35 } else { 0.07 }, Change::Volume(v), &mut ev);
        }
        at(0.45, Change::Focus(6), &mut ev);
        at(0.4, Change::Press, &mut ev);
        at(0.18, Change::Save, &mut ev);
        at(1.3, Change::Unsave, &mut ev);
    }
    let total = ev.last().unwrap().0 + 0.4;
    Box::new(FormControls { title: options.string("title", "preferences"), ev, total, start })
}

impl Piece for FormControls {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let p = (t + self.start).rem_euclid(self.total);
        let mut s = State {
            focus: 6,
            email: EMAILS[0].into(),
            typed: -9.0,
            unit: 0,
            open: false,
            hi: 0,
            theme: 0,
            mentions: false,
            knob: 0,
            volume: VOLUME[0],
            pressed: false,
            saved: false,
        };
        for (when, c) in &self.ev {
            if *when > p {
                break;
            }
            match c {
                Change::Focus(f) => s.focus = *f,
                Change::Email(e, at) => (s.email, s.typed) = (e.clone(), *at),
                Change::Open(h) => (s.open, s.hi) = (true, *h),
                Change::Hi(h) => s.hi = *h,
                Change::Unit(u) => (s.unit, s.open) = (*u, false),
                Change::Theme(th) => s.theme = *th,
                Change::Mentions(m) => s.mentions = *m,
                Change::Knob(k) => s.knob = *k,
                Change::Volume(v) => s.volume = *v,
                Change::Press => s.pressed = true,
                Change::Save => (s.pressed, s.saved) = (false, true),
                Change::Unsave => s.saved = false,
            }
        }
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        grid.clear();
        for c in 1..cols - 1 {
            grid.set(c, 0, '─', BORDER);
            grid.set(c, rows - 1, '─', BORDER);
        }
        for r in 1..rows - 1 {
            grid.set(0, r, '│', BORDER);
            grid.set(cols - 1, r, '│', BORDER);
        }
        for (c, r, ch) in [(0, 0, '╭'), (cols - 1, 0, '╮'), (0, rows - 1, '╰'), (cols - 1, rows - 1, '╯')] {
            grid.set(c, r, ch, BORDER);
        }
        grid.text_at(2, 0, &format!(" {} ", self.title), TITLE);
        let keys = " tab next · space change · enter save ";
        grid.text_at(cols - 2 - keys.chars().count() as i64, rows - 1, keys, BORDER);
        for (i, f) in ["email", "units", "theme", "notify", "sync", "volume"].iter().enumerate() {
            let y = 2 + 2 * i as i64;
            let here = s.focus == i;
            grid.text_at(LABEL_X, y, f, if here { FOCUS } else { LABEL });
            if here {
                grid.text_at(LABEL_X - 2, y, "▸", FOCUS);
            }
        }
        let field = |i: usize| if s.focus == i { FOCUS } else { BORDER };
        // The caret holds while keys are coming, then blinks.
        let since = p - s.typed;
        let caret = s.focus == 0 && (since < 0.4 || ((since - 0.4) / 0.5).floor() as i64 % 2 == 1);
        grid.text_at(FIELD, 2, "[", field(0));
        grid.text_at(FIELD + 2, 2, &s.email, VALUE);
        if caret {
            grid.text_at(FIELD + 2 + s.email.len() as i64, 2, "█", FOCUS);
        }
        grid.text_at(FIELD + 3 + WIDE as i64, 2, "]", field(0));
        grid.text_at(FIELD, 4, "[", field(1));
        grid.text_at(FIELD + 2, 4, UNITS[s.unit], VALUE);
        grid.text_at(FIELD + 2 + WIDE as i64 - 1, 4, if s.open { "▴" } else { "▾" }, field(1));
        grid.text_at(FIELD + 3 + WIDE as i64, 4, "]", field(1));
        let mut x = FIELD;
        for (i, th) in THEMES.iter().enumerate() {
            let on = s.theme == i;
            grid.text_at(x, 6, if on { "◉" } else { "○" }, if on { CHOSEN } else { LABEL });
            grid.text_at(x + 2, 6, th, if on { VALUE } else { LABEL });
            x += th.len() as i64 + 5;
        }
        for (i, (name, on)) in [("replies", true), ("mentions", s.mentions)].iter().enumerate() {
            let x = FIELD + 13 * i as i64;
            grid.text_at(x, 8, if *on { "▣" } else { "□" }, if *on { CHOSEN } else { LABEL });
            grid.text_at(x + 2, 8, name, if *on { VALUE } else { LABEL });
        }
        // A switch: the knob slides along a shaded track, filling it behind.
        grid.text_at(FIELD, 10, "▕", BORDER);
        for i in 0..6 {
            let (ch, c) = if i < s.knob {
                ('▒', CHOSEN)
            } else if i < s.knob + 2 {
                ('█', VALUE)
            } else {
                ('░', BORDER)
            };
            grid.set(FIELD + 1 + i as i64, 10, ch, c);
        }
        grid.text_at(FIELD + 7, 10, "▏", BORDER);
        grid.text_at(FIELD + 9, 10, if s.knob == 4 { "on" } else { "off" }, if s.knob == 4 { CHOSEN } else { LABEL });
        let k = (s.volume / 100.0 * (TRACK - 1) as f64).round() as usize;
        for i in 0..TRACK {
            let (ch, c) = if i < k {
                ('━', CHOSEN)
            } else if i == k {
                ('●', VALUE)
            } else {
                ('─', BORDER)
            };
            grid.set(FIELD + i as i64, 12, ch, c);
        }
        grid.text_at(FIELD + TRACK as i64, 12, &format!("{:>5}%", s.volume.round() as u32), VALUE);
        // Buttons at the right, the save button pressed with a fill.
        let bc = RIGHT - 10;
        grid.text_at(bc - 13, 14, "[ cancel ]", LABEL);
        grid.text_at(bc, 14, if s.pressed { "[▒▒save▒▒]" } else { "[  save  ]" }, if s.saved || s.pressed { SAVED } else { VALUE });
        if s.focus == 6 {
            grid.text_at(bc - 1, 14, "▸", FOCUS);
        }
        if s.saved {
            grid.text_at(LABEL_X, 14, "saved", SAVED);
        }
        // The select's list drops over the rows beneath it.
        if s.open {
            let w = (RIGHT - FIELD) as usize;
            grid.text_at(FIELD, 5, &format!("┌{}┐", "─".repeat(w - 2)), FOCUS);
            for (i, u) in UNITS.iter().enumerate() {
                let y = 6 + i as i64;
                grid.text_at(FIELD, y, &format!("│{:<w$}│", "", w = w - 2), FOCUS);
                grid.text_at(FIELD + 1, y, &format!("{}{u}", if s.hi == i { " ▸ " } else { "   " }), if s.hi == i { CHOSEN } else { VALUE });
            }
            grid.text_at(FIELD, 8, &format!("└{}┘", "─".repeat(w - 2)), FOCUS);
        }
    }
}
