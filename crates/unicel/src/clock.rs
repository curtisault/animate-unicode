//! The wall clock, for pieces whose picture depends on the real time or date
//! (`Meta::clock`). Nothing here reads a clock: the shell or the CLI puts the
//! time into [`Env::now`](crate::Env), and when it is absent (the contract
//! test, a server-rendered still) a piece uses [`Wall::FIXED`], so its frames
//! stay deterministic.

/// A moment, and how far local time there is from UTC.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wall {
    /// Seconds since 1970-01-01 00:00 UTC.
    pub unix: f64,
    /// Minutes local time is ahead of UTC (negative west of Greenwich).
    pub offset: i32,
}

/// A moment as local calendar and clock fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Local {
    /// The year, e.g. 2026.
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
    /// 1 to 31.
    pub day: u32,
    /// 0 for Sunday to 6 for Saturday.
    pub weekday: u32,
    /// Day of the year, 1 to 366.
    pub yday: u32,
    /// 0 to 23.
    pub hour: u32,
    /// 0 to 59.
    pub minute: u32,
    /// 0 to 59.
    pub second: u32,
    /// Thousandths of the second, 0 to 999.
    pub milli: u32,
}

impl Wall {
    /// The moment a clock piece shows when it is not told the time:
    /// Thursday 2026-10-08, 10:09:30 UTC, the hands of a watch advertisement.
    pub const FIXED: Wall = Wall { unix: 1_791_454_170.0, offset: 0 };

    /// The same moment `secs` seconds later.
    pub fn plus(self, secs: f64) -> Wall {
        Wall { unix: self.unix + secs, ..self }
    }

    /// The moment as local fields.
    pub fn local(&self) -> Local {
        let secs = self.unix + f64::from(self.offset) * 60.0;
        let whole = secs.floor();
        let milli = (((secs - whole) * 1000.0) as u32).min(999);
        let days = (whole / 86_400.0).floor() as i64;
        let rest = (whole as i64 - days * 86_400) as u32;
        let (year, month, day) = civil(days);
        Local {
            year,
            month,
            day,
            weekday: (days + 4).rem_euclid(7) as u32, // 1970-01-01 was a Thursday
            yday: (days - days_from_civil(year, 1, 1)) as u32 + 1,
            hour: rest / 3600,
            minute: rest / 60 % 60,
            second: rest % 60,
            milli,
        }
    }
}

/// Year, month and day from days since 1970-01-01 (Howard Hinnant's algorithm).
fn civil(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((yoe + era * 400 + i64::from(month <= 2)) as i32, month, day)
}

/// Days since 1970-01-01 of a date.
pub fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

/// The number of days in a month.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    (days_from_civil(ny, nm, 1) - days_from_civil(year, month, 1)) as u32
}

/// The weekday (0 Sunday) of a date.
pub fn weekday(year: i32, month: u32, day: u32) -> u32 {
    (days_from_civil(year, month, day) + 4).rem_euclid(7) as u32
}

/// Play time ticks the clock so it steps evenly between frames, and the wall
/// clock pulls it back whenever play was paused or fell behind.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ticker {
    offset: Option<f64>,
}

impl Ticker {
    /// The moment to show at play time `t`, given the wall clock (or none).
    pub fn now(&mut self, t: f64, wall: Option<Wall>) -> Wall {
        let wall = wall.unwrap_or(Wall::FIXED.plus(t));
        let offset = match self.offset {
            Some(o) if (o + t - wall.unix).abs() <= 1.0 => o,
            _ => wall.unix - t,
        };
        self.offset = Some(offset);
        Wall { unix: offset + t, offset: wall.offset }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_fields_match_known_dates() {
        let at = |unix: f64, offset: i32| Wall { unix, offset }.local();
        let l = Wall::FIXED.local();
        assert_eq!((l.year, l.month, l.day, l.weekday, l.yday, l.hour, l.minute, l.second), (2026, 10, 8, 4, 281, 10, 9, 30));
        assert_eq!(at(0.0, 0), Local { year: 1970, month: 1, day: 1, weekday: 4, yday: 1, hour: 0, minute: 0, second: 0, milli: 0 });
        let l = at(951_782_400.0, 0); // 2000-02-29, a Tuesday
        assert_eq!((l.year, l.month, l.day, l.weekday, l.yday), (2000, 2, 29, 2, 60));
        let l = at(1_709_251_199.5, 0); // 2024-02-29 23:59:59.5
        assert_eq!((l.day, l.hour, l.second, l.milli), (29, 23, 59, 500));
        let l = at(4_102_444_800.0, 0); // 2100-01-01, a Friday
        assert_eq!((l.year, l.month, l.day, l.weekday), (2100, 1, 1, 5));
        // Five hours west of UTC, midnight UTC is still the evening before.
        let l = at(0.0, -300);
        assert_eq!((l.year, l.month, l.day, l.hour), (1969, 12, 31, 19));
    }

    #[test]
    fn months_and_weekdays() {
        assert_eq!((days_in_month(2024, 2), days_in_month(2100, 2), days_in_month(2000, 2), days_in_month(2026, 12)), (29, 28, 29, 31));
        assert_eq!(weekday(2026, 10, 1), 4);
    }

    #[test]
    fn the_ticker_steps_with_play_time_and_snaps_to_the_wall() {
        let mut k = Ticker::default();
        let w = |unix| Some(Wall { unix, offset: 60 });
        assert_eq!(k.now(0.0, w(1000.0)).unix, 1000.0);
        // The wall clock jitters a little: play time keeps the steps even.
        assert_eq!(k.now(0.5, w(1000.7)).unix, 1000.5);
        // Paused for a minute: the wall clock wins.
        assert_eq!(k.now(1.0, w(1061.0)).unix, 1061.0);
        assert_eq!(k.now(1.0, w(1061.0)).offset, 60);
        // No wall clock: the fixed instant, moving with play time.
        let mut k = Ticker::default();
        assert_eq!(k.now(2.0, None).unix, Wall::FIXED.unix + 2.0);
    }
}
