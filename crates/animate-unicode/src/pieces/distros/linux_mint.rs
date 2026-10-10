//! linux mint: the Linux Mint logo, lm in a green circle, with a sheen crossing
//! it every few seconds. After ascii.rest's linux-mint.ts (MIT, @bas3line).
//!
//! Drawn from devicon's linuxmint-original.svg (MIT) by scripts/logo.py, into
//! art/linux-mint.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from Simple Icons' linuxmint.svg (CC0). The
//! logo is a trademark of its owner, shown here to name the distribution.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The linux mint logo's meta.
pub static META: Meta = Meta {
    name: "linux mint",
    slug: "linux-mint",
    category: Category::Distros,
    note: "lm in a green circle, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 2 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#fcfcfc", "#69b53f", "#fdfdfd", "#9ecf82", "#fefefe", "#d2e9c5", "#fcfcfc", "#69b53f", "#fdfdfd", "#9ecf82", "#fefefe", "#d2e9c5"]),
    ..Meta::DEFAULT
};

/// The linux mint logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/linux-mint.txt"), options)
}
