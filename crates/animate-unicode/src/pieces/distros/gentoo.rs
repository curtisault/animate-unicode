//! gentoo: the Gentoo logo, a lavender g like a bent stone, with a scan line
//! running down it every few seconds that scrambles what it passes. After
//! ascii.rest's gentoo.ts (MIT, @bas3line).
//!
//! Drawn from devicon's gentoo-plain.svg (MIT) by scripts/logo.py, into
//! art/gentoo.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The gentoo logo's meta.
pub static META: Meta = Meta {
    name: "gentoo",
    slug: "gentoo",
    category: Category::Distros,
    note: "the lavender g, scanned now and then",
    cols: 54,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#9d95de", "#bfbaea", "#e2dff5", "#9d95de", "#bfbaea", "#e2dff5"]),
    ..Meta::DEFAULT
};

/// The gentoo logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/gentoo.txt"), Loop::Scan, options)
}
