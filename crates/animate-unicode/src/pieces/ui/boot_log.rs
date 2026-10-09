//! boot log: a machine starting up. The firmware tests its memory and probes
//! its devices, the kernel prints timestamped lines, services start, and a
//! login waits. After ascii.rest's boot-log.ts (MIT, @bas3line).
//!
//! The same boot, in colour, the way a console paints it: green for what
//! came up, amber for the warning, dim kernel timestamps, a cyan memory bar
//! in eighth blocks; and a device that is still being probed spins a braille
//! spinner rather than a bar.

use crate::util::Mulberry32;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// The boot log's meta.
pub static META: Meta = Meta {
    name: "boot log",
    slug: "boot-log",
    category: Category::Ui,
    note: "a boot in colour: green ok, amber warn, braille spinners",
    cols: 64,
    rows: 16,
    fps: 15,
    charset: Charset::Extended,
    // Text, dim, label, ok, warn, the bar, the prompt.
    palette: Some(&["#c8ccd4", "#5c6370", "#e5e9f0", "#7ad87a", "#f2c45a", "#4ac8e8", "#ffffff"]),
    ground: Some("#0c0d10"),
    ..Meta::DEFAULT
};

const TEXT: u8 = 0;
const DIM: u8 = 1;
const LABEL: u8 = 2;
const OK: u8 = 3;
const WARN: u8 = 4;
const BAR: u8 = 5;
const BRIGHT: u8 = 6;

/// Text width inside a one-column margin.
const W: usize = 62;
/// Megabytes; when the memory test starts; seconds it takes.
const MEMORY: u64 = 16384;
const MEM_AT: f64 = 0.4;
const TEST: f64 = 2.0;
const SPIN: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const EIGHTHS: [char; 8] = ['░', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];

/// Kernel lines: seconds since the previous line on the kernel's clock, then the text.
const KERNEL: [(f64, &str); 13] = [
    (0.0, "kernel 6.8.0 starting on 8 cpus"),
    (0.0, "command line: root=/dev/sda2 ro quiet"),
    (0.004, "memory: 16384 MB available, 412 MB reserved"),
    (0.077, "smp: brought up 1 node, 8 cpus"),
    (0.082, "pci: probing bus 0000:00"),
    (0.047, "pci 0000:00:17.0: sata controller, 6 ports"),
    (0.087, "usb 1-1: new high speed device, number 2"),
    (0.022, "usb 1-4: new full speed device, number 3"),
    (0.084, "sd 0:0:0:0: [sda] 1000215216 sectors (512 GB)"),
    (0.002, " sda: sda1 sda2"),
    (0.107, "eth0: link up, 1000 Mb/s, full duplex"),
    (0.129, "ext4: mounted sda2 read-only"),
    (0.062, "init: running /sbin/init"),
];
/// Service lines: seconds after the previous line, the text, and for a start
/// job how long it spins before it lands. The last is the login.
const SERVICES: [(f64, &str, f64); 20] = [
    (0.35, "[  OK  ] mounted root file system", 0.0),
    (0.12, "[  OK  ] started file system check on sda1", 0.0),
    (0.08, "[  OK  ] reached target local file systems", 0.0),
    (0.2, "[  OK  ] started journal service", 0.0),
    (0.1, "[  OK  ] started device manager", 0.0),
    (0.06, "[  OK  ] created runtime directories", 0.0),
    (0.15, "[  OK  ] started load kernel modules", 0.0),
    (0.25, "[ WARN ] clock not synced, using hardware time", 0.0),
    (0.2, "[  OK  ] started system logger", 0.0),
    (0.08, "[  OK  ] reached target sockets", 0.0),
    (0.1, "[  OK  ] reached target basic system", 0.0),
    (0.15, "[  OK  ] finished waiting for network", 2.6),
    (0.1, "[  OK  ] reached target network", 0.0),
    (0.25, "[  OK  ] synced clock to network time", 0.0),
    (0.1, "[  OK  ] started secure shell server", 0.0),
    (0.08, "[  OK  ] started job scheduler", 0.0),
    (0.1, "[  OK  ] started print spooler", 0.0),
    (0.3, "[  OK  ] reached target multi-user system", 0.0),
    (0.5, "", 0.0),
    (0.1, "node-1 login: ", 0.0),
];
/// Seconds at the login prompt before the next boot.
const WAIT: f64 = 3.5;

