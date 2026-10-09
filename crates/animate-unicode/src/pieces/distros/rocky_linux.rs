//! rocky linux: the Rocky Linux logo, a mountain cut from a green circle, with
//! a scan line running down it every few seconds that scrambles what it passes.
//! After ascii.rest's rocky-linux.ts (MIT, @bas3line).
//!
//! Drawn from devicon's rockylinux-original.svg (MIT) by scripts/logo.py, into
//! art/rocky-linux.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from the same drawing with its white left
//! bare. The logo is a trademark of its owner, shown here to name the
//! distribution.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The rocky linux logo's meta.
pub static META: Meta = Meta {
    name: "rocky linux",
    slug: "rocky-linux",
    category: Category::Distros,
    note: "the mountain in a green circle, scanned now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"scan": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the scan.
    palette: Some(&["#10bd85", "#64d4b0", "#b7ebda", "#10bd85", "#64d4b0", "#b7ebda"]),
    ..Meta::DEFAULT
};

/// The rocky linux logo; `scan` is the seconds between scans, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/rocky-linux.txt"), Loop::Scan, options)
}
