//! What a piece says about itself: its size, speed, colours and which glyphs
//! it draws with. Serialised to JSON for the site and the shell.

use serde::Serialize;

/// Where a piece sits in the site's sidebar and the README's table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Large coloured pictures with a palette and a ground (up to 320 × 120).
    Scenes,
    /// Geometry turning or morphing: the donut, a cube.
    Shapes,
    /// Planets, stars, orbits.
    Space,
    /// Simulations: pendulums, cradles, particles.
    Physics,
    /// Weather, water, plants.
    Nature,
    /// Animals and characters.
    Creatures,
    /// Things: clocks, machines.
    Objects,
    /// Patterns from a rule or a formula.
    Generative,
    /// Screen effects: fire, rain, glitches.
    Effects,
    /// Interface pieces: spinners, progress bars.
    Ui,
    /// Charts and readouts.
    Data,
    /// Lettering and text effects.
    Type,
    /// Marks and wordmarks.
    Logos,
}

impl Category {
    /// Sidebar order on the site, and the order of the README's table.
    pub const ALL: &'static [Category] = &[
        Category::Scenes,
        Category::Ui,
        Category::Data,
        Category::Type,
        Category::Logos,
        Category::Shapes,
        Category::Space,
        Category::Physics,
        Category::Nature,
        Category::Creatures,
        Category::Objects,
        Category::Generative,
        Category::Effects,
    ];

    /// The lowercase name used in JSON and URLs.
    pub fn slug(self) -> &'static str {
        match self {
            Category::Scenes => "scenes",
            Category::Shapes => "shapes",
            Category::Space => "space",
            Category::Physics => "physics",
            Category::Nature => "nature",
            Category::Creatures => "creatures",
            Category::Objects => "objects",
            Category::Generative => "generative",
            Category::Effects => "effects",
            Category::Ui => "ui",
            Category::Data => "data",
            Category::Type => "type",
            Category::Logos => "logos",
        }
    }
}

/// Which glyphs a piece draws with. The shell uses it to decide which font to
/// load and whether cells can be wide; the contract test holds a piece to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Charset {
    /// Printable ASCII, `·°•●`, box drawing (U+2500–257F) and block elements
    /// (U+2580–259F). What a system monospace face has, or the 3 KB fallback
    /// font covers.
    Basic,
    /// Basic plus braille (U+2800–28FF), geometric shapes (U+25A0–25FF),
    /// arrows (U+2190–21FF), and the sextants and octants of Symbols for
    /// Legacy Computing (U+1FB00–1FBFF). Needs the bundled font.
    Extended,
    /// Anything, including double-width glyphs (CJK, emoji) that take two
    /// cells. Colour emoji ignore the palette.
    Wide,
}

/// What a piece says about itself. Every piece has one, as `pub static META`.
#[derive(Clone, Debug, Serialize)]
pub struct Meta {
    /// Lowercase display name: "newton's cradle".
    pub name: &'static str,
    /// File and URL name: "newtons-cradle". Kebab-case, unique.
    pub slug: &'static str,
    /// Its group on the site.
    pub category: Category,
    /// One lowercase line, at most 72 characters, saying what you see.
    pub note: &'static str,
    /// Frame size in cells: every frame is exactly `rows` lines of `cols` cells.
    pub cols: usize,
    /// Frame height in cells.
    pub rows: usize,
    /// Frames a second; 0 for a still.
    pub fps: u32,
    /// The glyphs it may draw. The contract test holds it to this.
    pub charset: Charset,
    /// Defaults for every option the piece takes, as a JSON object. `None` for a piece with no options.
    pub options: Option<&'static str>,
    /// True if the picture depends on the real time or date.
    pub clock: bool,
    /// Up to 64 colours as #rrggbb, indexed by each cell's colour. A piece with one draws on a canvas.
    /// Shading runs go dimmest first, so `palette[0]` is the dimmest (see [`crate::palette`]).
    pub palette: Option<&'static [&'static str]>,
    /// The colour behind a coloured piece.
    pub ground: Option<&'static str>,
    /// Cell height in cell widths on a canvas: 2, the shape of a character, or 1 for a square grid.
    pub cell: u8,
}

impl Meta {
    /// Every field a piece need not think about, so a piece's `META` reads as
    /// `Meta { name: "donut", ..., ..Meta::DEFAULT }`.
    pub const DEFAULT: Meta = Meta {
        name: "",
        slug: "",
        category: Category::Shapes,
        note: "",
        cols: 80,
        rows: 24,
        fps: 30,
        charset: Charset::Basic,
        options: None,
        clock: false,
        palette: None,
        ground: None,
        cell: 2,
    };

    /// The meta as a JSON object, as the site and the shell read it.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Meta serialises")
    }
}
