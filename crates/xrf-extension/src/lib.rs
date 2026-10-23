#![doc = include_str!("../README.md")]

mod file_extension;
mod xray_extension;
mod xray_extension_of;

pub use crate::file_extension::get_path_extension;
pub use crate::xray_extension::XrayExtension;
pub use crate::xray_extension_of::XrayExtensionOf;
