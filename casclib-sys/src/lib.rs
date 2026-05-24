#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
// bindgen-emitted patterns in bindings_{linux,windows,macos}.rs. The Windows
// bindings in particular contain hundreds of bitfield accessor helpers that
// trip these. Scoped at the crate root rather than per-file so regenerating
// the bindings doesn't lose the suppressions.
#![allow(
    clippy::missing_safety_doc,
    clippy::ptr_offset_with_cast,
    clippy::redundant_static_lifetimes,
    clippy::too_many_arguments,
    clippy::unnecessary_cast,
    clippy::useless_transmute,
    unnecessary_transmutes
)]

#[cfg(target_os = "windows")]
include!("./bindings_windows.rs");
#[cfg(target_os = "linux")]
include!("./bindings_linux.rs");
#[cfg(target_os = "macos")]
include!("./bindings_macos.rs");

pub type ErrorCode = DWORD;
