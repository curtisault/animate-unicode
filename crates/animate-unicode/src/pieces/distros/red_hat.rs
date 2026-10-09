//! red hat: the Red Hat logo, the man in the red hat, with a scan line running
//! down it every few seconds that scrambles what it passes. After ascii.rest's
//! red-hat.ts (MIT, @bas3line).
//!
//! Drawn from devicon's redhat-original.svg (MIT) by scripts/logo.py, into
//! art/red-hat.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from devicon's redhat-plain.svg (MIT). The logo is
//! a trademark of its owner, shown here to name the distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The red hat logo's meta.
pub static META: Meta = Meta {
    name: "red hat",
    slug: "red-hat",
    category: Category::Distros,
    note: "the man in the red hat, scanned now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's 3 colours for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&[
        "#000000", "#ffffff", "#e93442", "#595959", "#ffffff", "#f17b84", "#b3b3b3", "#ffffff", "#f8c2c6", "#545454", "#ffffff", "#e93442", "#909090",
        "#ffffff", "#f17b84", "#cccccc", "#ffffff", "#f8c2c6",
    ]),
    ..Meta::DEFAULT
};

/// The red hat logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/red-hat.txt"), Loop::Scan, options)
}
