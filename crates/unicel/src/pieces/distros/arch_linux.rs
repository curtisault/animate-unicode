//! arch linux: the Arch Linux logo, the blue arch, with a sheen crossing it
//! every few seconds. After ascii.rest's arch-linux.ts (MIT, @bas3line).
//!
//! Drawn from devicon's archlinux-original.svg (MIT) by scripts/logo.py, into
//! art/arch-linux.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from the same drawing with its white left
//! bare. The logo is a trademark of its owner, shown here to name the
//! distribution.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The arch linux logo's meta.
pub static META: Meta = Meta {
    name: "arch linux",
    slug: "arch-linux",
    category: Category::Distros,
    note: "the blue arch, glinting now and then",
    cols: 59,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#1894d4", "#69b9e3", "#badff2", "#1894d4", "#69b9e3", "#badff2"]),
    ..Meta::DEFAULT
};

/// The arch linux logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/arch-linux.txt"), options)
}
