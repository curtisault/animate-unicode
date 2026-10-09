//! maze: a recursive backtracker carving a maze out of a grid, its stack
//! shown as a trail, then the way through drawn in, then the walls close up
//! and a new maze starts. After ascii.rest's maze.ts (MIT, @bas3line).
//!
//! In braille a step of the maze is four dots, a wall one dot and the
//! corridor three, so the 61×21 cells hold a 30 by 20 maze where box drawing
//! held 15 by 10: four times the rooms. Uncarved ground is stippled, the
//! backtracker's stack is a cyan thread with a white head, and the way
//! through is drawn in amber down the middle of the corridors.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// The maze's meta.
pub static META: Meta = Meta {
    name: "maze",
    slug: "maze",
    category: Category::Generative,
    note: "a braille maze four times the rooms, carved, solved in amber, remade",
    cols: 61,
    rows: 21,
    fps: 20,
    charset: Charset::Extended,
    // Walls, uncarved ground, the stack, its head, the way through.
    palette: Some(&["#8a96b0", "#3a4258", "#3cc8e0", "#ffffff", "#ffb43c"]),
    ground: Some("#0a0c12"),
    ..Meta::DEFAULT
};

/// Rooms across and down, and dots from one wall to the next.
const MW: usize = 30;
const MH: usize = 20;
const STEP: i64 = 4;
/// The maze's top left, in dots.
const X0: i64 = 0;
const Y0: i64 = 1;
/// Backtracker moves a second, then seconds to draw the way, hold it, and
/// sweep the walls back; and where in the first cycle play begins.
const CARVE: f64 = 150.0;
const SOLVE: f64 = 2.4;
const HOLD: f64 = 2.2;
const CLOSE: f64 = 1.2;
const START: f64 = 4.0;
const WALL: u8 = 0;
const GROUND: u8 = 1;
const STACK: u8 = 2;
const HEAD: u8 = 3;
const WAY: u8 = 4;

/// One maze: the backtracker's moves (`Some(n)` steps forward into room n,
/// `None` steps back), and the way from the top left room to the bottom right.
#[derive(Debug, PartialEq)]
struct Plan {
    moves: Vec<Option<usize>>,
    way: Vec<usize>,
}

/// The backtracker on a `mw × mh` grid, from ascii.rest's seeded generator.
fn build(seed: u32, mw: usize, mh: usize) -> Plan {
    let mut rand = Mulberry32::new(seed);
    let n = mw * mh;
    let (mut seen, mut parent) = (vec![false; n], vec![usize::MAX; n]);
    let (mut moves, mut stack) = (vec![], vec![0]);
    seen[0] = true;
    while let Some(&c) = stack.last() {
        let (x, y) = (c % mw, c / mw);
        let mut next = vec![];
        if x > 0 && !seen[c - 1] {
            next.push(c - 1);
        }
        if x < mw - 1 && !seen[c + 1] {
            next.push(c + 1);
        }
        if y > 0 && !seen[c - mw] {
            next.push(c - mw);
        }
        if y < mh - 1 && !seen[c + mw] {
            next.push(c + mw);
        }
        if next.is_empty() {
            stack.pop();
            if !stack.is_empty() {
                moves.push(None);
            }
            continue;
        }
        let m = next[(rand.next_f64() * next.len() as f64) as usize];
        seen[m] = true;
        parent[m] = c;
        stack.push(m);
        moves.push(Some(m));
    }
    let mut way = vec![n - 1];
    while *way.last().unwrap() != 0 {
        way.push(parent[*way.last().unwrap()]);
    }
    way.reverse();
    Plan { moves, way }
}

/// The maze on its way: walls above and left of every room, which rooms are
/// carved, the stack, and the dots.
pub struct Maze {
    plans: Vec<(usize, Plan)>,
    above: Vec<bool>,
    left: Vec<bool>,
    state: Vec<u8>,
    stack: Vec<usize>,
    dots: ColorDots,
    fresh: ColorDots,
}

fn centre(n: usize) -> (i64, i64) {
    (X0 + (n % MW) as i64 * STEP + STEP / 2, Y0 + (n / MW) as i64 * STEP + STEP / 2)
}

impl Maze {
    fn plan(&mut self, k: usize) -> usize {
        if let Some(i) = self.plans.iter().position(|p| p.0 == k) {
            return i;
        }
        if self.plans.len() > 2 {
            self.plans.remove(0);
        }
        self.plans.push((k, build(1009 + k as u32 * 7919, MW, MH)));
        self.plans.len() - 1
    }

