//! cpu meters: the top of an htop screen. Eight cores split into user and
//! kernel time, memory and swap bars, load and uptime, and the busiest
//! processes. After ascii.rest's cpu-meters.ts (MIT, @bas3line).
//!
//! The machine is ascii.rest's, from its seeded generator. The meters are
//! blocks, each part its own shade so they part in one ink (user `█`, kernel
//! `▓`, buffers `▒`, cache `░`), and in htop's own colours: user green,
//! kernel red, buffers blue, cache amber; processes working hard are bright.

use crate::util::Mulberry32;
use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The CPU meters' meta.
pub static META: Meta = Meta {
    name: "cpu meters",
    slug: "cpu-meters",
    category: Category::Data,
    note: "htop's meters in block shades and its colours, over a process list",
    cols: 66,
    rows: 16,
    fps: 10,
    options: Some(r#"{"commands": ["build", "indexer", "server", "backup", "sync", "editor", "shell"]}"#),
    // User, kernel, buffers, cache; labels; text; header; busy; root.
    palette: Some(&["#5ad870", "#e85a5a", "#5a8af0", "#e8b04a", "#4ac8e8", "#a8aebb", "#7ad8f0", "#ffffff", "#e87a7a"]),
    ground: Some("#0b0c10"),
    ..Meta::DEFAULT
};

const USER: u8 = 0;
const KERNEL: u8 = 1;
const BUFFERS: u8 = 2;
const CACHE: u8 = 3;
const LABEL: u8 = 4;
const TEXT: u8 = 5;
const HEADER: u8 = 6;
const BUSY: u8 = 7;
const ROOT: u8 = 8;
/// Refreshes per second, like htop's delay, and refreshes run before frame 0.
const RATE: f64 = 2.0;
const WARM: usize = 80;
const NCPU: usize = 8;
/// Gigabytes of memory and swap, and one meter column, label and brackets included.
const MEM: f64 = 15.5;
const SWAP: f64 = 2.0;
const COL: usize = 31;

struct Core {
    u: f64,
    k: f64,
    idle: f64,
    base: f64,
    part: [f64; 2],
}

struct Job {
    left: u32,
    load: f64,
    on: Vec<bool>,
}

struct Proc {
    cmd: String,
    pid: u32,
    root: bool,
    virt: f64,
    mem: f64,
    time: f64,
    w: f64,
    cpu: f64,
}

/// The machine.
pub struct CpuMeters {
    commands: Vec<String>,
    made: usize,
    rand: Mulberry32,
    cores: Vec<Core>,
    jobs: Vec<Job>,
    mem: [f64; 4], // used, buffers, cache, swap
    load: [f64; 3],
    tasks: i64,
    procs: Vec<Proc>,
}

fn size(gb: f64) -> String {
    if gb < 0.98 {
        format!("{}M", (gb * 1024.0).round())
    } else if gb < 10.0 {
        format!("{gb:.2}G")
    } else {
        format!("{gb:.1}G")
    }
}

fn cpu_time(s: f64) -> String {
    let m = (s / 60.0).floor();
    let sec = s - m * 60.0;
    if m < 60.0 {
        format!("{m}:{sec:05.2}")
    } else {
        format!("{}h{:02}:{:02}", (m / 60.0).floor(), m as u64 % 60, sec.floor() as u64)
    }
}

impl CpuMeters {
    fn reset(&mut self) {
        let mut rand = Mulberry32::new(1);
        // Each core's load is a background part plus one part per job, each easing to its target.
        self.cores = (0..NCPU).map(|_| Core { u: 0.1, k: 0.02, idle: 0.03 + rand.next_f64() * 0.14, base: 0.1, part: [0.0; 2] }).collect();
        // Two kinds of work that come and go, run by the first two commands.
        self.jobs = (0..2).map(|_| Job { left: 0, load: 0.0, on: vec![] }).collect();
        self.mem = [4.6, 0.31, 5.2, 0.13];
        self.load = [1.62, 1.24, 0.91];
        self.tasks = 141;
        self.procs = self
            .commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| Proc {
                cmd: cmd.chars().take(13).collect(),
                pid: 1000 + (rand.next_f64() * 48000.0) as u32,
                root: i % 3 == 2,
                virt: 0.4 + rand.next_f64() * 3.2,
                mem: 0.4 + rand.next_f64() * 4.0,
                time: 20.0 + rand.next_f64() * 3000.0,
                w: rand.next_f64(),
                cpu: 0.0,
            })
            .collect();
        self.rand = rand;
        self.made = 0;
    }

    /// One refresh of the whole machine.
    fn step(&mut self) {
        let r = &mut self.rand;
        // Now and then a job lands on some of the cores, works a while, and lets go.
        for (who, j) in self.jobs.iter_mut().enumerate() {
            if j.left > 0 {
                j.left -= 1;
            } else if r.next_f64() < [0.1, 0.06][who] {
                j.left = 5 + (r.next_f64() * 16.0) as u32;
                j.load = 0.35 + r.next_f64() * 0.55;
                j.on = (0..NCPU).map(|_| r.next_f64() < 0.3 + j.load * 0.5).collect();
            }
        }
        let (mut busy, mut share) = (0.0, [0.0; 2]);
        for (i, c) in self.cores.iter_mut().enumerate() {
            if r.next_f64() < 0.06 {
                c.idle = 0.02 + r.next_f64() * 0.2;
            }
            c.base += (c.idle - c.base) * 0.6;
            for (n, j) in self.jobs.iter().enumerate() {
                let target = if j.left > 0 && j.on.get(i).copied().unwrap_or(false) { j.load } else { 0.0 };
                c.part[n] += (target - c.part[n]) * 0.6;
            }
            let want = c.base + c.part[0] + c.part[1];
            c.u = (want + (r.next_f64() - 0.5) * 0.12).clamp(0.004, 0.95);
            let k = c.u * (0.1 + r.next_f64() * 0.22) + r.next_f64() * 0.015;
            let cap = 0.97 + r.next_f64() * 0.03 - c.u;
            c.k = k.clamp(0.0, cap.max(0.0));
            busy += c.u + c.k;
            for (s, part) in share.iter_mut().zip(c.part) {
                *s += (c.u + c.k) * part / want;
            }
        }
        let working = self.jobs.iter().any(|j| j.left > 0);
        let m = &mut self.mem;
        m[0] = (m[0] + (4.4 + if working { 2.6 } else { 0.0 } - m[0]) * 0.12 + (r.next_f64() - 0.5) * 0.05).clamp(3.0, 9.0);
        m[1] = (m[1] + (r.next_f64() - 0.5) * 0.01).clamp(0.25, 0.4);
        m[2] = (m[2] + (r.next_f64() - 0.45) * 0.03).clamp(4.0, MEM - 0.4 - m[0] - m[1]);
        m[3] = (m[3] + (r.next_f64() - 0.5) * 0.004).clamp(0.11, 0.16);
        for (i, l) in self.load.iter_mut().enumerate() {
            *l += (busy - *l) * (1.0 - (-1.0 / RATE / [60.0, 300.0, 900.0][i]).exp());
        }
        self.tasks = (self.tasks + ((r.next_f64() - 0.5) * 2.2).round() as i64).clamp(136, 149);
        // A job's process takes what the job does; the rest share the idle work.
        let job_cpu = (share[0] + share[1]) * 100.0;
        let mut sw = 0.0;
        for p in &mut self.procs {
            p.w = (p.w + (r.next_f64() - 0.5) * 0.3).clamp(0.02, 1.0);
            sw += p.w * p.w;
        }
        for (i, p) in self.procs.iter_mut().enumerate() {
            let mine = i < 2 && self.jobs[i].left > 0;
            p.cpu = (busy * 100.0 - job_cpu) * 0.75 * p.w * p.w / sw + if i < 2 { share[i] * 100.0 } else { 0.0 };
            p.mem = (p.mem + (r.next_f64() - 0.5) * 0.06 + if mine { 0.05 } else { -0.004 }).clamp(0.2, 14.0);
            p.time += p.cpu / 100.0 / RATE;
        }
    }

    fn fill(&mut self, need: usize) {
        if need < self.made {
            self.reset();
        }
        while self.made < need {
            self.step();
            self.made += 1;
        }
    }
}

