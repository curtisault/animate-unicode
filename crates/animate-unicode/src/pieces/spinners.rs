//! spinners: loading spinners in a grid, each in a three-row box with its
//! name underneath. Pass names to pick some of them, in that order. After
//! ascii.rest's spinners.ts (MIT, @bas3line).
//!
//! ascii.rest's twelve, and twelve more that only Unicode can spin: braille
//! dots and a braille snake, a braille wave and a filling cell, the arcs,
//! quarters, halves and corners of the geometric shapes, an arrow going round,
//! and a block that grows round its corners. Each spinner has its own colour.

use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// The spinners' meta.
pub static META: Meta = Meta {
    name: "spinners",
    slug: "spinners",
    category: Category::Ui,
    note: "twenty-four loading spinners, half of them braille and shapes",
    cols: 66,
    rows: 25,
    fps: 12,
    charset: Charset::Extended,
    options: Some(r#"{"names": [], "labels": true, "speed": 1}"#),
    // Six hues for the spinners, the labels.
    palette: Some(&["#ff7a7a", "#f4b04a", "#d8e05a", "#6ad890", "#5ac0f0", "#b088f4", "#6a707c"]),
    ground: Some("#0d0e12"),
    ..Meta::DEFAULT
};

const LABEL: u8 = 6;

/// A spinner: its name, frames a step, and its picture at step i as one to three rows.
type Spinner = (&'static str, u32, fn(i64) -> Vec<String>);

/// Quadrant blocks by which of upper left, upper right, lower left, lower right are filled.
fn quad(ul: bool, ur: bool, ll: bool, lr: bool) -> char {
    const Q: [char; 16] = [' ', '▗', '▖', '▄', '▝', '▐', '▞', '▟', '▘', '▚', '▌', '▙', '▀', '▜', '▛', '█'];
    Q[(usize::from(ul) << 3) | (usize::from(ur) << 2) | (usize::from(ll) << 1) | usize::from(lr)]
}

/// 0 up to n - 1 and back down, step by step.
fn ping(n: i64, i: i64) -> i64 {
    let k = i.rem_euclid(2 * n - 2);
    if k < n {
        k
    } else {
        2 * n - 2 - k
    }
}

/// A w × h picture with glyphs at the given places.
fn plot(w: usize, h: usize, cells: &[(usize, usize, char)]) -> Vec<String> {
    let mut g = vec![vec![' '; w]; h];
    for &(x, y, ch) in cells {
        g[y][x] = ch;
    }
    g.into_iter().map(|r| r.into_iter().collect()).collect()
}

/// One glyph a step from a cycle.
fn cycle(glyphs: &str, i: i64) -> Vec<String> {
    let g: Vec<char> = glyphs.chars().collect();
    vec![g[i.rem_euclid(g.len() as i64) as usize].to_string()]
}

/// A braille cell from its dots, numbered 1 to 8 as braille numbers them.
fn braille(dots: &[u8]) -> char {
    char::from_u32(0x2800 + dots.iter().map(|d| 1u32 << (d - 1)).sum::<u32>()).unwrap_or(' ')
}

const SPINNERS: [Spinner; 24] = [
    ("line", 2, |i| {
        let states: [&[(usize, usize, char)]; 4] = [
            &[(2, 0, '│'), (2, 1, '│'), (2, 2, '│')],
            &[(3, 0, '╱'), (2, 1, '╱'), (1, 2, '╱')],
            &[(0, 1, '─'), (1, 1, '─'), (2, 1, '─'), (3, 1, '─'), (4, 1, '─')],
            &[(1, 0, '╲'), (2, 1, '╲'), (3, 2, '╲')],
        ];
        plot(5, 3, states[i.rem_euclid(4) as usize])
    }),
    ("dots", 2, |i| {
        // Three dots hopping in turn, half a row at a time.
        let mut cells = vec![];
        for d in 0..3 {
            let h = [0, 1, 2, 1, 0, 0, 0][(i - d * 2 + 70).rem_euclid(7) as usize];
            cells.push((d as usize * 2, if h == 2 { 0 } else { 1 }, if h == 1 { '▀' } else { '▄' }));
        }
        plot(5, 2, &cells)
    }),
    ("pipe", 2, |i| {
        // A junction turning, its arms reaching out from the middle: up, left, right, down.
        let arms = [[1, 1, 0, 1], [1, 1, 0, 0], [1, 1, 1, 0], [1, 0, 1, 0], [1, 0, 1, 1], [0, 0, 1, 1], [0, 1, 1, 1], [0, 1, 0, 1]][i.rem_euclid(8) as usize];
        let [u, l, r, d] = arms;
        let hub: Vec<char> = "┼├┤│┴└┘╵┬┌┐╷─╶╴ ".chars().collect();
        let mut cells = vec![(2, 1, hub[((1 - u) << 3) | ((1 - d) << 2) | ((1 - r) << 1) | (1 - l)])];
        if u == 1 {
            cells.push((2, 0, '│'));
        }
        if d == 1 {
            cells.push((2, 2, '│'));
        }
        if l == 1 {
            cells.extend([(0, 1, '─'), (1, 1, '─')]);
        }
        if r == 1 {
            cells.extend([(3, 1, '─'), (4, 1, '─')]);
        }
        plot(5, 3, &cells)
    }),
    ("arc", 1, |i| {
        // An arc chasing round a rounded box, stretching and shrinking as it goes.
        let ring = [
            (0, 0, '╭'),
            (1, 0, '─'),
            (2, 0, '─'),
            (3, 0, '─'),
            (4, 0, '╮'),
            (4, 1, '│'),
            (4, 2, '╯'),
            (3, 2, '─'),
            (2, 2, '─'),
            (1, 2, '─'),
            (0, 2, '╰'),
            (0, 1, '│'),
        ];
        let n = 3 + (ping(8, i) as f64 * 4.0 / 7.0).round() as i64;
        plot(5, 3, &(0..n).map(|k| ring[(i + 12 - k).rem_euclid(12) as usize]).collect::<Vec<_>>())
    }),
    ("bounce", 1, |i| vec![format!("[{}o{}]", " ".repeat(ping(6, i) as usize), " ".repeat(5 - ping(6, i) as usize))]),
    ("clock", 3, |i| {
        // One hand from the middle to the rim, a quarter turn a step.
        let mut g: Vec<Vec<char>> = ["╭─────╮", "│     │", "╰─────╯"].iter().map(|r| r.chars().collect()).collect();
        let hands: [&[(usize, usize, char)]; 4] = [
            &[(3, 0, '┬'), (3, 1, '╵')],
            &[(3, 1, '╶'), (4, 1, '─'), (5, 1, '─'), (6, 1, '┤')],
            &[(3, 1, '╷'), (3, 2, '┴')],
            &[(0, 1, '├'), (1, 1, '─'), (2, 1, '─'), (3, 1, '╴')],
        ];
        for &(x, y, ch) in hands[i.rem_euclid(4) as usize] {
            g[y][x] = ch;
        }
        g.into_iter().map(|r| r.into_iter().collect()).collect()
    }),
    ("bar", 1, |i| {
        // A bar three rows tall filling and draining in eighths.
        let level = (ping(12, i + 2) as f64 * 24.0 / 11.0).round() as i64;
        let e: Vec<char> = " ▁▂▃▄▅▆▇█".chars().collect();
        [2, 1, 0].iter().map(|k| e[(level - 8 * k).clamp(0, 8) as usize].to_string().repeat(3)).collect()
    }),
    ("loop", 1, |i| {
        // A snake of four quadrants running round a three-cell strip.
        let path = [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (5, 1), (4, 1), (3, 1), (2, 1), (1, 1), (0, 1)];
        let on: Vec<(i64, i64)> = (0..4).map(|k| path[(i + k).rem_euclid(12) as usize]).collect();
        let at = |x: i64, y: i64| on.contains(&(x, y));
        vec![[0, 2, 4].iter().map(|&x| quad(at(x, 0), at(x + 1, 0), at(x, 1), at(x + 1, 1))).collect()]
    }),
    ("scan", 1, |i| {
        let (head, dir) = (ping(7, i), if i.rem_euclid(12) < 6 { -1 } else { 1 });
        vec![(0..7)
            .map(|x| {
                if x == head {
                    '█'
                } else if x == head + dir {
                    '▓'
                } else if x == head + 2 * dir {
                    '░'
                } else {
                    '·'
                }
            })
            .collect()]
    }),
    ("wave", 1, |i| {
        let e: Vec<char> = "▁▂▃▄▅▆▇█".chars().collect();
        vec![(0..7).map(|x| e[(3.5 + 3.5 * ((x - i) as f64 * PI / 4.0).sin()).round() as usize]).collect()]
    }),
    ("slide", 1, |i| vec![format!("[{}]", (0..8).map(|x| if (0..3).contains(&(x - (i.rem_euclid(10) - 2))) { '=' } else { ' ' }).collect::<String>())]),
    ("orbit", 1, |i| {
        let ring = [(1, 0), (2, 0), (3, 0), (4, 1), (3, 2), (2, 2), (1, 2), (0, 1)];
        let k0 = i.rem_euclid(8) as usize;
        plot(
            5,
            3,
            &ring
                .iter()
                .enumerate()
                .map(|(k, &(x, y))| {
                    (
                        x,
                        y,
                        if k == k0 {
                            'o'
                        } else if k == (k0 + 7) % 8 {
                            '°'
                        } else {
                            '·'
                        },
                    )
                })
                .collect::<Vec<_>>(),
        )
    }),
    // Twelve that need more than ASCII and box drawing.
    ("braille", 1, |i| cycle("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏", i)),
    ("snake", 1, |i| cycle("⣾⣽⣻⢿⡿⣟⣯⣷", i)),
    ("rise", 1, |i| cycle("⡀⡄⡆⡇⣇⣧⣷⣿⣶⣤⣀ ", i)),
    ("hop", 1, |i| {
        // A dot bouncing along a row of braille, a dot column at a time.
        let x = ping(10, i) as usize;
        let lift = [4u8, 3, 2, 1, 2, 3, 4, 3, 2, 3][x];
        let dot = if x.is_multiple_of(2) { [7, 3, 2, 1][(4 - lift) as usize] } else { [8, 6, 5, 4][(4 - lift) as usize] };
        vec![(0..5).map(|c| if c == x / 2 { braille(&[dot]) } else { '⠀' }).collect::<String>().replace('⠀', " ")]
    }),
    ("sine", 1, |i| {
        // A sine wave running along five braille cells, a dot a column.
        let row = |x: i64| (1.5 + 1.5 * ((x - i) as f64 * PI / 5.0).sin()).round() as usize;
        vec![(0..5)
            .map(|c| {
                let (l, r) = (row(2 * c), row(2 * c + 1));
                braille(&[[1, 2, 3, 7][3 - l], [4, 5, 6, 8][3 - r]])
            })
            .collect()]
    }),
    ("fill", 1, |i| {
        // A cell's eight dots lit one by one round the cell, then put out.
        const ORDER: [u8; 8] = [1, 4, 5, 6, 8, 7, 3, 2];
        let k = ping(9, i) as usize;
        vec![if k == 0 { " ".into() } else { braille(&ORDER[..k]).to_string() }]
    }),
    ("arcs", 1, |i| cycle("◜◝◞◟", i)),
    ("quarters", 2, |i| cycle("◴◷◶◵", i)),
    ("halves", 2, |i| cycle("◐◓◑◒", i)),
    ("corners", 2, |i| cycle("◢◣◤◥", i)),
    ("arrow", 2, |i| cycle("↑↗→↘↓↙←↖", i)),
    ("grow", 1, |i| cycle("▖▌▛█▜▐▗ ", i)),
];

/// The spinners on show, and how they are laid out.
pub struct Spinners {
    chosen: Vec<&'static Spinner>,
    labels: bool,
    speed: f64,
}

/// Spinners; reads `names` (empty for all), `labels` and `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let names = options.list_str("names", &[]);
    let chosen: Vec<&Spinner> =
        if names.is_empty() { SPINNERS.iter().collect() } else { names.iter().filter_map(|n| SPINNERS.iter().find(|s| s.0 == n)).collect() };
    Box::new(Spinners { chosen, labels: options.bool("labels", true), speed: options.f64("speed", 1.0) })
}

impl Piece for Spinners {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        // Slots eleven columns wide, six to a row; a block is the art's three
        // rows, a blank row and the label.
        let (slot, per, block) = (11i64, 6usize, 5i64);
        grid.clear();
        for (n, &&(name, frames, draw)) in self.chosen.iter().enumerate() {
            let (x0, y0) = ((n % per) as i64 * slot, 1 + (n / per) as i64 * (block + 1));
            let art = draw((t * META.fps as f64 * self.speed / f64::from(frames) + 1e-3).floor() as i64);
            let top = y0 + (3 - art.len() as i64) / 2;
            let color = (n % 6) as u8;
            for (j, line) in art.iter().enumerate() {
                let left = x0 + (slot - line.chars().count() as i64) / 2;
                grid.text_at(left, top + j as i64, line, color);
            }
            if self.labels {
                grid.text_at(x0 + (slot - name.len() as i64) / 2, y0 + 4, name, LABEL);
            }
        }
    }
}
