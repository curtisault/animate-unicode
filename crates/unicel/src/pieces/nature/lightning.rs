//! lightning: a storm over low hills in driving rain. Every few seconds a
//! bolt zigzags down from the cloud to the ground, forks as it strikes,
//! flickers through a restrike, lights the cloud and the land, and fades.
//! After ascii.rest's lightning.ts (MIT, @bas3line).
//!
//! The storm's timing, the bolts' paths and the cloud are ascii.rest's; it is
//! drawn in braille. The bolt is a jagged line at a dot's resolution, doubled
//! while it is brightest, its forks finer; the rain is fine slanting streaks;
//! the cloud a braille texture thick toward its underside. In colour the
//! storm is slate until a flash lights the cloud violet-white over the bolt,
//! the bolt white with violet forks, the rain catching the light, the hills'
//! slopes lit from the skyline down.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};

/// Lightning's meta.
pub static META: Meta = Meta {
    name: "lightning",
    slug: "lightning",
    category: Category::Nature,
    note: "a white braille bolt forking out of a slate cloud lit violet, in rain",
    cols: 64,
    rows: 22,
    fps: 20,
    charset: Charset::Extended,
    // The cloud, dark to lit; the bolt; its forks; rain, dim and lit; the
    // hills' outline; the land in the flash.
    palette: Some(&["#262a38", "#40465e", "#767ca4", "#c8c8f4", "#ffffff", "#a890ff", "#3a4658", "#8a9ac0", "#2a3628", "#56645a"]),
    ground: Some("#07080d"),
    ..Meta::DEFAULT
};

const CLOUD: Ramp = Ramp::new(0, 4);
const BOLT: u8 = 4;
const FORK: u8 = 5;
const RAIN: u8 = 6;
const RAIN_LIT: u8 = 7;
const HILLS: u8 = 8;
const LAND_LIT: u8 = 9;
/// Seconds a loop; strikes (start within the loop, column the bolt leaves
/// the cloud); the row the cloud's flat base sits on; columns a second the
/// billows drift; and the raindrops.
const LOOP: f64 = 12.0;
const STRIKES: [(f64, f64); 5] = [(11.94, 27.0), (2.44, 44.0), (4.62, 17.0), (7.6, 51.0), (9.9, 34.0)];
const DECK: f64 = 2.6;
const DRIFT: f64 = 0.35;
const DROPS: usize = 130;

fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn ease(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn noise(x: f64, y: f64) -> f64 {
    let (i, j) = (x.floor() as i64, y.floor() as i64);
    let (u, v) = (ease(x - x.floor()), ease(y - y.floor()));
    let (a, b, c, d) = (hash(i, j), hash(i + 1, j), hash(i, j + 1), hash(i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// How bright a strike is, a seconds after it starts: a faint leader, the
/// return stroke, a dark gap, a restrike, then a fading afterglow.
fn flash(a: f64) -> f64 {
    match a {
        a if !(0.0..=1.0).contains(&a) => 0.0,
        a if a < 0.05 => 0.3,
        a if a < 0.15 => 1.0,
        a if a < 0.2 => 0.15,
        a if a < 0.3 => 0.9,
        a => 0.9 * (-(a - 0.3) / 0.18).exp(),
    }
}

fn hill(c: f64) -> f64 {
    META.rows as f64 - 3.6 - 1.2 * (c * 0.07 + 0.8).sin() - 0.5 * (c * 0.19 + 2.1).sin()
}

/// A bolt: when it starts, the column it leaves the cloud at, where it
/// strikes and the ground's row there, and its channel and forks as points
/// (column, row) a row apart.
struct Bolt {
    start: f64,
    x0: f64,
    x1: f64,
    ground: f64,
    main: Vec<(f64, f64)>,
    forks: Vec<Vec<(f64, f64)>>,
}

/// The storm: the billows (place, width, depth), the bolts, the raindrops
/// (place, phase, speed, length), and the dots.
pub struct Lightning {
    billows: Vec<(f64, f64, f64)>,
    bolts: Vec<Bolt>,
    drops: Vec<(f64, f64, f64, f64)>,
    dots: ColorDots,
}

impl Lightning {
    /// How far the cloud's underside hangs at a column: a flat base with
    /// billows hanging from it, drifting, thinning toward the sides.
    fn bottom(&self, c: f64, t: f64) -> f64 {
        let wrap = META.cols as f64 + 24.0;
        let s = side(c);
        let mut y = DECK - 1.2 * (1.0 - s);
        for &(bx, bw, bd) in &self.billows {
            let x = (c + 0.5 - bx + t * DRIFT).rem_euclid(wrap) - 12.0;
            if x.abs() < bw {
                y = y.max(DECK + bd * s * (1.0 - (x / bw).powi(2)).sqrt());
            }
        }
        y
    }
}

fn side(c: f64) -> f64 {
    ((c + 1.0).min(META.cols as f64 - c) / 16.0).min(1.0)
}

/// Lightning; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let cols = META.cols as f64;
    let mut rand = Mulberry32::new(7);
    let wrap = cols + 24.0;
    let mut billows = vec![];
    let mut x = 0.0;
    while x < wrap - 4.0 {
        billows.push((x, 3.5 + rand.next_f64() * 3.5, 1.0 + rand.next_f64() * 2.6));
        x += 6.0 + rand.next_f64() * 5.0;
    }
    let mut storm = Lightning { billows, bolts: vec![], drops: vec![], dots: ColorDots::new(META.cols, META.rows) };
    // A bolt: one point a row from the cloud to the ground, kicked a column
    // or two at a time in runs that switch side, always going down. Forks
    // leave it partway, run a few rows away from it, and give out in the air.
    for (n, &(start, x0)) in STRIKES.iter().enumerate() {
        let mut rand = Mulberry32::new(31 + n as u32 * 17);
        let r0 = storm.bottom(x0, start).floor() + 1.0;
        let ground = hill(x0 + 0.5).floor();
        let walk = |rand: &mut Mulberry32, mut c: f64, from: f64, to: f64, lean: f64, main: bool| {
            let (mut dir, mut run, mut pts) = (lean, 0, vec![(c + 0.5, from)]);
            let mut r = from;
            while r < to {
                if run <= 0 {
                    dir = if main {
                        if rand.next_f64() < 0.5 {
                            -dir
                        } else {
                            dir
                        }
                    } else {
                        lean
                    };
                    run = 1 + (rand.next_f64() * 3.0) as i64;
                }
                run -= 1;
                let still = if main { 0.22 } else { 0.15 };
                let dx = if rand.next_f64() < still { 0.0 } else { dir * if rand.next_f64() < 0.3 { 2.0 } else { 1.0 } };
                c += dx;
                r += 1.0;
                // A little jitter within the cell, so the channel is jagged at a dot's scale too.
                pts.push((c + 0.5 + (rand.next_f64() - 0.5) * 0.5, r));
            }
            pts
        };
        let lean = if rand.next_f64() < 0.5 { -1.0 } else { 1.0 };
        let main = walk(&mut rand, x0, r0, ground, lean, true);
        let mut forks = vec![];
        for f in 0..2 + (rand.next_f64() * 2.0) as usize {
            let k = (main.len() as f64 * (0.2 + 0.5 * rand.next_f64())) as usize;
            let (c, r) = main[k.min(main.len() - 2)];
            let dx = main[k.min(main.len() - 2) + 1].0 - c;
            let lean = if dx < -0.3 {
                1.0
            } else if dx > 0.3 {
                -1.0
            } else if f % 2 == 1 {
                1.0
            } else {
                -1.0
            };
            let end = (ground - 2.0).min(r + 3.0 + (rand.next_f64() * 4.0).floor());
            forks.push(walk(&mut rand, c - 0.5 + lean, r, end, lean, false));
        }
        let x1 = main.last().map_or(x0, |p| p.0);
        storm.bolts.push(Bolt { start, x0, x1, ground, main, forks });
    }
    // Rain on the wind, a column left for every row it falls: far drops
    // short and slow, near ones longer streaks and faster.
    storm.drops = (0..DROPS)
        .map(|_| {
            let near = rand.next_f64();
            (
                rand.next_f64() * (cols + 20.0),
                rand.next_f64(),
                0.9 + near * 0.9,
                if near < 0.25 {
                    1.0
                } else if near < 0.7 {
                    2.0
                } else {
                    3.0
                },
            )
        })
        .collect();
    Box::new(storm)
}

/// Dots along a polyline of (column, row) points, `thick` dots wide.
fn path(dots: &mut ColorDots, pts: &[(f64, f64)], upto: usize, thick: i64, color: u8) {
    for p in pts.windows(2).take(upto) {
        let ((x0, y0), (x1, y1)) = ((p[0].0 * 2.0, p[0].1 * 4.0), (p[1].0 * 2.0, p[1].1 * 4.0));
        let n = (x1 - x0).abs().max(y1 - y0).ceil().max(1.0) as i64;
        for s in 0..=n {
            let f = s as f64 / n as f64;
            let (x, y) = ((x0 + (x1 - x0) * f).floor() as i64, (y0 + (y1 - y0) * f).floor() as i64);
            for d in 0..thick {
                dots.set(x + d, y, color);
            }
        }
    }
}

impl Piece for Lightning {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        let u = t.rem_euclid(LOOP);
        let (mut lit, mut bolt, mut age) = (0.0, None, 0.0);
        for (k, b) in self.bolts.iter().enumerate() {
            for a in [u - b.start, u - b.start + LOOP] {
                let f = flash(a);
                if f > lit {
                    (lit, bolt, age) = (f, Some(k), a);
                }
            }
        }
        self.dots.clear();
        // Rain first, from the cloud to the hills: streaks slanting with the wind.
        let fall = rows - 3.0;
        for &(dx, dy, speed, len) in &self.drops {
            let y = 3.0 + (dy + t * speed * 0.8).rem_euclid(1.0) * fall;
            for k in 0..(len * 4.0) as i64 {
                let yy = y - k as f64 / 4.0;
                let c = (dx - (yy - 3.0)).rem_euclid(cols + 20.0) - 10.0;
                if c >= 0.0 && c < cols && yy < hill(c + 0.5) - 0.6 {
                    self.dots.set((c * 2.0).floor() as i64, (yy * 4.0).floor() as i64, if lit > 0.3 { RAIN_LIT } else { RAIN });
                }
            }
        }
        // The cloud, dim until a flash lights it all over, brightest over the
        // bolt; thicker toward the underside, broken into rags at the sides.
        let near_x = bolt.map(|k| self.bolts[k].x0);
        for i in 0..w {
            let c = (i as f64 + 0.5) / 2.0;
            let y = self.bottom(c - 0.5, t);
            let near = near_x.map_or(0.0, |x0| (-((c - x0) / 12.0).powi(2)).exp());
            let light = 0.5 + 0.35 * lit + 0.6 * lit * near;
            for j in 0..((y * 4.0).ceil() as i64).min(h) {
                let r = (j as f64 + 0.5) / 4.0;
                let n = noise(c * 0.2 + t * DRIFT * 0.2, r * 0.8 + 3.0);
                let edge = y - r < 0.3;
                if !edge && n < 0.45 - 0.2 * r + 0.5 * (1.0 - side(c)) {
                    continue;
                }
                let v = ((0.3 + 0.3 * (1.0 - (y - r - 1.0) / 2.5).max(0.0) + 0.6 * (n - 0.5)) * light * (0.5 + 0.5 * side(c))).clamp(0.0, 1.0);
                if edge || dither(i as usize, j as usize, 0.25 + v) {
                    self.dots.set(i, j, CLOUD.at(v * (0.6 + 0.6 * lit)));
                }
            }
        }
        // The land: a dark outline of hills; a flash lights the slopes from the
        // skyline down, more of them the brighter it is.
        for i in 0..w {
            let c = (i as f64 + 0.5) / 2.0;
            let y = hill(c);
            self.dots.set(i, (y * 4.0).floor() as i64, HILLS);
            for j in (y * 4.0).floor() as i64 + 1..h {
                let g = lit * (1.1 - (j as f64 / 4.0 - y) / 4.0) - 0.2;
                if g > 0.08 && dither(i as usize, j as usize, g) {
                    self.dots.set(i, j, LAND_LIT);
                }
            }
        }
        if let Some(k) = bolt.filter(|_| lit > 0.12) {
            let b = &self.bolts[k];
            // The leader only gets partway down; the strokes show it all, forks
            // too while it is brightest; the afterglow keeps just the channel.
            let reach = if age < 0.05 { (b.main.len() as f64 * age / 0.05 * 0.8) as usize } else { b.main.len() };
            path(&mut self.dots, &b.main, reach, if lit > 0.6 { 2 } else { 1 }, if lit > 0.4 { BOLT } else { FORK });
            if lit > 0.6 {
                for f in &b.forks {
                    path(&mut self.dots, f, f.len(), 1, FORK);
                }
            }
            // Where it strikes, a splash of light on the ground.
            if lit > 0.4 && age >= 0.05 {
                let (cx, cy) = ((b.x1 * 2.0) as i64, (b.ground * 4.0) as i64);
                for (dx, dy) in [(-4, 0), (-3, 0), (-2, -1), (-1, -1), (0, -1), (0, 0), (1, -1), (1, 0), (2, -1), (3, 0), (4, 0), (-2, -2), (2, -2), (0, -3)] {
                    self.dots.set(cx + dx, cy + dy, BOLT);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