/// CPU meters; reads `commands`, the processes' names.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let commands = options.list_str("commands", &["build", "indexer", "server", "backup", "sync", "editor", "shell"]);
    let mut m = CpuMeters { commands, made: 0, rand: Mulberry32::new(1), cores: vec![], jobs: vec![], mem: [0.0; 4], load: [0.0; 3], tasks: 0, procs: vec![] };
    m.reset();
    Box::new(m)
}

/// A meter at (x, y): its label, then stacked parts each a fraction, glyph
/// and colour, and its value at the right inside the brackets.
fn meter(grid: &mut Grid, x: i64, y: i64, label: &str, parts: &[(f64, char, u8)], text: &str) {
    let inner = COL - 5;
    grid.text_at(x, y, &format!("{label:>3}"), LABEL);
    grid.text_at(x + 3, y, "[", TEXT);
    let (mut at, mut acc) = (0usize, 0.0);
    for &(f, g, c) in parts {
        acc += f;
        let end = ((acc * inner as f64).round() as usize).min(inner);
        while at < end {
            grid.set(x + 4 + at as i64, y, g, c);
            at += 1;
        }
    }
    grid.text_at(x + 4 + (inner - text.len()) as i64, y, text, TEXT);
    grid.text_at(x + 4 + inner as i64, y, "]", TEXT);
}

