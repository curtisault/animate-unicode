//! opensuse: the openSUSE logo, the green chameleon, with a scan line running
//! down it every few seconds that scrambles what it passes. After ascii.rest's
//! opensuse.ts (MIT, @bas3line).
//!
//! Drawn from devicon's opensuse-original.svg (MIT) by scripts/logo.py, into
//! art/opensuse.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from the same drawing with its white left
//! bare. The logo is a trademark of its owner, shown here to name the
//! distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The opensuse logo's meta.
pub static META: Meta = Meta {
    name: "opensuse",
    slug: "opensuse",
    category: Category::Distros,
    note: "the green chameleon, scanned now and then",
    cols: 69,
    rows: 18,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#76be26", "#a6d572", "#d6ecbe", "#76be26", "#a6d572", "#d6ecbe"]),
    ..Meta::DEFAULT
};

/// The opensuse logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/opensuse.txt"), Loop::Scan, options)
}
