//! Company marks: the `companies` category, and the `companies` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code (`crate::logo` draws them all) and this folder's other pieces, never
//! another category's.

use crate::registry::Entry;

pub mod apple;
pub mod cloudflare;
pub mod coderabbit;
pub mod helium;
pub mod mintlify;
pub mod planetscale;
pub mod playstation;
pub mod supabase;
pub mod vercel;

/// The companies pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &apple::META, make: apple::make },
    Entry { meta: &cloudflare::META, make: cloudflare::make },
    Entry { meta: &coderabbit::META, make: coderabbit::make },
    Entry { meta: &helium::META, make: helium::make },
    Entry { meta: &mintlify::META, make: mintlify::make },
    Entry { meta: &planetscale::META, make: planetscale::make },
    Entry { meta: &playstation::META, make: playstation::make },
    Entry { meta: &supabase::META, make: supabase::make },
    Entry { meta: &vercel::META, make: vercel::make },
];
