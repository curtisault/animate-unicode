//! spider: a spider drops from the hub of its web in the corner on a line of
//! silk, bounces at the end of it with its legs going, then climbs back up.
//! After ascii.rest's spider.ts (MIT, @bas3line).
//!
//! The drop, the bounce and the climb are ascii.rest's. The web is braille,
//! fine hexagonal rings and spokes behind the spider, and the silk a thread
//! of dots; the spider is drawn in octants, head down, a round abdomen over
//! a smaller head and eight jointed legs that step between two poses. In
//! colour it is a black widow, glossy black with a red hourglass, hanging in
//! a pale grey web.

use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece};

/// The spider's meta.
pub static META: Meta = Meta {
    name: "spider",
    slug: "spider",
    category: Category::Creatures,
    note: "an octant black widow dropping on braille silk from its web, and back",
    cols: 28,
    rows: 24,
    fps: 15,
    charset: Charset::Extended,
    // The web; the silk; the body; its gloss; the hourglass; the legs.
    palette: Some(&["#5c6278", "#c8ccd8", "#1c1a1e", "#5a5866", "#e02828", "#2c2a30"]),
    ground: Some("#0a0b10"),
    ..Meta::DEFAULT
};

const WEB: u8 = 0;
const SILK: u8 = 1;
const BODY: u8 = 2;
const GLOSS: u8 = 3;
const HOURGLASS: u8 = 4;
const LEGS: u8 = 5;
/// Seconds a loop; the web's hub (column, row); its hexagonal rings, by how
/// many rows they sit from the hub; the rows the spider's top hangs at on the
/// web and at the end of its line.
const LOOP: f64 = 10.0;
const HUB: (f64, f64) = (10.0, 4.0);
const RINGS: [f64; 2] = [3.0, 6.0];
const TOP: f64 = 3.0;
const LOW: f64 = 15.6;

