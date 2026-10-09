//! Every piece by slug. Each category's `pieces/<category>/mod.rs` lists its
//! pieces; nothing else needs editing when a piece is added.

use crate::{Category, Meta, Options, Piece};

/// One piece in the registry: its meta and how to make one.
pub struct Entry {
    /// What the piece says about itself.
    pub meta: &'static Meta,
    /// Builds a fresh instance from the option overrides (already layered on the defaults by `make`).
    pub make: fn(&Options) -> Box<dyn Piece>,
}

/// Every piece built in, category by category in [`Category::ALL`]'s order.
pub fn all() -> impl Iterator<Item = &'static Entry> + Clone {
    crate::pieces::CATEGORIES.iter().flat_map(|&(_, entries)| entries)
}

/// Every category built in (each a Cargo feature), with its pieces.
pub fn categories() -> &'static [(Category, &'static [Entry])] {
    crate::pieces::CATEGORIES
}

/// The piece with this slug.
pub fn find(slug: &str) -> Option<&'static Entry> {
    all().find(|e| e.meta.slug == slug)
}

/// Every slug, sorted.
pub fn slugs() -> Vec<&'static str> {
    let mut v: Vec<_> = all().map(|e| e.meta.slug).collect();
    v.sort_unstable();
    v
}

/// A piece by slug, with JSON option overrides layered on its defaults.
pub fn make(slug: &str, options_json: &str) -> Result<(&'static Meta, Box<dyn Piece>), MakeError> {
    let entry = find(slug).ok_or_else(|| MakeError::NoSuchPiece(slug.to_string()))?;
    let options = Options::from_json(options_json).map_err(|e| MakeError::BadOptions(e.to_string()))?.with_defaults(entry.meta.options);
    Ok((entry.meta, (entry.make)(&options)))
}

/// Why `make` failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MakeError {
    /// No registered piece has this slug.
    NoSuchPiece(String),
    /// The options were not valid JSON; the parser's message.
    BadOptions(String),
}

impl std::fmt::Display for MakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MakeError::NoSuchPiece(s) => write!(f, "there is no piece named \"{s}\""),
            MakeError::BadOptions(e) => write!(f, "options are not a JSON object: {e}"),
        }
    }
}

impl std::error::Error for MakeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_unique_and_registered() {
        let s = slugs();
        let mut dedup = s.clone();
        dedup.dedup();
        assert_eq!(s, dedup, "duplicate slugs in pieces/mod.rs");
        assert!(find("donut").is_some());
        assert!(matches!(make("nope", ""), Err(MakeError::NoSuchPiece(_))));
        assert!(matches!(make("donut", "{"), Err(MakeError::BadOptions(_))));
    }
}
