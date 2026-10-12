//! file tree: a project listed the way `tree -F` prints it, opened one
//! folder at a time by a cursor walking down the list, then closed from the
//! bottom up. After ascii.rest's file-tree.ts (MIT, @bas3line).
//!
//! In colour, the way a file manager paints a listing: folders blue, files
//! by their kind (code yellow, docs white, styles pink, data green, the rest
//! grey), the branches dim; a closed folder carries a `▸` where ascii.rest
//! had a `+`, and the cursor's line is lit amber.

use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

const ENTRIES: &str = "docs/\n  guide.md\n  notes.md\npublic/\n  favicon.ico\n  robots.txt\nsrc/\n  components/\n    button.ts\n    card.ts\n    menu.ts\n  lib/\n    fetch.ts\n    format.ts\n  index.ts\n  styles.css\ntests/\n  format.test.ts\npackage.json\nREADME.md";

/// The file tree's meta.
pub static META: Meta = Meta {
    name: "file tree",
    slug: "file-tree",
    category: Category::Ui,
    note: "a tree listing opened folder by folder, coloured by file kind",
    cols: 36,
    rows: 23,
    fps: 15,
    charset: Charset::Extended,
    options: Some(
        r#"{"root": ".", "entries": "docs/\n  guide.md\n  notes.md\npublic/\n  favicon.ico\n  robots.txt\nsrc/\n  components/\n    button.ts\n    card.ts\n    menu.ts\n  lib/\n    fetch.ts\n    format.ts\n  index.ts\n  styles.css\ntests/\n  format.test.ts\npackage.json\nREADME.md"}"#,
    ),
    // Branches, folders, code, docs, styles, data, other files, the cursor.
    palette: Some(&["#4a505c", "#6aa8f0", "#f0d060", "#e8eaf0", "#f08ac8", "#7ad890", "#9aa0aa", "#ffb44a"]),
    ground: Some("#0e0f13"),
    ..Meta::DEFAULT
};

const BRANCH: u8 = 0;
const FOLDER: u8 = 1;
const CURSOR: u8 = 7;
/// Seconds per line the cursor steps; on a folder before it opens or
/// closes; per line appearing or going; all open; all closed.
const MOVE: f64 = 0.13;
const PAUSE: f64 = 0.3;
const REVEAL: f64 = 0.07;
const HOLD: f64 = 1.8;
const REST: f64 = 0.7;

/// An entry: its name, whether it is a folder, and what it holds.
struct Entry {
    name: String,
    dir: bool,
    kids: Vec<usize>,
}

/// The colour of a file by its kind.
fn kind(name: &str) -> u8 {
    match name.rsplit('.').next() {
        Some("ts" | "js" | "rs" | "py") => 2,
        Some("md" | "txt") => 3,
        Some("css" | "scss") => 4,
        Some("json" | "toml" | "yaml") => 5,
        _ => 6,
    }
}

/// The tree being walked: entries (index 0 is the root), how many of each
/// folder's entries are listed, and the cursor.
struct Walk {
    nodes: Vec<Entry>,
    shown: Vec<usize>,
    cursor: usize,
}

impl Walk {
    /// The lines on screen: the folders' entries in order, cut short where a
    /// folder is closed or still opening; each line as glyphs and colours.
    fn listing(&self) -> Vec<(usize, Vec<(char, u8)>)> {
        let mut out = vec![];
        self.walk(0, String::new(), &mut out);
        out
    }

    fn walk(&self, node: usize, prefix: String, out: &mut Vec<(usize, Vec<(char, u8)>)>) {
        let kids = &self.nodes[node].kids;
        for (i, &kid) in kids.iter().take(self.shown[node]).enumerate() {
            let last = i == kids.len() - 1;
            let e = &self.nodes[kid];
            let here = kid == self.cursor;
            let mut line: Vec<(char, u8)> = prefix.chars().map(|c| (c, BRANCH)).collect();
            line.extend(if last { "└─" } else { "├─" }.chars().map(|c| (c, BRANCH)));
            line.extend(if here { "> " } else { "─ " }.chars().map(|c| (c, if here { CURSOR } else { BRANCH })));
            let color = if here {
                CURSOR
            } else if e.dir {
                FOLDER
            } else {
                kind(&e.name)
            };
            line.extend(e.name.chars().map(|c| (c, color)));
            if e.dir && !e.kids.is_empty() && self.shown[kid] == 0 {
                line.extend([(' ', BRANCH), ('▸', color)]);
            }
            out.push((kid, line));
            if e.dir {
                self.walk(kid, format!("{prefix}{}", if last { "    " } else { "│   " }), out);
            }
        }
    }
}

/// Pictures in the loop, each with the moment it shows.
type Snaps = Vec<(f64, Grid)>;
/// Takes a picture of the walk and holds it for a while.
type Snap<'a> = dyn FnMut(&Walk, f64, &mut Snaps) + 'a;

/// The file tree: every picture in the loop and when it shows, the loop's
/// length, and where play begins.
pub struct FileTree {
    snaps: Vec<(f64, Grid)>,
    total: f64,
    start: f64,
}

