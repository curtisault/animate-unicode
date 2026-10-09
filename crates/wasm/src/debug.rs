//! Pieces for checking the shell, compiled only with the `debug` feature.
//! `Player::new` accepts their slugs; the registry never lists them, so no
//! release page can play one. web/test and site/test.html use them.

use animate_unicode::registry::Entry;
use animate_unicode::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// Every debug piece, by slug.
pub static PIECES: &[Entry] = &[Entry { meta: &PANICS, make: |_: &Options| Box::new(Panics) }, Entry { meta: &WIDE, make: |_: &Options| Box::new(Wide) }];

/// The debug piece with this slug.
pub fn find(slug: &str) -> Option<&'static Entry> {
    PIECES.iter().find(|e| e.meta.slug == slug)
}

static PANICS: Meta = Meta {
    name: "debug panic",
    slug: "debug-panic",
    note: "draws for a second, then writes one cell past the end of its grid",
    cols: 4,
    rows: 2,
    ..Meta::DEFAULT
};

/// Draws for its first second, then writes one cell past the end of its grid
/// on every frame: what a bug in a piece looks like at run time.
struct Panics;

impl Piece for Panics {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        grid.clear();
        grid.text_at(0, 0, "fine", 0);
        grid.text_at(0, 1, "....", 0);
        if t < 1.0 {
            return;
        }
        let past_the_end = grid.cells().len();
        grid.cells_mut()[past_the_end] = '!' as u32;
    }
}

static WIDE: Meta = Meta {
    name: "debug wide",
    slug: "debug-wide",
    category: Category::Type,
    note: "double-width glyphs between narrow ones, one moving, colours changing",
    cols: 14,
    rows: 3,
    fps: 4,
    charset: Charset::Wide,
    palette: Some(&["#d8dee9", "#ff7a59", "#5ec8ff"]),
    ground: Some("#14181f"),
    ..Meta::DEFAULT
};

/// Wide glyphs for checking the renderers: two CJK glyphs that change colour
/// every second, a fire emoji that steps along its row (so tails move and the
/// cells it leaves must clear), and a ruler of narrow glyphs underneath to
/// see whether the columns still line up.
struct Wide;

impl Piece for Wide {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let tick = (t * 2.0) as i64;
        grid.clear();
        let ink = if (t as i64) % 2 == 0 { 1 } else { 2 };
        grid.set(0, 0, 'a', 0);
        grid.set_wide(1, 0, '漢', ink);
        grid.set_wide(3, 0, '字', ink);
        grid.text_at(5, 0, "b·c", 0);
        grid.set_wide(8, 0, '中', 3 - ink);
        grid.text_at(10, 0, "d·e·", 0);
        grid.text_at(0, 1, "··············", 0);
        grid.set_wide(tick % 13, 1, '🔥', 1);
        grid.text_at(0, 2, "0123456789abcd", 0);
    }
}
