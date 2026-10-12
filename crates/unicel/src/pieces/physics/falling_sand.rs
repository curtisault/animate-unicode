//! falling sand: sand pouring from a spout onto a shelf, as a cellular
//! automaton. The heap grows at its angle of repose; then the shelf opens, it
//! drains, and it pours again. After ascii.rest's falling-sand.ts (MIT,
//! @bas3line).
//!
//! ascii.rest's automaton, one grain per octant dot: the 55×22 characters
//! hold a 110×88 field, four times the grains, so the heap's slopes are
//! smooth and its grains small. The spout pours two grains wide at twice the
//! steps a frame, so a heap takes as long to build, and it pours in layers of
//! three sand tones, so the heap shows how it grew.

use crate::util::Mulberry32;
use crate::{Category, Charset, Env, Grid, Meta, Octants, Options, Piece};

/// Falling sand's meta.
pub static META: Meta = Meta {
    name: "falling sand",
    slug: "falling-sand",
    category: Category::Physics,
    note: "sand in octants, a grain a dot, pouring in layers and draining",
    cols: 55,
    rows: 22,
    fps: 20,
    charset: Charset::Extended,
    // Three sand tones; grains in the air; the shelf and spout.
    palette: Some(&["#e8c27a", "#c8945a", "#a86a3a", "#fff0c8", "#8a8f98"]),
    ground: Some("#100d0a"),
    ..Meta::DEFAULT
};

/// Automaton steps a frame; the chance each nozzle cell lets a grain go each
/// step; steps of pouring before the sand changes tone; steps played before
/// the first frame.
const STEPS: usize = 8;
const RATE: f64 = 0.92;
const LAYER: usize = 300;
const WARM: usize = 580;
/// The field, a grain a dot.
const W: usize = META.cols * 2;
const H: usize = META.rows * 4;
/// The shelf is the bottom two rows; sand that misses it leaves the frame.
const SHELF: usize = H - 2;
/// The nozzle's left column, and the shelf runs from END to W - 1 - END.
const MID: usize = W / 2 - 1;
const END: usize = 10;
/// The row grains leave the spout from.
const NOZZLE: usize = 11;
const AIR: u8 = 3;
const STEEL: u8 = 4;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Pour,
    Hold,
    Open,
    Close,
}

/// The heap: each dot empty (0) or a grain (1 + tone), when each last moved,
/// the generator, and where the pour is up to.
pub struct FallingSand {
    grid: Vec<u8>,
    moved: Vec<i32>,
    rand: Mulberry32,
    step: i32,
    phase: Phase,
    clock: usize,
    slide: f64,
    tone: u8,
    poured: usize,
    spilled: usize,
    held: usize,
    last: f64,
}

impl FallingSand {
    /// The middle of the nozzle, about which the shelf opens.
    fn mid() -> f64 {
        MID as f64 + 0.5
    }

    /// Opening, each half of the shelf slides back toward its end, from the middle out.
    fn solid(&self, x: i64, y: usize) -> bool {
        y >= SHELF && x >= END as i64 && x <= (W - 1 - END) as i64 && (x as f64 + 0.5 - Self::mid()).abs() >= self.slide
    }

    fn free(&self, x: i64, y: usize) -> bool {
        y < H && (x < 0 || x >= W as i64 || (self.grid[y * W + x as usize] == 0 && !self.solid(x, y)))
    }

    fn mv(&mut self, a: usize, x: i64, y: usize) {
        if x >= 0 && x < W as i64 {
            self.grid[y * W + x as usize] = self.grid[a];
            self.moved[y * W + x as usize] = self.step;
        }
        self.grid[a] = 0;
    }

