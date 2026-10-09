//! alpine linux: the Alpine Linux logo, a mountain in a hexagon, with a scan
//! line running down it every few seconds that scrambles what it passes. After
//! ascii.rest's alpine-linux.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' alpinelinux.svg (CC0) by scripts/logo.py, into
//! art/alpine-linux.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from the same drawing with its white left
//! bare. The logo is a trademark of its owner, shown here to name the
//! distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The alpine linux logo's meta.
pub static META: Meta = Meta {
    name: "alpine linux",
    slug: "alpine-linux",
    category: Category::Distros,
    note: "the mountain in a hexagon, scanned now and then",
    cols: 64,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#0d597f", "#6293ac", "#b6cdd9", "#0f6692", "#639cb8", "#b7d1de"]),
    ..Meta::DEFAULT
};

/// The alpine linux logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/alpine-linux.txt"), Loop::Scan, options)
}
