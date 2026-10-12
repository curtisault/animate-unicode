//! Interface pieces: spinners, progress bars: the `ui` category, and the `ui` feature that
//! builds it into its own wasm module. A piece here may use the crate's shared
//! code and this folder's other pieces, never another category's.

use crate::registry::Entry;

pub mod boot_log;
pub mod box_frames;
pub mod calendar;
pub mod digital_clock;
pub mod dividers;
pub mod file_tree;
pub mod form_controls;
pub mod not_found;
pub mod progress_bar;
pub mod skeleton;
pub mod spinners;
pub mod terminal;

/// The ui pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &box_frames::META, make: box_frames::make },
    Entry { meta: &boot_log::META, make: boot_log::make },
    Entry { meta: &calendar::META, make: calendar::make },
    Entry { meta: &digital_clock::META, make: digital_clock::make },
    Entry { meta: &dividers::META, make: dividers::make },
    Entry { meta: &spinners::META, make: spinners::make },
    Entry { meta: &progress_bar::META, make: progress_bar::make },
    Entry { meta: &skeleton::META, make: skeleton::make },
    Entry { meta: &file_tree::META, make: file_tree::make },
    Entry { meta: &form_controls::META, make: form_controls::make },
    Entry { meta: &not_found::META, make: not_found::make },
    Entry { meta: &terminal::META, make: terminal::make },
];
