//! endeavouros: the EndeavourOS logo, a sail, with a scan line running down it
//! every few seconds that scrambles what it passes. After ascii.rest's
//! endeavouros.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' endeavouros.svg (CC0) by scripts/logo.py, into
//! art/endeavouros.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from the same drawing with its white left
//! bare. The logo is a trademark of its owner, shown here to name the
//! distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The endeavouros logo's meta.
pub static META: Meta = Meta {
    name: "endeavouros",
    slug: "endeavouros",
    category: Category::Distros,
    note: "the violet sail, scanned now and then",
    cols: 67,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#7f7fff", "#acacff", "#d9d9ff", "#7f7fff", "#acacff", "#d9d9ff"]),
    ..Meta::DEFAULT
};

/// The endeavouros logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/endeavouros.txt"), Loop::Scan, options)
}
