//! cloudflare: the Cloudflare logo, an orange cloud, with a sheen crossing it
//! every few seconds. After ascii.rest's cloudflare.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' cloudflare.svg (CC0) by scripts/logo.py, into
//! art/cloudflare.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colour, lifted on a dark page where it would sink into it;
//! in a `<pre>` it is one ink, from the same drawing with its white left bare.
//! The logo is a trademark of its owner, shown here to name the company.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The cloudflare logo's meta.
pub static META: Meta = Meta {
    name: "cloudflare",
    slug: "cloudflare",
    category: Category::Companies,
    note: "the orange cloud, glinting now and then",
    cols: 79,
    rows: 19,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#f38020", "#f7ac6e", "#fbd9bc", "#f38020", "#f7ac6e", "#fbd9bc"]),
    ..Meta::DEFAULT
};

/// The cloudflare logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/cloudflare.txt"), options)
}
