//! Every piece, in a folder for each category. To add one: a file in its
//! category's folder, a `pub mod` there, and one line in that folder's
//! `REGISTRY`. A category new to the crate also gets a `pub mod` and a line in
//! [`CATEGORIES`] here.
//!
//! Each category is a Cargo feature, all of them on by default, so the wasm
//! can be built one category to a module and a page loads only the categories
//! it shows. That is why a piece may use the crate's shared code and its own
//! folder's pieces, never another category's: the module built for its
//! category alone would not have them.
//!
//! The contract test (`tests/contract.rs`) checks every piece against the
//! rules in docs/IMPLEMENTATION_PLAN.md.

use crate::registry::Entry;
use crate::Category;

#[cfg(feature = "data")]
pub mod data;
#[cfg(feature = "effects")]
pub mod effects;
#[cfg(feature = "generative")]
pub mod generative;
#[cfg(feature = "physics")]
pub mod physics;
#[cfg(feature = "shapes")]
pub mod shapes;
#[cfg(feature = "type")]
pub mod r#type;
#[cfg(feature = "ui")]
pub mod ui;

/// Every category built in, with its pieces, in the site's order ([`Category::ALL`]).
pub static CATEGORIES: &[(Category, &[Entry])] = &[
    #[cfg(feature = "ui")]
    (Category::Ui, ui::REGISTRY),
    #[cfg(feature = "data")]
    (Category::Data, data::REGISTRY),
    #[cfg(feature = "type")]
    (Category::Type, r#type::REGISTRY),
    #[cfg(feature = "shapes")]
    (Category::Shapes, shapes::REGISTRY),
    #[cfg(feature = "physics")]
    (Category::Physics, physics::REGISTRY),
    #[cfg(feature = "generative")]
    (Category::Generative, generative::REGISTRY),
    #[cfg(feature = "effects")]
    (Category::Effects, effects::REGISTRY),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_piece_is_in_its_categorys_folder() {
        for &(category, entries) in CATEGORIES {
            for e in entries {
                assert_eq!(e.meta.category, category, "{} is in the {} folder", e.meta.slug, category.slug());
            }
        }
        let order: Vec<Category> = CATEGORIES.iter().map(|&(c, _)| c).collect();
        let mut sorted = order.clone();
        sorted.sort_by_key(|c| Category::ALL.iter().position(|a| a == c));
        sorted.dedup();
        assert_eq!(order, sorted, "CATEGORIES out of Category::ALL's order, or a category twice");
    }
}