fn ease(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Dots along a line between two points in cells.
fn line(out: &mut Vec<(i64, i64)>, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
    let (a, b) = ((x0 * 2.0, y0 * 4.0), (x1 * 2.0, y1 * 4.0));
    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil().max(1.0) as i64;
    for s in 0..=n {
        let f = s as f64 / n as f64;
        out.push(((a.0 + (b.0 - a.0) * f).floor() as i64, (a.1 + (b.1 - a.1) * f).floor() as i64));
    }
}

/// The scene: the web's dots, and the braille and octant canvases.
pub struct Spider {
    web: Vec<(i64, i64)>,
    lines: ColorDots,
    body: ColorOctants,
}

/// A spider; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    // Hexagonal rings, a row apart for every two columns, and eight spokes
    // over them, out to the ceiling and the wall and just past the last ring
    // elsewhere.
    let mut web = vec![];
    let (hx, hy) = (HUB.0 + 0.5, HUB.1 + 0.5);
    let k_out = RINGS[RINGS.len() - 1];
    let hex = |k: f64, a: f64| (hx + 2.0 * k * a.cos(), hy + k * a.sin());
    for &k in &RINGS {
        for s in 0..6 {
            let (a0, a1) = (f64::from(s) * std::f64::consts::FRAC_PI_3, f64::from(s + 1) * std::f64::consts::FRAC_PI_3);
            line(&mut web, hex(k, a0), hex(k, a1));
        }
    }
    for s in 0..8 {
        let a = f64::from(s) * std::f64::consts::FRAC_PI_4;
        let far = if (1..=3).contains(&s) { k_out + 0.6 } else { 30.0 };
        line(&mut web, (hx, hy), (hx + 2.0 * far * a.cos(), hy + far * a.sin()));
    }
    Box::new(Spider { web, lines: ColorDots::new(META.cols, META.rows), body: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Spider {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let u = (t + 2.4).rem_euclid(LOOP);
        // Where the spider's top is, and how fast the legs go.
        let (mut y, mut rate, mut climb) = (TOP, 0.0, None);
        if (1.0..3.4).contains(&u) {
            y = TOP + (LOW - TOP) * ((u - 1.0) / 2.4).powi(2);
            rate = 3.0;
        } else if (3.4..6.0).contains(&u) {
            // The line stops paying out and the silk springs back: it leaves at the speed it fell.
            let (s, w, v) = (u - 3.4, 6.0, 2.0 * (LOW - TOP) / 2.4);
            y = LOW + v / w * (-s * 1.3).exp() * (s * w).sin();
            rate = if s < 1.8 { 6.0 } else { 0.0 };
        } else if (6.0..9.4).contains(&u) {
            // Hand over hand: a lurch up with each pull, then a pause.
            let pull = (u - 6.0) / 3.4 * 8.0;
            y = LOW - (LOW - TOP) * ((pull.floor() + ease(pull.fract() * 1.6)) / 8.0);
            climb = Some(pull.floor() as i64 & 1);
        }
        let sway = if u > 3.0 && u < 7.0 { 0.8 * ((u - 3.0) * 2.1).sin() * (-(u - 3.0) * 0.6).exp() } else { 0.0 };
        let cx = HUB.0 + 0.5 + sway;
        // Legs step between poses, the two sides out of turn with each other.
        let step = match climb {
            Some(c) => c,
            None if rate > 0.0 => (u * rate).floor() as i64 & 1,
            None => i64::from(u > 0.3 && u < 0.55),
        };
        self.lines.clear();
        let (top, abd) = (y, y + 1.9); // the spinnerets, and the abdomen's middle
        for &(x, yy) in &self.web {
            // Off the hub, the lower spoke is where the line hangs instead; the
            // web is kept clear of the spider.
            let near = ((x as f64 / 2.0 - cx) / 6.0).powi(2) + ((yy as f64 / 4.0 - (top + 3.5)) / 4.5).powi(2) < 1.0;
            if !near {
                self.lines.set(x, yy, WEB);
            }
        }
        if top > HUB.1 + 1.0 {
            let mut silk = vec![];
            line(&mut silk, (HUB.0 + 0.5, HUB.1 + 0.5), (cx, top));
            for (x, yy) in silk {
                self.lines.set(x, yy, SILK);
            }
        }
        self.body.clear();
        // The body, head down: a round abdomen with a red hourglass under it,
        // a glint of gloss, and the smaller head below.
        let (w, h) = (self.body.width() as i64, self.body.height() as i64);
        for j in ((top + 0.2) * 4.0) as i64..((top + 6.0) * 4.0) as i64 {
            for i in ((cx - 3.0) * 2.0) as i64..((cx + 3.0) * 2.0) as i64 {
                if !(0..w).contains(&i) || !(0..h).contains(&j) {
                    continue;
                }
                let (x, yy) = ((i as f64 + 0.5) / 2.0 - cx, (j as f64 + 0.5) / 4.0);
                let a = (x / 2.4).powi(2) + ((yy - abd) / 1.6).powi(2);
                let head = (x / 1.3).powi(2) + ((yy - (top + 4.4)) / 0.9).powi(2);
                if a < 1.0 {
                    let hour = x.abs() < 0.25 + (yy - abd).abs() * 0.5 && (yy - abd).abs() < 0.9;
                    let gloss = ((x + 1.0) / 0.5).powi(2) + ((yy - abd + 0.7) / 0.3).powi(2) < 1.0;
                    self.body.set(
                        i,
                        j,
                        if hour {
                            HOURGLASS
                        } else if gloss {
                            GLOSS
                        } else {
                            BODY
                        },
                    );
                } else if head < 1.0 {
                    self.body.set(i, j, BODY);
                }
            }
        }
        // Eight legs, four a side from where the abdomen meets the head: the
        // front pairs reach down, the back pairs up, each bent at the knee
        // with its foot hanging below it. The sides step out of turn.
        for side in [-1.0, 1.0] {
            let pose = if side < 0.0 { step } else { 1 - step };
            for k in 0..4 {
                let kf = f64::from(k);
                let root = (cx + side * 0.8, top + 3.3 - kf * 0.25);
                let lift = [1.7, 0.7, -0.6, -1.7][k as usize];
                let reach = if (pose + k as i64) % 2 == 1 { 0.45 } else { -0.35 };
                let knee = (cx + side * 2.7, top + 3.3 + lift - reach);
                let foot = (cx + side * 4.3, top + 3.3 + lift * 1.5 + 1.0 - reach * 0.5);
                let mut pts = vec![];
                line(&mut pts, root, knee);
                line(&mut pts, knee, foot);
                for (x, yy) in pts {
                    self.body.set(x, yy, LEGS);
                }
            }
        }
        grid.clear();
        self.lines.draw(grid, 0, 0);
        self.body.draw(grid, 0, 0);
    }
}
