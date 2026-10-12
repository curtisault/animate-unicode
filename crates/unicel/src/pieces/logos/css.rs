//! css: the CSS logo, a blue shield with a 3, with a sheen crossing it every
//! few seconds. After ascii.rest's css.ts (MIT, @bas3line).
//!
//! Drawn from devicon's css3-original.svg (MIT) by scripts/logo.py, into
//! art/css.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from css3-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The css logo's meta.
pub static META: Meta = Meta {
    name: "css",
    slug: "css",
    category: Category::Logos,
    note: "the blue shield with a 3, glinting now and then",
    cols: 50,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 3 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#1572b6", "#f6f7f7", "#33a9dc", "#67a3d0", "#f9fafa", "#7ac7e8", "#b9d5e9", "#fcfdfd", "#c2e5f5", "#1572b6", "#f6f7f7", "#33a9dc", "#67a3d0",
        "#f9fafa", "#7ac7e8", "#b9d5e9", "#fcfdfd", "#c2e5f5",
    ]),
    ..Meta::DEFAULT
};

/// The css logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/css.txt"), options)
}
