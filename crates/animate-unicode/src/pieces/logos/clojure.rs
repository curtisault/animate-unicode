//! clojure: the Clojure logo, a lambda in a green and blue circle, with a sheen
//! crossing it every few seconds. After ascii.rest's clojure.ts (MIT,
//! @bas3line).
//!
//! Drawn from devicon's clojure-original.svg (MIT) by scripts/logo.py, into
//! art/clojure.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The clojure logo's meta.
pub static META: Meta = Meta {
    name: "clojure",
    slug: "clojure",
    category: Category::Logos,
    note: "a lambda in a green and blue circle, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 5 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#ffffff", "#5881d8", "#91dc48", "#90b4fe", "#63b132", "#ffffff", "#92ade6", "#b8e888", "#b7cefe", "#9acc7a", "#ffffff", "#cdd9f3", "#def5c8",
        "#dee9ff", "#d0e8c2", "#ffffff", "#5881d8", "#91dc48", "#90b4fe", "#63b132", "#ffffff", "#92ade6", "#b8e888", "#b7cefe", "#9acc7a", "#ffffff",
        "#cdd9f3", "#def5c8", "#dee9ff", "#d0e8c2",
    ]),
    ..Meta::DEFAULT
};

/// The clojure logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/clojure.txt"), options)
}