impl Piece for CpuMeters {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        self.fill(WARM + (t * RATE).floor() as usize);
        grid.clear();
        let (lx, rx) = (1, 1 + COL as i64 + 2);
        for i in 0..NCPU / 2 {
            for (x, n) in [(lx, i), (rx, i + NCPU / 2)] {
                let c = &self.cores[n];
                meter(grid, x, 1 + i as i64, &n.to_string(), &[(c.u, '█', USER), (c.k, '▓', KERNEL)], &format!("{:.1}%", (c.u + c.k) * 100.0));
            }
        }
        let m = self.mem;
        meter(grid, lx, 5, "Mem", &[(m[0] / MEM, '█', USER), (m[1] / MEM, '▒', BUFFERS), (m[2] / MEM, '░', CACHE)], &format!("{}/{MEM}G", size(m[0])));
        meter(grid, lx, 6, "Swp", &[(m[3] / SWAP, '█', KERNEL)], &format!("{}/{SWAP:.2}G", size(m[3])));
        let running = self.cores.iter().filter(|c| c.u + c.k > 0.45).count().max(1);
        let up = 4 * 86400 + 3 * 3600 + 12 * 60 + 45 + t.floor() as u64;
        let pairs = [
            ("Tasks: ", format!("{}, {} thr; {running} running", self.tasks, self.tasks * 3 - 5)),
            ("Load average: ", self.load.iter().map(|l| format!("{l:.2}")).collect::<Vec<_>>().join(" ")),
            ("Uptime: ", format!("{} days, {:02}:{:02}:{:02}", up / 86400, up / 3600 % 24, up / 60 % 60, up % 60)),
        ];
        for (i, (k, v)) in pairs.iter().enumerate() {
            grid.text_at(rx, 5 + i as i64, k, LABEL);
            grid.text_at(rx + k.len() as i64, 5 + i as i64, v, BUSY);
        }
        let row = |pid: &str, user: &str, virt: &str, res: &str, s: &str, cpu: &str, mem: &str, time: &str, cmd: &str| {
            format!(" {pid:>5} {user:<8} {virt:>5} {res:>5} {s} {cpu:>5} {mem:>4} {time:>9}  {cmd}")
        };
        grid.text_at(0, 9, &row("PID", "USER", "VIRT", "RES", "S", "CPU%", "MEM%", "TIME+", "Command"), HEADER);
        let mut top: Vec<&Proc> = self.procs.iter().collect();
        top.sort_by(|a, b| b.cpu.total_cmp(&a.cpu));
        for (i, p) in top.iter().take(5).enumerate() {
            let y = 10 + i as i64;
            let line = row(
                &p.pid.to_string(),
                if p.root { "root" } else { "app" },
                &size(p.virt),
                &size(p.mem / 100.0 * MEM),
                if p.cpu > 30.0 { "R" } else { "S" },
                &format!("{:.1}", p.cpu),
                &format!("{:.1}", p.mem),
                &cpu_time(p.time),
                &p.cmd,
            );
            grid.text_at(0, y, &line, if p.cpu > 30.0 { BUSY } else { TEXT });
            if p.root {
                grid.text_at(7, y, "root", ROOT);
            }
        }
    }
}
