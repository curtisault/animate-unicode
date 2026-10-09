//! Linux distributions' marks: the `distros` category, and the `distros`
//! feature that builds it into its own wasm module. A piece here may use the
//! crate's shared code (`crate::logo` draws them all) and this folder's other
//! pieces, never another category's.

use crate::registry::Entry;

/// The distros pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[];
