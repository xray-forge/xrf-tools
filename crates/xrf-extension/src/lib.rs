#![doc = include_str!("../README.md")]
//!
//! # Module map
//!
//! Two of the three layers an extension question has, kept together because keeping them apart is what let the
//! workspace grow three answers to the same question:
//!
//! - `file_extension` — how an extension comes off a name, and the one comparison of one against a spelling.
//! - `xray_extension` — which extensions exist, as a closed vocabulary.
//! - `xray_extension_of` — the one door from a name to what its extension turned out to be.
//!
//! The third layer is policy — which extensions a crate reads as text, compresses, or skips — and it deliberately
//! does not live here. Policy is per consumer and the sets disagree; only their element type is shared.

mod file_extension;
mod xray_extension;
mod xray_extension_of;

pub use crate::file_extension::{get_file_extension, has_extension};
pub use crate::xray_extension::XrayExtension;
pub use crate::xray_extension_of::XrayExtensionOf;
