//! scala: the Scala logo, a red spiral stair, with a glint crossing it every
//! few seconds. After ascii.rest's scala.ts (MIT, @bas3line).
//!
//! Drawn from devicon's scala-original.svg (MIT) by scripts/logo.py, into
//! art/scala.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from scala-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The scala logo's meta.
pub static META: Meta = Meta {
    name: "scala",
    slug: "scala",
    category: Category::Logos,
    note: "the red spiral stair, glinting now and then",
    cols: 37,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 2 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&["#3a0d09", "#e33524", "#7f625f", "#ed7c71", "#c4b6b5", "#f7c2bd", "#b6291c", "#e33524", "#d0746b", "#ed7c71", "#e9bfbb", "#f7c2bd"]),
    ..Meta::DEFAULT
};

/// The scala logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/scala.txt"), Loop::Glint, options)
}