/// A file tree; reads `root` (the first line) and `entries` (one a line,
/// two spaces of indent a level, folders ending in "/").
pub fn make(options: &Options) -> Box<dyn Piece> {
    let label = options.string("root", ".");
    let text = options.string("entries", ENTRIES);
    let mut nodes = vec![Entry { name: String::new(), dir: true, kids: vec![] }];
    let mut stack = vec![0usize];
    for raw in text.split('\n') {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        let depth = (raw.len() - raw.trim_start().len()) / 2;
        let id = nodes.len();
        nodes.push(Entry { name: name.to_string(), dir: name.ends_with('/'), kids: vec![] });
        stack.truncate(depth + 1);
        let parent = *stack.last().unwrap();
        nodes[parent].kids.push(id);
        if nodes[id].dir {
            stack.push(id);
        }
    }
    // Counted over the whole tree, as tree itself does.
    let (nd, nf) = nodes.iter().skip(1).fold((0, 0), |(d, f), e| if e.dir { (d + 1, f) } else { (d, f + 1) });
    let summary = format!("{nd} director{}, {nf} file{}", if nd == 1 { "y" } else { "ies" }, if nf == 1 { "" } else { "s" });
    // Folders that hold something, in listing order.
    let mut dirs = vec![];
    fn collect(nodes: &[Entry], n: usize, dirs: &mut Vec<usize>) {
        for &k in &nodes[n].kids {
            if nodes[k].dir && !nodes[k].kids.is_empty() {
                dirs.push(k);
            }
            collect(nodes, k, dirs);
        }
    }
    collect(&nodes, 0, &mut dirs);
    let n = nodes.len();
    let first = dirs.first().copied().unwrap_or_else(|| nodes[0].kids.first().copied().unwrap_or(0));
    let mut w = Walk { nodes, shown: vec![0; n], cursor: first };
    w.shown[0] = w.nodes[0].kids.len();
    // Centre the listing as it stands fully open, with room for the marker.
    for &d in &dirs {
        w.shown[d] = w.nodes[d].kids.len();
    }
    let width = w.listing().iter().map(|l| l.1.len() + 2).chain([label.chars().count(), summary.len()]).max().unwrap_or(0);
    for &d in &dirs {
        w.shown[d] = 0;
    }
    let pad = ((META.cols as i64 - width as i64) / 2).max(0);
    let render = |w: &Walk| {
        let mut g = Grid::new(META.cols, META.rows);
        g.text_at(pad, 0, &label, FOLDER);
        for (r, (_, line)) in w.listing().iter().take(META.rows - 2).enumerate() {
            for (k, &(ch, c)) in line.iter().enumerate() {
                g.set(pad + k as i64, r as i64 + 1, ch, c);
            }
        }
        g.text_at(pad, META.rows as i64 - 1, &summary, BRANCH);
        g
    };
    // The whole loop as a list of pictures and how long each stays up.
    let (mut snaps, mut total, mut start) = (vec![], 0.0, 0.0);
    let mut snap = |w: &Walk, dur: f64, snaps: &mut Snaps| {
        snaps.push((total, render(w)));
        total += dur;
    };
    let walk_to = |w: &mut Walk, node: usize, snaps: &mut Snaps, snap: &mut Snap| loop {
        let vis = w.listing();
        let at = |n: usize| vis.iter().position(|v| v.0 == n).unwrap_or(0);
        if w.cursor == node {
            break;
        }
        let dir: i64 = if at(node) > at(w.cursor) { 1 } else { -1 };
        w.cursor = vis[(at(w.cursor) as i64 + dir) as usize].0;
        snap(w, MOVE, snaps);
    };
    snap(&w, REST, &mut snaps);
    for (i, &node) in dirs.iter().enumerate() {
        walk_to(&mut w, node, &mut snaps, &mut snap);
        snap(&w, PAUSE, &mut snaps);
        for k in 1..=w.nodes[node].kids.len() {
            w.shown[node] = k;
            snap(&w, REVEAL, &mut snaps);
        }
        if i + 1 == (dirs.len() as f64 * 0.6).ceil() as usize {
            start = snaps.last().map_or(0.0, |s| s.0 + REVEAL);
        }
    }
    snap(&w, HOLD, &mut snaps);
    for &node in dirs.iter().rev() {
        walk_to(&mut w, node, &mut snaps, &mut snap);
        snap(&w, PAUSE, &mut snaps);
        for k in (0..w.nodes[node].kids.len()).rev() {
            w.shown[node] = k;
            snap(&w, REVEAL, &mut snaps);
        }
    }
    let total = snaps.last().map_or(1.0, |s| s.0) + REVEAL + REST; // the last picture is the first one, so the loop closes on it
    Box::new(FileTree { snaps, total, start })
}

impl Piece for FileTree {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let p = (t + self.start).rem_euclid(self.total);
        let i = self.snaps.iter().rposition(|s| s.0 <= p).unwrap_or(0);
        grid.cells_mut().copy_from_slice(self.snaps[i].1.cells());
        grid.colors_mut().copy_from_slice(self.snaps[i].1.colors());
    }
}
