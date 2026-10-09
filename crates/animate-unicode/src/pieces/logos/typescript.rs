//! typescript: the TypeScript logo, a blue square lettered TS, with a glint
//! crossing it every few seconds. After ascii.rest's typescript.ts (MIT,
//! @bas3line).
//!
//! Drawn from devicon's typescript-original.svg (MIT) by scripts/logo.py, into
//! art/typescript.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from typescript-plain.svg. The logo is a
//! trademark of its owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The typescript logo's meta.
pub static META: Meta = Meta {
    name: "typescript",
    slug: "typescript",
    category: Category::Logos,
    note: "ts on a blue square, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 2 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&["#007acc", "#feffff", "#59a9de", "#feffff", "#b3d7f0", "#ffffff", "#007acc", "#feffff", "#59a9de", "#feffff", "#b3d7f0", "#ffffff"]),
    ..Meta::DEFAULT
};

/// The typescript logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/typescript.txt"), Loop::Glint, options)
}
