//! Linux distributions' marks: the `distros` category, and the `distros`
//! feature that builds it into its own wasm module. A piece here may use the
//! crate's shared code (`crate::logo` draws them all) and this folder's other
//! pieces, never another category's.

use crate::registry::Entry;

pub mod almalinux;
pub mod alpine_linux;
pub mod arch_linux;
pub mod centos;
pub mod debian;
pub mod deepin;
pub mod elementary_os;
pub mod endeavouros;
pub mod fedora;
pub mod gentoo;
pub mod kali_linux;
pub mod linux_mint;
pub mod manjaro;
pub mod nixos;
pub mod omarchy;
pub mod opensuse;
pub mod pop_os;
pub mod red_hat;
pub mod rocky_linux;
pub mod tux;
pub mod ubuntu;
pub mod void_linux;
pub mod zorin_os;

/// The distros pieces, in the order `registry::all` gives them.
pub static REGISTRY: &[Entry] = &[
    Entry { meta: &almalinux::META, make: almalinux::make },
    Entry { meta: &alpine_linux::META, make: alpine_linux::make },
    Entry { meta: &arch_linux::META, make: arch_linux::make },
    Entry { meta: &centos::META, make: centos::make },
    Entry { meta: &debian::META, make: debian::make },
    Entry { meta: &deepin::META, make: deepin::make },
    Entry { meta: &elementary_os::META, make: elementary_os::make },
    Entry { meta: &endeavouros::META, make: endeavouros::make },
    Entry { meta: &fedora::META, make: fedora::make },
    Entry { meta: &gentoo::META, make: gentoo::make },
    Entry { meta: &kali_linux::META, make: kali_linux::make },
    Entry { meta: &linux_mint::META, make: linux_mint::make },
    Entry { meta: &manjaro::META, make: manjaro::make },
    Entry { meta: &nixos::META, make: nixos::make },
    Entry { meta: &omarchy::META, make: omarchy::make },
    Entry { meta: &opensuse::META, make: opensuse::make },
    Entry { meta: &pop_os::META, make: pop_os::make },
    Entry { meta: &red_hat::META, make: red_hat::make },
    Entry { meta: &rocky_linux::META, make: rocky_linux::make },
    Entry { meta: &tux::META, make: tux::make },
    Entry { meta: &ubuntu::META, make: ubuntu::make },
    Entry { meta: &void_linux::META, make: void_linux::make },
    Entry { meta: &zorin_os::META, make: zorin_os::make },
];
