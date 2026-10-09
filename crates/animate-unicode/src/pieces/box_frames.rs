//! box frames: a sheet of twelve frame styles, from box drawing lines to half
//! blocks, a title in the edge, a drop shadow and bare corners.
//!
//! A straight port of ascii.rest's box-frames.ts (MIT, @bas3line); the test
//! `box_frames_matches_the_typescript_original` in tests/contract.rs holds it
//! to the TS sheet cell for cell. It proves box-drawing joins in each renderer.

use crate::{Category, Env, Grid, Meta, Options, Piece};

/// The box frames' meta.
pub static META: Meta = Meta {
    name: "box frames",
    slug: "box-frames",
    category: Category::Ui,
    note: "twelve frame styles to copy, each one labelled",
    cols: 78,
    rows: 20,
    fps: 0,
    ..Meta::DEFAULT
};

/// One frame, borders included.
const W: i64 = 16;
const H: i64 = 5;
const GAP_X: i64 = 4;
const GAP_Y: i64 = 1;

/// Corners top left, top right, bottom left, bottom right, then the top and
/// bottom edge, then the sides.
fn lines(name: &str) -> [char; 8] {
    let s = match name {
        "double" => "╔╗╚╝══║║",
        "rounded" => "╭╮╰╯──││",
        "heavy" => "┏┓┗┛━━┃┃",
        "dashed" => "┌┐└┘┄┄┆┆",
        "ascii" => "++++--||",
        "mixed" => "╒╕╘╛══││",
        "block" => "▛▜▙▟▀▄▌▐",
        _ => "┌┐└┘──││", // single, titled, shadow
    };
    let mut g = [' '; 8];
    for (slot, ch) in g.iter_mut().zip(s.chars()) {
        *slot = ch;
    }
    g
}

const SHEET: [&str; 12] = ["single", "double", "rounded", "heavy", "dashed", "ascii", "mixed", "titled", "shade", "block", "shadow", "corners"];

/// The sheet. It never changes, so it is drawn once and copied.
pub struct BoxFrames {
    sheet: Grid,
}

/// The box frames; they take no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut g = Grid::new(META.cols, META.rows);
    let left = (META.cols as i64 - 4 * W - 3 * GAP_X) / 2;
    for (i, name) in SHEET.iter().enumerate() {
        let i = i as i64;
        let (y, x) = (1 + (i / 4) * (H + GAP_Y), left + (i % 4) * (W + GAP_X));
        match *name {
            "corners" => corners(&mut g, y, x),
            "shade" => shade(&mut g, y, x),
            _ => frame(&mut g, y, x, lines(name)),
        }
        if *name == "shadow" {
            shadow(&mut g, y, x);
        }
        // The titled frame carries its name in the top edge; the rest inside.
        if *name == "titled" {
            g.text_at(x + 2, y, &format!(" {name} "), 0);
        } else {
            g.text_at(x + (W - name.len() as i64) / 2, y + 2, name, 0);
        }
    }
    Box::new(BoxFrames { sheet: g })
}

fn frame(g: &mut Grid, y: i64, x: i64, c: [char; 8]) {
    for col in x + 1..x + W - 1 {
        g.put(col, y, c[4]);
        g.put(col, y + H - 1, c[5]);
    }
    for row in y + 1..y + H - 1 {
        g.put(x, row, c[6]);
        g.put(x + W - 1, row, c[7]);
    }
    g.put(x, y, c[0]);
    g.put(x + W - 1, y, c[1]);
    g.put(x, y + H - 1, c[2]);
    g.put(x + W - 1, y + H - 1, c[3]);
}

/// A border of light shade, two columns wide at the sides so it is as thick
/// as the one-row top and bottom, a cell being twice as tall as it is wide.
fn shade(g: &mut Grid, y: i64, x: i64) {
    for col in x..x + W {
        g.put(col, y, '░');
        g.put(col, y + H - 1, '░');
    }
    for row in y + 1..y + H - 1 {
        for col in [x, x + 1, x + W - 2, x + W - 1] {
            g.put(col, row, '░');
        }
    }
}

/// Only the corners, each with a short arm along the top or bottom edge.
fn corners(g: &mut Grid, y: i64, x: i64) {
    let (r, b) = (x + W - 1, y + H - 1);
    g.text_at(x, y, "┌──", 0);
    g.text_at(r - 2, y, "──┐", 0);
    g.text_at(x, b, "└──", 0);
    g.text_at(r - 2, b, "──┘", 0);
}

/// A shadow one row down and two columns over, so it falls square.
fn shadow(g: &mut Grid, y: i64, x: i64) {
    for row in y + 1..=y + H {
        g.put(x + W, row, '▒');
        g.put(x + W + 1, row, '▒');
    }
    for col in x + 2..x + W {
        g.put(col, y + H, '▒');
    }
}

impl Piece for BoxFrames {
    fn frame(&mut self, _: f64, _: &Env, grid: &mut Grid) {
        grid.cells_mut().copy_from_slice(self.sheet.cells());
        grid.colors_mut().copy_from_slice(self.sheet.colors());
    }
}
