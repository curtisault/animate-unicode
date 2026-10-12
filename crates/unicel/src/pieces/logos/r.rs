//! r: the R logo, a blue R over a grey ring, with a sheen crossing it every few
//! seconds. After ascii.rest's r.ts (MIT, @bas3line).
//!
//! Drawn from devicon's r-original.svg (MIT) by scripts/logo.py, into
//! art/r.txt: each cell holds the octant, or the wedge where an edge cuts it at
//! a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from r-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The r logo's meta.
pub static META: Meta = Meta {
    name: "r",
    slug: "r",
    category: Category::Logos,
    note: "a blue r over a grey ring, glinting now and then",
    cols: 69,
    rows: 27,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 3 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#b0b2b6", "#2066b9", "#97989e", "#cccdd0", "#6e9cd2", "#bbbcc0", "#e7e8e9", "#bcd1ea", "#e0e0e2", "#b0b2b6", "#2066b9", "#97989e", "#cccdd0",
        "#6e9cd2", "#bbbcc0", "#e7e8e9", "#bcd1ea", "#e0e0e2",
    ]),
    ..Meta::DEFAULT
};

/// The r logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/r.txt"), options)
}
