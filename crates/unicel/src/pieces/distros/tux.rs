//! tux: the Linux logo, Tux, the penguin, with a sheen crossing it every few
//! seconds. After ascii.rest's tux.ts (MIT, @bas3line).
//!
//! Drawn from devicon's linux-original.svg (MIT) by scripts/logo.py, into
//! art/tux.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from devicon's linux-plain.svg (MIT). The logo is a
//! trademark of its owner, shown here to name the distribution.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The tux logo's meta.
pub static META: Meta = Meta {
    name: "tux",
    slug: "tux",
    category: Category::Distros,
    note: "tux the penguin, glinting now and then",
    cols: 49,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 4 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#030202", "#fcfcfc", "#a58137", "#fdcc1b", "#5b5b5b", "#fdfdfd", "#c5ad7d", "#fede6b", "#b3b3b3", "#fefefe", "#e4d9c3", "#fef0bb", "#545454",
        "#fcfcfc", "#a58137", "#fdcc1b", "#909090", "#fdfdfd", "#c5ad7d", "#fede6b", "#cccccc", "#fefefe", "#e4d9c3", "#fef0bb",
    ]),
    ..Meta::DEFAULT
};

/// The tux logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/tux.txt"), options)
}
