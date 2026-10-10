//! java: the Java logo, a steaming cup, with a sheen crossing it every few
//! seconds. After ascii.rest's java.ts (MIT, @bas3line).
//!
//! Drawn from devicon's java-original.svg (MIT) by scripts/logo.py, into
//! art/java.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from java-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The java logo's meta.
pub static META: Meta = Meta {
    name: "java",
    slug: "java",
    category: Category::Logos,
    note: "the steaming cup, glinting now and then",
    cols: 42,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 2 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#ef2e2f", "#0077c1", "#f57778", "#59a7d7", "#fac0c1", "#b3d6ec", "#ef2e2f", "#0077c1", "#f57778", "#59a7d7", "#fac0c1", "#b3d6ec"]),
    ..Meta::DEFAULT
};

/// The java logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/java.txt"), options)
}
