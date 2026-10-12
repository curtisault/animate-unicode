//! supabase: the Supabase logo, a lightning bolt, with a sheen crossing it
//! every few seconds. After ascii.rest's supabase.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' supabase.svg (CC0) by scripts/logo.py, into
//! art/supabase.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colour, and a deeper shade of it on a light page, where it
//! would fade into it; in a `<pre>` it is one ink, from the same drawing with
//! its white left bare. The logo is a trademark of its owner, shown here to
//! name the company.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The supabase logo's meta.
pub static META: Meta = Meta {
    name: "supabase",
    slug: "supabase",
    category: Category::Companies,
    note: "the green bolt, glinting now and then",
    cols: 55,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#2aa76e", "#75c6a1", "#bfe5d4", "#3fcf8e", "#82e0b6", "#c5f1dd"]),
    ..Meta::DEFAULT
};

/// The supabase logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/supabase.txt"), options)
}
