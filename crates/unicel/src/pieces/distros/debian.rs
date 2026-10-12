//! debian: the Debian logo, the swirl, with a sheen crossing it every few
//! seconds. After ascii.rest's debian.ts (MIT, @bas3line).
//!
//! Drawn from devicon's debian-original.svg (MIT) by scripts/logo.py, into
//! art/debian.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from devicon's debian-plain.svg (MIT). The logo is
//! a trademark of its owner, shown here to name the distribution.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The debian logo's meta.
pub static META: Meta = Meta {
    name: "debian",
    slug: "debian",
    category: Category::Distros,
    note: "the red swirl, glinting now and then",
    cols: 46,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#a80030", "#c65978", "#e5b3c1", "#c20037", "#d7597d", "#edb3c3"]),
    ..Meta::DEFAULT
};

/// The debian logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/debian.txt"), options)
}
