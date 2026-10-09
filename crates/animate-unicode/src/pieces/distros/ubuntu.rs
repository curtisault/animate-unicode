//! ubuntu: the Ubuntu logo, the circle of friends, with a scan line running
//! down it every few seconds that scrambles what it passes. After ascii.rest's
//! ubuntu.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' ubuntu.svg (CC0) by scripts/logo.py, into
//! art/ubuntu.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The ubuntu logo's meta.
pub static META: Meta = Meta {
    name: "ubuntu",
    slug: "ubuntu",
    category: Category::Distros,
    note: "the circle of friends, in orange, scanned now and then",
    cols: 58,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#e95420", "#f1906e", "#f8ccbc", "#e95420", "#f1906e", "#f8ccbc"]),
    ..Meta::DEFAULT
};

/// The ubuntu logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/ubuntu.txt"), Loop::Scan, options)
}