    /// Plan `p` carved `k` moves in, its way drawn `solved` (0 to 1) of the way through.
    fn draw(&mut self, p: usize, k: usize, solved: f64, into_fresh: bool) {
        let plan = &self.plans[p].1;
        // Walls above each room (and below the last row), left of each (and
        // right of the last column); the way in top left and out bottom right.
        self.above.fill(true);
        self.left.fill(true);
        self.left[0] = false;
        self.left[(MH - 1) * (MW + 1) + MW] = false;
        self.state.fill(0);
        self.state[0] = 1;
        self.stack.clear();
        self.stack.push(0);
        for mv in plan.moves.iter().take(k) {
            let c = *self.stack.last().unwrap();
            let Some(m) = *mv else {
                let done = self.stack.pop().unwrap();
                self.state[done] = 2;
                continue;
            };
            match m as i64 - c as i64 {
                1 => self.left[(c / MW) * (MW + 1) + c % MW + 1] = false,
                -1 => self.left[(c / MW) * (MW + 1) + c % MW] = false,
                d if d == MW as i64 => self.above[m] = false,
                _ => self.above[c] = false,
            }
            self.state[m] = 1;
            self.stack.push(m);
        }
        if k >= plan.moves.len() {
            self.state[0] = 2;
        }
        let dots = if into_fresh { &mut self.fresh } else { &mut self.dots };
        dots.clear();
        for j in 0..=MH {
            for i in 0..MW {
                if self.above[j * MW + i] {
                    let (x, y) = (X0 + i as i64 * STEP, Y0 + j as i64 * STEP);
                    dots.line(x, y, x + STEP, y, WALL);
                }
            }
        }
        for j in 0..MH {
            for i in 0..=MW {
                if self.left[j * (MW + 1) + i] {
                    let (x, y) = (X0 + i as i64 * STEP, Y0 + j as i64 * STEP);
                    dots.line(x, y, x, y + STEP, WALL);
                }
            }
        }
        // Uncarved rooms are stippled ground.
        for n in 0..MW * MH {
            if self.state[n] == 0 {
                let (cx, cy) = centre(n);
                for (dx, dy) in [(-1, -1), (1, 1)] {
                    dots.set(cx + dx, cy + dy, GROUND);
                }
            }
        }
        // The stack as a thread through the rooms it holds, its head a block.
        if k < plan.moves.len() {
            for w in self.stack.windows(2) {
                let ((x0, y0), (x1, y1)) = (centre(w[0]), centre(w[1]));
                dots.line(x0, y0, x1, y1, STACK);
            }
            let (hx, hy) = centre(*self.stack.last().unwrap());
            for dy in -1..=1 {
                for dx in -1..=1 {
                    dots.set(hx + dx, hy + dy, HEAD);
                }
            }
        }
        // The way through, from the entrance, as far as `solved`.
        if solved > 0.0 {
            let mut pts = vec![(X0 - 1, centre(0).1)];
            pts.extend(plan.way.iter().map(|&n| centre(n)));
            pts.push((X0 + MW as i64 * STEP + 1, centre(MW * MH - 1).1));
            let mut cells = vec![];
            for w in pts.windows(2) {
                let ((mut x, mut y), (x1, y1)) = (w[0], w[1]);
                while (x, y) != (x1, y1) {
                    x += (x1 - x).signum();
                    y += (y1 - y).signum();
                    cells.push((x, y));
                }
            }
            let shown = (solved * cells.len() as f64).round() as usize;
            for &(x, y) in cells.iter().take(shown) {
                dots.set(x, y, WAY);
            }
        }
    }
}

/// A maze; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Maze {
        plans: vec![],
        above: vec![true; (MH + 1) * MW],
        left: vec![true; MH * (MW + 1)],
        state: vec![0; MW * MH],
        stack: vec![],
        dots: ColorDots::new(META.cols, META.rows),
        fresh: ColorDots::new(META.cols, META.rows),
    })
}

impl Piece for Maze {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        // The backtracker steps into every room but the first once and back once.
        let carved = (2 * (MW * MH - 1)) as f64 / CARVE;
        let period = carved + SOLVE + HOLD + CLOSE;
        let n = ((t + START) / period).floor();
        let p = t + START - n * period;
        let plan = self.plan(n as usize);
        let k = ((p * CARVE).floor() as usize).min(self.plans[plan].1.moves.len());
        let solved = ((p - carved) / SOLVE).clamp(0.0, 1.0);
        self.draw(plan, k, solved, false);
        grid.clear();
        self.dots.draw(grid, 0, 0);
        let close = (p - carved - SOLVE - HOLD) / CLOSE;
        if close <= 0.0 {
            return;
        }
        // The walls sweep back in from the left a column of rooms at a time,
        // ready for the next maze.
        let next = self.plan(n as usize + 1);
        self.draw(next, 0, 0.0, true);
        let rooms = ((close * (MW + 1) as f64).floor() as usize).min(MW);
        let edge = ((X0 + rooms as i64 * STEP) / 2 + 1) as usize;
        let mut fresh = Grid::new(META.cols, META.rows);
        self.fresh.draw(&mut fresh, 0, 0);
        for r in 0..META.rows {
            for c in 0..edge.min(META.cols) {
                let i = r * META.cols + c;
                grid.cells_mut()[i] = fresh.cells()[i];
                grid.colors_mut()[i] = fresh.colors()[i];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ascii.rest's build(1009) at its own size, 15 by 10, from node.
    #[test]
    fn the_backtracker_matches_the_typescript() {
        let plan = build(1009, 15, 10);
        assert_eq!(plan.moves.len(), 298);
        let first: Vec<usize> = plan.moves.iter().take(24).map(|m| m.unwrap()).collect();
        assert_eq!(first, [15, 30, 31, 16, 1, 2, 17, 18, 19, 20, 35, 34, 49, 48, 47, 62, 63, 64, 79, 94, 95, 110, 111, 112]);
        assert_eq!(
            plan.way,
            [
                0, 15, 30, 31, 16, 1, 2, 17, 18, 19, 20, 35, 34, 49, 48, 47, 62, 63, 64, 79, 94, 95, 110, 111, 112, 113, 128, 143, 144, 129, 130, 145, 146,
                131, 132, 133, 134, 149
            ]
        );
    }
}
