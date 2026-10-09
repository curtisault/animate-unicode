//! calendar: this month laid out like the cal command, today picked out,
//! with the weekday and the running time along the bottom. After
//! ascii.rest's calendar.ts (MIT, @bas3line).
//!
//! Today sits in a pill of half blocks, `▐ 8▌`, in the accent colour, so it
//! stands out in one ink as well as in colour; weekends are dimmer than
//! weekdays, and the clock in the corner ticks in its own colour.

use crate::clock::{days_in_month, weekday, Ticker};
use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The calendar's meta.
pub static META: Meta = Meta {
    name: "calendar",
    slug: "calendar",
    category: Category::Ui,
    note: "this month like cal, today in a pill of colour, time below",
    cols: 32,
    rows: 13,
    fps: 8,
    options: Some(r#"{"monday": false, "time": true}"#),
    clock: true,
    // Text, dim (rules, weekends), the heading, today, the clock.
    palette: Some(&["#c8ccd4", "#5c6370", "#e5e9f0", "#f2a65a", "#7ac4e8"]),
    ground: Some("#0e0f13"),
    ..Meta::DEFAULT
};

const TEXT: u8 = 0;
const DIM: u8 = 1;
const HEAD: u8 = 2;
const TODAY: u8 = 3;
const TIME: u8 = 4;
const DAYS: [&str; 7] = ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"];
const MONTHS: [&str; 12] = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
/// Left margin; the grid is 7 cells of 4.
const L: i64 = 2;

/// The calendar: whether weeks start on Monday, whether the clock shows, and its ticker.
pub struct Calendar {
    monday: bool,
    time: bool,
    ticker: Ticker,
}

/// A calendar; reads `monday` and `time`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(Calendar { monday: options.bool("monday", false), time: options.bool("time", true), ticker: Ticker::default() })
}

impl Piece for Calendar {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let now = self.ticker.now(t, env.now).local();
        let order: [u32; 7] = if self.monday { [1, 2, 3, 4, 5, 6, 0] } else { [0, 1, 2, 3, 4, 5, 6] };
        grid.clear();
        let title = format!("{} {}", MONTHS[now.month as usize - 1], now.year);
        grid.text_at(L + (28 - title.len() as i64) / 2, 1, &title, HEAD);
        grid.text_at(L, 2, &"─".repeat(28), DIM);
        for (i, &d) in order.iter().enumerate() {
            let weekend = d == 0 || d == 6;
            grid.text_at(L + 4 * i as i64 + 1, 3, &DAYS[d as usize][..2], if weekend { DIM } else { TEXT });
        }
        let lead = (weekday(now.year, now.month, 1) + 7 - order[0]) % 7;
        let count = days_in_month(now.year, now.month);
        for w in 0..6 {
            for c in 0..7u32 {
                let n = (w * 7 + c) as i64 - lead as i64 + 1;
                if n < 1 || n > count as i64 {
                    continue;
                }
                let (x, y) = (L + 4 * c as i64, 4 + w as i64);
                let d = order[c as usize];
                if n == now.day as i64 {
                    grid.text_at(x, y, &format!("▐{n:>2}▌"), TODAY);
                } else {
                    grid.text_at(x + 1, y, &format!("{n:>2}"), if d == 0 || d == 6 { DIM } else { TEXT });
                }
            }
        }
        grid.text_at(L, 10, &"─".repeat(28), DIM);
        let day = format!("{} {}", DAYS[now.weekday as usize], now.day);
        grid.text_at(L + 1, 11, &day, TEXT);
        if self.time {
            let clock = format!("{:02}:{:02}:{:02}", now.hour, now.minute, now.second);
            grid.text_at(L + 1 + 26 - clock.len() as i64, 11, &clock, TIME);
        }
    }
}
