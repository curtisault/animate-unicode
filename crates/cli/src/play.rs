//! Plays a piece in the terminal: the alternate screen with the cursor hidden
//! and wrapping off, raw mode so any key stops it, a redraw on resize, and the
//! terminal put back however it ends: after `--seconds`, on a key, on Ctrl+C,
//! on SIGTERM or SIGHUP, or on a panic.
//!
//! The release profile aborts on panic, so a guard's `Drop` would never run
//! there; a panic hook puts the terminal back first instead.

use crate::paint::{dark, Painter, Screen};
use animate_unicode::{Env, Grid, Meta, Piece};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The alternate screen, the cursor hidden and no wrapping, so a glyph a
/// terminal draws two cells wide can only clip its own row.
const ENTER: &str = "\x1b[?1049h\x1b[?25l\x1b[?7l\x1b[2J";
const LEAVE: &str = "\x1b[0m\x1b[?7h\x1b[?25h\x1b[?1049l";

/// How a play went.
pub struct Played {
    /// Ctrl+C stopped it.
    pub interrupted: bool,
    /// The terminal was smaller than the piece when it stopped.
    pub cropped: bool,
    /// The piece's size in terminal cells, and the terminal's, at the end.
    pub piece: (usize, usize),
    pub terminal: (usize, usize),
}

/// What `play` needs besides the piece.
pub struct Settings {
    pub mono: bool,
    pub light: bool,
    pub fps: Option<u32>,
    pub seconds: Option<f64>,
}

fn restore() {
    let mut out = io::stdout();
    let _ = out.write_all(LEAVE.as_bytes());
    let _ = out.flush();
    let _ = terminal::disable_raw_mode();
}

/// Puts the terminal back when dropped: every ordinary way out of `play`.
struct Session;

impl Session {
    fn start() -> io::Result<Session> {
        // A panic aborts in release, skipping Drop: restore before the usual report.
        let report = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            report(info);
        }));
        terminal::enable_raw_mode()?;
        let mut out = io::stdout();
        out.write_all(ENTER.as_bytes())?;
        out.flush()?;
        Ok(Session)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        restore();
        let _ = std::panic::take_hook();
    }
}

/// SIGTERM and SIGHUP set a flag the loop checks, so the terminal is put back
/// before the process goes. (In raw mode Ctrl+C is a key, not SIGINT.)
fn signals() -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    #[cfg(unix)]
    for sig in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGHUP] {
        let _ = signal_hook::flag::register(sig, Arc::clone(&stop));
    }
    stop
}

fn size() -> (usize, usize) {
    // A terminal that reports no size (some ptys) is taken to be 80 by 24.
    match terminal::size() {
        Ok((c, r)) if c > 0 && r > 0 => (usize::from(c), usize::from(r)),
        _ => (80, 24),
    }
}

/// Plays `piece` until a key, Ctrl+C, a signal or `settings.seconds`.
pub fn play(meta: &Meta, mut piece: Box<dyn Piece>, settings: &Settings) -> io::Result<Played> {
    let mut painter = Painter::new(meta, !settings.mono);
    // A coloured piece on its own ground shades for that ground, as on a canvas.
    let env = Env { paper: painter.ground().map_or(settings.light, |g| !dark(g)) };
    let mut grid = Grid::new(meta.cols, meta.rows);
    let mut screen = Screen::new(painter.cells.len());
    let rate = settings.fps.unwrap_or(meta.fps);
    let step = (rate > 0).then(|| Duration::from_secs_f64(1.0 / f64::from(rate)));
    let stop = signals();

    let session = Session::start()?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    let start = Instant::now();
    let end = settings.seconds.map(|s| start + Duration::from_secs_f64(s.max(0.0)));
    let (mut t, mut last) = (0.0, start);
    let mut term = size();
    let mut draw = |t: f64, term: (usize, usize), screen: &mut Screen, out: &mut io::BufWriter<_>| -> io::Result<()> {
        piece.frame(t, &env, &mut grid);
        painter.paint(&grid);
        out.write_all(screen.draw(&painter, term.0, term.1).as_bytes())?;
        out.flush()
    };
    draw(t, term, &mut screen, &mut out)?;

    let mut interrupted = false;
    // Frames fall on fixed boundaries from the start, so a slow frame does not
    // push every later one back.
    let mut next = step.map(|s| start + s);
    loop {
        let now = Instant::now();
        if stop.load(Ordering::Relaxed) || end.is_some_and(|e| now >= e) {
            break;
        }
        // Wait for a key until the next frame, the end, or a quarter second
        // (to notice a signal), whichever comes first.
        let mut wake = now + Duration::from_millis(250);
        for at in [next, end].into_iter().flatten() {
            wake = wake.min(at);
        }
        if event::poll(wake.saturating_duration_since(now))? {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => {
                    interrupted = k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL);
                    break;
                }
                Event::Resize(..) => {
                    term = size();
                    screen.forget();
                    out.write_all(b"\x1b[2J")?;
                    draw(t, term, &mut screen, &mut out)?;
                }
                _ => {}
            }
            continue;
        }
        if let (Some(s), Some(at)) = (step, next) {
            let now = Instant::now();
            if now >= at {
                // Play time moves with the clock, but no more than 100 ms a
                // frame, so a stopped process does not jump ahead on waking.
                t += (now - last).min(Duration::from_millis(100)).as_secs_f64();
                last = now;
                draw(t, term, &mut screen, &mut out)?;
                let mut n = at + s;
                if n <= now {
                    n = now + s; // fell behind: skip ahead rather than catch up in a burst
                }
                next = Some(n);
            }
        }
    }
    drop(out);
    drop(session);
    Ok(Played { interrupted, cropped: screen.cropped, piece: (meta.cols, painter.rows), terminal: term })
}
