//! Every piece by slug. `pieces/mod.rs` lists them; nothing else needs editing
//! when a piece is added.

use crate::{Meta, Options, Piece};

pub struct Entry {
    pub meta: &'static Meta,
    /// Builds a fresh instance from the option overrides (already layered on the defaults by `make`).
    pub make: fn(&Options) -> Box<dyn Piece>,
}

/// Every piece, in the order `pieces/mod.rs` lists them.
pub fn all() -> &'static [Entry] {
    crate::pieces::REGISTRY
}

pub fn find(slug: &str) -> Option<&'static Entry> {
    all().iter().find(|e| e.meta.slug == slug)
}

/// Every slug, sorted.
pub fn slugs() -> Vec<&'static str> {
    let mut v: Vec<_> = all().iter().map(|e| e.meta.slug).collect();
    v.sort_unstable();
    v
}

/// A piece by slug, with JSON option overrides layered on its defaults.
pub fn make(slug: &str, options_json: &str) -> Result<(&'static Meta, Box<dyn Piece>), MakeError> {
    let entry = find(slug).ok_or_else(|| MakeError::NoSuchPiece(slug.to_string()))?;
    let options = Options::from_json(options_json).map_err(|e| MakeError::BadOptions(e.to_string()))?.with_defaults(entry.meta.options);
    Ok((entry.meta, (entry.make)(&options)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MakeError {
    NoSuchPiece(String),
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
