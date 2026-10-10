//! lua: the Lua logo, a moon beside a planet lettered Lua, with a sheen
//! crossing it every few seconds. After ascii.rest's lua.ts (MIT, @bas3line).
//!
//! Drawn from devicon's lua-original.svg (MIT) by scripts/logo.py, into
//! art/lua.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from lua-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The lua logo's meta.
pub static META: Meta = Meta {
    name: "lua",
    slug: "lua",
    category: Category::Logos,
    note: "a planet and its moon, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 3 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#818186", "#ffffff", "#000080", "#adadb0", "#ffffff", "#5959ac", "#d9d9db", "#ffffff", "#b3b3d9", "#818186", "#ffffff", "#3e3eff", "#adadb0",
        "#ffffff", "#8282ff", "#d9d9db", "#ffffff", "#c5c5ff",
    ]),
    ..Meta::DEFAULT
};

/// The lua logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/lua.txt"), options)
}
