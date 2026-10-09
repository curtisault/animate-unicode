//! c#: the C# logo, a C and a sharp sign on a hexagon, with a glint crossing it
//! every few seconds. After ascii.rest's csharp.ts (MIT, @bas3line).
//!
//! Drawn from devicon's csharp-original.svg (MIT) by scripts/logo.py, into
//! art/csharp.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from csharp-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The c# logo's meta.
pub static META: Meta = Meta {
    name: "c#",
    slug: "csharp",
    category: Category::Logos,
    note: "c# on a purple hexagon, glinting now and then",
    cols: 49,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 3 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&[
        "#9b4f96", "#ffffff", "#68217a", "#be8dbb", "#ffffff", "#9d6fa9", "#e1cae0", "#ffffff", "#d2bcd7", "#9b4f96", "#ffffff", "#942fae", "#be8dbb",
        "#ffffff", "#b978ca", "#e1cae0", "#ffffff", "#dfc1e7",
    ]),
    ..Meta::DEFAULT
};

/// The c# logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/csharp.txt"), Loop::Glint, options)
}
