//! elm: the Elm logo, a tangram of seven pieces, with a sheen crossing it every
//! few seconds. After ascii.rest's elm.ts (MIT, @bas3line).
//!
//! Drawn from devicon's elm-original.svg (MIT) by scripts/logo.py, into
//! art/elm.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it, with the gaps between the
//! pieces widened so they read in one ink. On a canvas it takes the logo's
//! colours, lifted on a dark page where they would sink into it; in a `<pre>`
//! it is one ink, from the same drawing with its white left bare. The logo is a
//! trademark of its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The elm logo's meta.
pub static META: Meta = Meta {
    name: "elm",
    slug: "elm",
    category: Category::Logos,
    note: "seven tangram pieces in a square, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 4 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#efa500", "#8dd737", "#60b5cc", "#34495e", "#f5c559", "#b5e57d", "#98cfde", "#7b8996", "#fae4b3", "#ddf3c3", "#cfe9f0", "#c2c8cf", "#efa500",
        "#8dd737", "#60b5cc", "#45617d", "#f5c559", "#b5e57d", "#98cfde", "#8698ab", "#fae4b3", "#ddf3c3", "#cfe9f0", "#c7d0d8",
    ]),
    ..Meta::DEFAULT
};

/// The elm logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/elm.txt"), options)
}
