//! almalinux: the AlmaLinux logo, five coloured figures in a ring, with a sheen
//! crossing it every few seconds. After ascii.rest's almalinux.ts (MIT,
//! @bas3line).
//!
//! Drawn from devicon's almalinux-original.svg (MIT) by scripts/logo.py, into
//! art/almalinux.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from devicon's almalinux-plain.svg (MIT).
//! The logo is a trademark of its owner, shown here to name the distribution.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The almalinux logo's meta.
pub static META: Meta = Meta {
    name: "almalinux",
    slug: "almalinux",
    category: Category::Distros,
    note: "five coloured figures in a ring, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 5 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#ff4546", "#0068d8", "#85d82e", "#ffc911", "#22c1ff", "#ff8687", "#599de6", "#b0e677", "#ffdc64", "#6fd7ff", "#ffc7c8", "#b3d2f3", "#daf3c0",
        "#ffefb8", "#bdecff", "#ff4546", "#0068d8", "#85d82e", "#ffc911", "#22c1ff", "#ff8687", "#599de6", "#b0e677", "#ffdc64", "#6fd7ff", "#ffc7c8",
        "#b3d2f3", "#daf3c0", "#ffefb8", "#bdecff",
    ]),
    ..Meta::DEFAULT
};

/// The almalinux logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/almalinux.txt"), options)
}