/// A line as runs of text, each in a colour.
type Line = Vec<(String, u8)>;

/// What one entry of the log says at a moment.
enum Text {
    Fixed(Line),
    Memory,
    Bar,
    Probe(&'static str, &'static str, &'static str, f64),
    Job(&'static str, f64, f64),
}

/// The boot: every entry and when it appears, when the login shows, and the loop.
pub struct BootLog {
    entries: Vec<(f64, Text)>,
    login: f64,
    period: f64,
    start: f64,
}

/// A self-test row: a label, its detail, and a status at the right edge.
fn row(label: &str, detail: &str, status: &str, color: u8) -> Line {
    let left = format!("{label:<11}{detail}");
    let pad = W.saturating_sub(left.chars().count() + status.chars().count());
    vec![(format!("{label:<11}"), LABEL), (detail.to_string(), TEXT), (" ".repeat(pad), TEXT), (status.to_string(), color)]
}

/// A service line: its bracketed status coloured, the rest plain.
fn service(text: &str) -> Line {
    match text.split_once("] ") {
        Some((status, rest)) if status.starts_with('[') => {
            let color = if status.contains("WARN") { WARN } else { OK };
            vec![("[".into(), DIM), (status[1..].to_string(), color), ("] ".into(), DIM), (rest.to_string(), TEXT)]
        }
        _ => vec![(text.to_string(), BRIGHT)],
    }
}

fn tested(now: f64) -> f64 {
    ((now - MEM_AT) / TEST).clamp(0.0, 1.0)
}

impl Text {
    fn at(&self, now: f64) -> Line {
        match self {
            Text::Fixed(l) => l.clone(),
            Text::Memory => {
                let f = tested(now);
                let mb = (f * MEMORY as f64) as u64;
                row("memory", &format!("{mb:>5} MB of {MEMORY} MB"), if f < 1.0 { "testing" } else { "ok" }, if f < 1.0 { WARN } else { OK })
            }
            Text::Bar => {
                // The test's progress, the address it has reached, and a percentage.
                let f = tested(now);
                let fill = f * 32.0;
                let full = fill as usize;
                let mut bar: String = "█".repeat(full);
                if full < 32 {
                    bar.push(EIGHTHS[((fill - full as f64) * 8.0) as usize]);
                }
                let rest = "░".repeat(32 - bar.chars().count());
                let top = MEMORY * 1_048_576;
                let addr = if f < 1.0 { (f * top as f64 / 4096.0) as u64 * 4096 } else { top - 1 };
                let pct = format!("{}%", (f * 100.0) as u32);
                vec![(" ".repeat(11), TEXT), (bar, BAR), (rest, DIM), (format!("  0x{addr:09X}"), DIM), (format!("{pct:>w$}", w = W - 56), TEXT)]
            }
            Text::Probe(label, detail, status, done) => {
                if now >= *done {
                    row(label, detail, status, OK)
                } else {
                    let spin = SPIN[((now + done) * 12.0) as usize % SPIN.len()];
                    vec![(format!("{label:<11}"), LABEL), ("detecting ".into(), TEXT), (spin.to_string(), BAR)]
                }
            }
            Text::Job(text, from, spin) => {
                let age = now - from;
                if age >= *spin {
                    return service(text);
                }
                // Three stars sweep back and forth inside the brackets. The job
                // shows itself once it has run for 5 s, so its clock starts there.
                let k = (age * 8.0) as usize % 6;
                let p = if k < 4 { k } else { 6 - k };
                let stars = format!("{:<6}", " ".repeat(p) + "***");
                vec![
                    ("[".into(), DIM),
                    (stars, WARN),
                    ("] ".into(), DIM),
                    (format!("a start job is running for network ({}s / 1min 30s)", (age + 5.0) as u32), TEXT),
                ]
            }
        }
    }
}

/// The boot log; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut rand = Mulberry32::new(11);
    let mut entries: Vec<(f64, Text)> = vec![];
    let rule = || Text::Fixed(vec![("─".repeat(W), DIM)]);
    let mut at = 0.12;
    let mut put = |at: f64, t: Text| entries.push((at, t));
    put(at, Text::Fixed(row("system firmware 2.31.0", "", "self test", BRIGHT)));
    at += 0.04;
    put(at, rule());
    at += 0.04;
    put(at, Text::Fixed(row("cpu", "4 cores, 8 threads at 2.40 GHz", "ok", OK)));
    at += 0.04;
    put(at, Text::Fixed(row("cache", "l1 256 KB   l2 2 MB   l3 8 MB", "ok", OK)));
    at += 0.04;
    put(at, Text::Memory);
    put(at, Text::Bar);
    at += 0.04;
    put(at, Text::Fixed(row("display", "1920 x 1080 at 60 Hz", "ok", OK)));
    for (label, detail, status, done) in [
        ("storage", "sata 0: disk 0, 512 GB", "ok", 1.3),
        ("usb", "2 controllers, 2 devices", "ok", 1.9),
        ("network", "1 port, 1000 Mb/s", "link up", 2.2),
        ("sensors", "cpu 41°C   board 33°C   fan 2400 rpm", "ok", 0.95),
    ] {
        at += 0.04;
        put(at, Text::Probe(label, detail, status, done));
    }
    at += 0.04;
    put(at, Text::Fixed(row("boot order", "disk 0, usb, network", "", TEXT)));
    at += 0.04;
    put(at, rule());
    at += 0.04;
    put(at, Text::Fixed(vec![("F2 setup    F12 boot menu    esc skip memory test".into(), DIM)]));
    at = MEM_AT + TEST + 0.2;
    put(at, Text::Fixed(vec![]));
    at += 0.05;
    put(at, Text::Fixed(vec![("booting from disk 0".into(), TEXT)]));
    at += 0.2;
    put(at, Text::Fixed(vec![("loading kernel 6.8.0 ...".into(), TEXT)]));
    // The kernel's burst plays at two thirds speed so it reads as a fast scroll.
    at += 0.3;
    let mut stamp: f64 = 0.0;
    for (gap, text) in KERNEL {
        stamp += gap;
        at += gap * 1.5;
        let s = if stamp > 0.0 { stamp + rand.next_f64() * 0.0009 } else { 0.0 };
        put(at, Text::Fixed(vec![(format!("[{s:>12.6}] "), DIM), (text.into(), TEXT)]));
    }
    for (gap, text, spin) in SERVICES {
        at += gap;
        if spin > 0.0 {
            put(at, Text::Job(text, at, spin));
            at += spin;
        } else {
            put(at, Text::Fixed(service(text)));
        }
    }
    let login = at;
    let period = login + WAIT;
    // Frame 0 is the finished boot at the login prompt, a moment before it restarts.
    Box::new(BootLog { entries, login, period, start: period - 0.25 })
}

impl Piece for BootLog {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let now = (t + self.start).rem_euclid(self.period);
        let mut lines: Vec<Line> = self.entries.iter().filter(|(at, _)| *at <= now).map(|(_, e)| e.at(now)).collect();
        // The cursor blinks after the login prompt, or at the top of the blank screen.
        if now >= self.login {
            if (now - self.login).rem_euclid(1.0) < 0.6 {
                lines.last_mut().unwrap().push(("█".into(), BRIGHT));
            }
        } else if lines.is_empty() {
            lines.push(vec![("█".into(), BRIGHT)]);
        }
        let top = lines.len().saturating_sub(META.rows);
        grid.clear();
        for (r, line) in lines.iter().skip(top).enumerate() {
            let mut x = 1;
            for (s, c) in line {
                grid.text_at(x, r as i64, s, *c);
                x += s.chars().count() as i64;
            }
        }
    }
}
