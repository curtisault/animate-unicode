//! deepin: the deepin logo, a swirl in a ring, with a scan line running down it
//! every few seconds that scrambles what it passes. After ascii.rest's
//! deepin.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' deepin.svg (CC0) by scripts/logo.py, into
//! art/deepin.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The deepin logo's meta.
pub static META: Meta = Meta {
    name: "deepin",
    slug: "deepin",
    category: Category::Distros,
    note: "the swirl in a blue ring, scanned now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#007cff", "#59aaff", "#b3d8ff", "#007cff", "#59aaff", "#b3d8ff"]),
    ..Meta::DEFAULT
};

/// The deepin logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/deepin.txt"), Loop::Scan, options)
}
