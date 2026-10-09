//! fedora: the Fedora logo, an f in a blue speech bubble, with a scan line
//! running down it every few seconds that scrambles what it passes. After
//! ascii.rest's fedora.ts (MIT, @bas3line).
//!
//! Drawn from devicon's fedora-original.svg (MIT) by scripts/logo.py, into
//! art/fedora.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from devicon's fedora-plain.svg (MIT). The logo is
//! a trademark of its owner, shown here to name the distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The fedora logo's meta.
pub static META: Meta = Meta {
    name: "fedora",
    slug: "fedora",
    category: Category::Distros,
    note: "the f in its blue bubble, scanned now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's 3 colours for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&[
        "#294172", "#ffffff", "#3c6eb4", "#7484a3", "#ffffff", "#80a1ce", "#bfc6d5", "#ffffff", "#c5d4e9", "#3b5da3", "#ffffff", "#3c6eb4", "#8096c3",
        "#ffffff", "#80a1ce", "#c4cee3", "#ffffff", "#c5d4e9",
    ]),
    ..Meta::DEFAULT
};

/// The fedora logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/fedora.txt"), Loop::Scan, options)
}
