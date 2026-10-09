//! perl: the Perl logo, an onion, with a glint crossing it every few seconds.
//! After ascii.rest's perl.ts (MIT, @bas3line).
//!
//! Drawn from devicon's perl-original.svg (MIT) by scripts/logo.py, into
//! art/perl.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from perl-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The perl logo's meta.
pub static META: Meta = Meta {
    name: "perl",
    slug: "perl",
    category: Category::Logos,
    note: "the onion, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 2 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&["#212178", "#fefefe", "#6f6fa7", "#fefefe", "#bcbcd7", "#ffffff", "#4f4fcf", "#fefefe", "#8d8de0", "#fefefe", "#cacaf1", "#ffffff"]),
    ..Meta::DEFAULT
};

/// The perl logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/perl.txt"), Loop::Glint, options)
}
