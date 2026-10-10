//! kotlin: the Kotlin logo, a K cut from a square, with a sheen crossing it
//! every few seconds. After ascii.rest's kotlin.ts (MIT, @bas3line).
//!
//! Drawn from devicon's kotlin-original.svg (MIT) by scripts/logo.py, into
//! art/kotlin.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from kotlin-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The kotlin logo's meta.
pub static META: Meta = Meta {
    name: "kotlin",
    slug: "kotlin",
    category: Category::Logos,
    note: "the k, in violet and pink, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 5 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#c01ae1", "#963df5", "#d124b1", "#ac2aec", "#db3683", "#d66aec", "#bb81f9", "#e171cc", "#c975f3", "#e87cae", "#ecbaf6", "#e0c5fc", "#f1bde8",
        "#e6bff9", "#f4c3da", "#c01ae1", "#963df5", "#d124b1", "#ac2aec", "#db3683", "#d66aec", "#bb81f9", "#e171cc", "#c975f3", "#e87cae", "#ecbaf6",
        "#e0c5fc", "#f1bde8", "#e6bff9", "#f4c3da",
    ]),
    ..Meta::DEFAULT
};

/// The kotlin logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/kotlin.txt"), options)
}
