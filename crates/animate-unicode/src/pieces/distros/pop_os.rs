//! pop!_os: the Pop!_OS logo, a p and a ! in a circle, with a scan line running
//! down it every few seconds that scrambles what it passes. After ascii.rest's
//! pop-os.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' popos.svg (CC0) by scripts/logo.py, into
//! art/pop-os.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The pop!_os logo's meta.
pub static META: Meta = Meta {
    name: "pop!_os",
    slug: "pop-os",
    category: Category::Distros,
    note: "the p! in a teal circle, scanned now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#48b9c7", "#88d2db", "#c8eaee", "#48b9c7", "#88d2db", "#c8eaee"]),
    ..Meta::DEFAULT
};

/// The pop!_os logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/pop-os.txt"), Loop::Scan, options)
}
