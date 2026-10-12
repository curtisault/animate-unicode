//! centos: the CentOS logo, four coloured squares in a star, with a sheen
//! crossing it every few seconds. After ascii.rest's centos.ts (MIT,
//! @bas3line).
//!
//! Drawn from devicon's centos-original.svg (MIT) by scripts/logo.py, into
//! art/centos.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from devicon's centos-plain.svg (MIT). The logo is
//! a trademark of its owner, shown here to name the distribution.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The centos logo's meta.
pub static META: Meta = Meta {
    name: "centos",
    slug: "centos",
    category: Category::Distros,
    note: "four coloured squares in a star, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 5 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#fefefe", "#262577", "#9ccd2c", "#93237a", "#efa725", "#fefefe", "#7271a7", "#bfdf76", "#b970a9", "#f5c671", "#ffffff", "#bebed6", "#e1f0c0",
        "#dfbdd7", "#fae5be", "#fefefe", "#504ec8", "#9ccd2c", "#a8288b", "#efa725", "#fefefe", "#8d8cdb", "#bfdf76", "#c673b4", "#f5c671", "#ffffff",
        "#cbcaef", "#e1f0c0", "#e5bfdc", "#fae5be",
    ]),
    ..Meta::DEFAULT
};

/// The centos logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/centos.txt"), options)
}