    fn tick(&mut self) {
        self.step += 1;
        let mut top = H;
        self.held = 0;
        for y in (0..H).rev() {
            let flip = (self.step as usize + y) & 1 == 1;
            for i in 0..W {
                let x = if flip { W - 1 - i } else { i };
                let a = y * W + x;
                if self.grid[a] == 0 || self.moved[a] == self.step {
                    continue;
                }
                self.held += 1;
                let xi = x as i64;
                if y == H - 1 {
                    self.grid[a] = 0; // gone out of the bottom of the frame
                    self.spilled += 1;
                    continue;
                }
                if self.free(xi, y + 1) {
                    self.mv(a, xi, y + 1);
                    continue;
                }
                let d = if self.rand.next_f64() < 0.5 { -1 } else { 1 };
                if self.free(xi + d, y + 1) {
                    self.mv(a, xi + d, y + 1);
                } else if self.free(xi - d, y + 1) {
                    self.mv(a, xi - d, y + 1);
                } else if self.rand.next_f64() < 0.35 && self.free(xi + d, y) && self.free(xi + 2 * d, y + 1) {
                    // A grain on a steep slope slides a step sideways before it rolls on down.
                    self.mv(a, xi + d, y);
                } else if self.moved[a] < self.step - 2 && y < top {
                    top = y;
                }
            }
        }
        // The spout never stops; while the shelf is open the stream runs straight through it.
        for x in [MID, MID + 1] {
            if self.rand.next_f64() < RATE && self.free(x as i64, NOZZLE) {
                self.grid[NOZZLE * W + x] = 1 + self.tone;
                self.moved[NOZZLE * W + x] = self.step;
            }
        }
        self.poured += 1;
        if self.poured.is_multiple_of(LAYER) {
            self.tone = (self.tone + 1) % 3;
        }
        let open = Self::mid() - END as f64 + 0.5;
        // Full once the heap runs off both ends of the shelf, or stands too near the spout.
        match self.phase {
            Phase::Pour if self.spilled > 200 || top < NOZZLE + 20 => {
                self.phase = Phase::Hold;
                self.clock = 0;
            }
            Phase::Hold => {
                self.clock += 1;
                if self.clock > 40 {
                    self.phase = Phase::Open;
                }
            }
            Phase::Open => {
                self.slide = (self.slide + 0.7).min(open);
                // Close again once the heap has run out and only the stream is left.
                if self.slide == open && self.held < 440 {
                    self.phase = Phase::Close;
                    self.spilled = 0;
                }
            }
            Phase::Close => {
                self.slide = (self.slide - 1.0).max(0.0);
                if self.slide == 0.0 {
                    self.phase = Phase::Pour;
                }
            }
            Phase::Pour => {}
        }
    }

    /// A grain is in the air if it moved a moment ago and has nothing resting under it.
    fn air(&self, x: usize, y: usize) -> bool {
        let rests = |x: i64, y: usize| {
            y < H && (self.solid(x, y) || (x >= 0 && x < W as i64 && self.grid[y * W + x as usize] != 0 && self.moved[y * W + x as usize] < self.step - 2))
        };
        let xi = x as i64;
        self.grid[y * W + x] != 0 && self.moved[y * W + x] >= self.step - 2 && !rests(xi, y + 1) && !rests(xi - 1, y + 1) && !rests(xi + 1, y + 1)
    }
}

/// Falling sand; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut s = FallingSand {
        grid: vec![0; W * H],
        moved: vec![-9; W * H],
        rand: Mulberry32::new(7),
        step: 0,
        phase: Phase::Pour,
        clock: 0,
        slide: 0.0,
        tone: 0,
        poured: 0,
        spilled: 0,
        held: 0,
        last: 0.0,
    };
    for _ in 0..WARM {
        s.tick();
    }
    Box::new(s)
}

impl Piece for FallingSand {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let n = ((t - self.last) * META.fps as f64).round().clamp(0.0, 12.0) as usize;
        self.last += n as f64 / META.fps as f64;
        for _ in 0..n * STEPS {
            self.tick();
        }
        // Each character takes the colour most of its grains have; grains
        // in the air are lighter, so the stream shows.
        let mut dots = Octants::new(META.cols, META.rows);
        let mut tally = vec![[0u8; 5]; META.cols * META.rows];
        for y in 0..H {
            for x in 0..W {
                let c = if self.solid(x as i64, y) {
                    STEEL
                } else if self.grid[y * W + x] == 0 {
                    continue;
                } else if self.air(x, y) {
                    AIR
                } else {
                    self.grid[y * W + x] - 1
                };
                dots.set(x as i64, y as i64);
                tally[(y / 4) * META.cols + x / 2][c as usize] += 1;
            }
        }
        grid.clear();
        for cy in 0..META.rows {
            for cx in 0..META.cols {
                if dots.mask(cx, cy) != 0 {
                    let t = &tally[cy * META.cols + cx];
                    let c = (0..5).max_by_key(|&k| (t[k], k)).unwrap_or(0);
                    grid.set(cx as i64, cy as i64, dots.glyph(cx, cy), c as u8);
                }
            }
        }
        // The spout: a hopper down from the top, sand showing inside it; and the shelf's end caps.
        let p = META.cols as i64 / 2 - 3;
        for (r, s) in [(0, "│░░░░░│"), (1, "╰╮░░░╭╯"), (2, " ╰─╯ ")] {
            for (i, ch) in s.chars().enumerate() {
                if ch != ' ' {
                    grid.set(p + i as i64 + i64::from(r == 2), r, ch, if ch == '░' { self.tone } else { STEEL });
                }
            }
        }
        let (end, last) = ((END / 2) as i64, META.rows as i64 - 1);
        grid.set(end - 1, last, '▐', STEEL);
        grid.set(META.cols as i64 - end, last, '▌', STEEL);
    }
}
