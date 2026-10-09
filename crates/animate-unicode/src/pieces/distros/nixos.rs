//! nixos: the NixOS logo, the snowflake of lambdas, with a scan line running
//! down it every few seconds that scrambles what it passes. After ascii.rest's
//! nixos.ts (MIT, @bas3line).
//!
//! Drawn from devicon's nixos-original.svg (MIT) by scripts/logo.py, into
//! art/nixos.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from devicon's nixos-plain.svg (MIT). The logo is a
//! trademark of its owner, shown here to name the distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The nixos logo's meta.
pub static META: Meta = Meta {
    name: "nixos",
    slug: "nixos",
    category: Category::Distros,
    note: "the snowflake of lambdas, scanned now and then",
    cols: 64,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's 2 colours for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#5277c3", "#81bfea", "#8fa7d8", "#add5f1", "#cbd6ed", "#d9ecf9", "#5277c3", "#81bfea", "#8fa7d8", "#add5f1", "#cbd6ed", "#d9ecf9"]),
    ..Meta::DEFAULT
};

/// The nixos logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/nixos.txt"), Loop::Scan, options)
}
