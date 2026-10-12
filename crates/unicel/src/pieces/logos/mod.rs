//! Language marks and wordmarks: the `logos` category, and the `logos` feature
//! that builds it into its own wasm module. A piece here may use the crate's
//! shared code (`crate::logo` draws them all) and this folder's other pieces,
//! never another category's.

use crate::registry::Entry;

pub mod c;
pub mod clojure;
pub mod cpp;
pub mod csharp;
pub mod css;
pub mod dart;
pub mod elixir;
pub mod elm;
pub mod erlang;
pub mod go;
pub mod haskell;
pub mod html;
pub mod java;
pub mod javascript;
pub mod julia;
pub mod kotlin;
pub mod lua;
pub mod ocaml;
pub mod perl;
pub mod php;
pub mod python;
pub mod r;
pub mod ruby;
pub mod rust;
pub mod scala;
pub mod swift;
pub mod typescript;
pub mod zig;

/// The logos pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &c::META, make: c::make },
    Entry { meta: &clojure::META, make: clojure::make },
    Entry { meta: &cpp::META, make: cpp::make },
    Entry { meta: &csharp::META, make: csharp::make },
    Entry { meta: &css::META, make: css::make },
    Entry { meta: &dart::META, make: dart::make },
    Entry { meta: &elixir::META, make: elixir::make },
    Entry { meta: &elm::META, make: elm::make },
    Entry { meta: &erlang::META, make: erlang::make },
    Entry { meta: &go::META, make: go::make },
    Entry { meta: &haskell::META, make: haskell::make },
    Entry { meta: &html::META, make: html::make },
    Entry { meta: &java::META, make: java::make },
    Entry { meta: &javascript::META, make: javascript::make },
    Entry { meta: &julia::META, make: julia::make },
    Entry { meta: &kotlin::META, make: kotlin::make },
    Entry { meta: &lua::META, make: lua::make },
    Entry { meta: &ocaml::META, make: ocaml::make },
    Entry { meta: &perl::META, make: perl::make },
    Entry { meta: &php::META, make: php::make },
    Entry { meta: &python::META, make: python::make },
    Entry { meta: &r::META, make: r::make },
    Entry { meta: &ruby::META, make: ruby::make },
    Entry { meta: &rust::META, make: rust::make },
    Entry { meta: &scala::META, make: scala::make },
    Entry { meta: &swift::META, make: swift::make },
    Entry { meta: &typescript::META, make: typescript::make },
    Entry { meta: &zig::META, make: zig::make },
];
