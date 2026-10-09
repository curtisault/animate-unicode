//! haskell: the Haskell logo, an angle, a lambda and an equals sign, with a
//! glint crossing it every few seconds. After ascii.rest's haskell.ts (MIT,
//! @bas3line).
//!
//! Drawn from devicon's haskell-original.svg (MIT) by scripts/logo.py, into
//! art/haskell.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from haskell-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The haskell logo's meta.
pub static META: Meta = Meta {
    name: "haskell",
    slug: "haskell",
    category: Category::Logos,
    note: "an angle, a lambda and an equals sign, glinting now and then",
    cols: 69,
    rows: 25,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 3 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&[
        "#463b63", "#5e5187", "#904f8c", "#87809a", "#968eb1", "#b78db4", "#c8c4d0", "#cfcbdb", "#decadd", "#665690", "#655791", "#904f8c", "#9c91b7",
        "#9b92b8", "#b78db4", "#d1ccde", "#d1cdde", "#decadd",
    ]),
    ..Meta::DEFAULT
};

/// The haskell logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/haskell.txt"), Loop::Glint, options)
}
