//! langton's ant: an ant that turns right on a blank square and left on an
//! inked one, flipping each square it leaves. Order, then chaos, then the
//! highway. After ascii.rest's langtons-ant.ts (MIT, @bas3line).
//!
//! One square is one octant dot, eight to a character, so the 72×25
//! characters show a 144×100 stretch of board, four times what ascii.rest
//! could. Each square is coloured by how long ago the ant last flipped it,
//! white-hot where it has just been, cooling through orange to deep blue, so
//! the highway is drawn hot as it leaves the cooled chaos behind.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};

/// Langton's ant's meta.
pub static META: Meta = Meta {
    name: "langton's ant",
    slug: "langtons-ant",
    category: Category::Generative,
    note: "langton's ant in octants, each square coloured by when it flipped",
    cols: 72,
    rows: 25,
    fps: 30,
    charset: Charset::Extended,
    // Coldest first; then the ant.
    palette: Some(&["#1c2a5c", "#2c4a8a", "#3a72b0", "#5aa0c8", "#d8a050", "#f08a3a", "#ffc070", "#fff4dc", "#ff3a5a"]),
    ground: Some("#07080d"),
    ..Meta::DEFAULT
};

/// The board, in squares.
const N: i64 = 260;
/// The board square at the frame's top left, from the start.
const X0: i64 = 62;
const Y0: i64 = -40;
/// Play time against steps taken: the cycle opens as the highway sets off,
/// so the first frame already shows all three phases; then the board fades
/// and it starts again from blank. The highway starts at step 9977; it runs on
/// to 13 400 here (ascii.rest stops at 11 330), far enough to reach the bottom
/// of the larger frame.
const KEYS: [(f64, f64); 8] = [(0.0, 10350.0), (2.0, 13400.0), (3.2, 13400.0), (3.2, 0.0), (4.3, 100.0), (5.6, 520.0), (12.4, 9977.0), (13.2, 10350.0)];
const PERIOD: f64 = 13.2;
const FADE: (f64, f64) = (2.4, 3.2);
/// Steps over which a flipped square cools from white to blue.
const COOL: f64 = 3000.0;
const HEAT: Ramp = Ramp::new(0, 8);
const ANT: u8 = 8;

/// The board, when each square was last flipped, the ant, and the dots.
pub struct LangtonsAnt {
    board: Vec<u8>,
    flipped: Vec<u32>,
    x: i64,
    y: i64,
    dir: usize,
    steps: u32,
    fade_at: Vec<f32>,
    dots: ColorOctants,
}

impl LangtonsAnt {
    fn reset(&mut self) {
        self.board.fill(0);
        self.flipped.fill(0);
        (self.x, self.y, self.dir, self.steps) = (N / 2, N / 2, 0, 0);
    }

    fn step(&mut self) {
        const DX: [i64; 4] = [0, 1, 0, -1];
        const DY: [i64; 4] = [-1, 0, 1, 0];
        let i = (self.y * N + self.x) as usize;
        self.dir = (self.dir + if self.board[i] == 1 { 3 } else { 1 }) & 3;
        self.board[i] ^= 1;
        self.steps += 1;
        self.flipped[i] = self.steps;
        self.x = (self.x + DX[self.dir]).rem_euclid(N);
        self.y = (self.y + DY[self.dir]).rem_euclid(N);
    }

    /// The board square under dot (c, r); the frame is mirrored so the
    /// highway runs off to the lower right.
    fn square(c: i64, r: i64) -> usize {
        ((N / 2 + Y0 + r) * N + N / 2 + X0 - c) as usize
    }
}

/// Steps the ant should have taken at point `p` of the cycle.
fn target(p: f64) -> u32 {
    let mut k = 1;
    while KEYS[k].0 < p {
        k += 1;
    }
    let ((p0, s0), (p1, s1)) = (KEYS[k - 1], KEYS[k]);
    (s0 + (s1 - s0) * (p - p0) / (p1 - p0)).round() as u32
}

/// Langton's ant; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(fresh())
}

/// An ant on a blank board.
fn fresh() -> LangtonsAnt {
    let dots = ColorOctants::new(META.cols, META.rows);
    // Each square fades at its own moment.
    let mut rand = Mulberry32::new(7);
    let fade_at = (0..dots.width() * dots.height()).map(|_| rand.next_f64() as f32).collect();
    let mut ant = LangtonsAnt { board: vec![0; (N * N) as usize], flipped: vec![0; (N * N) as usize], x: 0, y: 0, dir: 0, steps: 0, fade_at, dots };
    ant.reset();
    ant
}

impl Piece for LangtonsAnt {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let p = t.rem_euclid(PERIOD);
        let want = target(p);
        if want < self.steps {
            self.reset();
        }
        while self.steps < want {
            self.step();
        }
        let fade = if p > FADE.0 && p <= FADE.1 { (p - FADE.0) / (FADE.1 - FADE.0) } else { 0.0 };
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        self.dots.clear();
        for r in 0..h {
            for c in 0..w {
                let i = Self::square(c, r);
                if self.board[i] == 0 || f64::from(self.fade_at[(r * w + c) as usize]) < fade {
                    continue;
                }
                let age = f64::from(self.steps - self.flipped[i]) / COOL;
                self.dots.set(c, r, HEAT.at(1.0 - age.sqrt()));
            }
        }
        // The ant, a bright knot where it stands.
        if fade == 0.0 {
            let (ac, ar) = (N / 2 + X0 - self.x, self.y - N / 2 - Y0);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                self.dots.set(ac + dx, ar + dy, ANT);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ant_builds_its_highway_after_10000_steps() {
        let mut ant = fresh();
        // The highway repeats every 104 steps, moving the ant two squares diagonally.
        for _ in 0..11000 {
            ant.step();
        }
        let (x0, y0, d0) = (ant.x, ant.y, ant.dir);
        for _ in 0..104 {
            ant.step();
        }
        assert_eq!(((ant.x - x0).abs(), (ant.y - y0).abs(), ant.dir), (2, 2, d0));
    }
}
