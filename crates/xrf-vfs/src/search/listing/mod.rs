//! What a search's listings answer: the entries it reaches, the copies they hide, and the sources it searches.

mod xray_directory_listing;
mod xray_mounted_entry;
mod xray_searched_source;
mod xray_shadowed_copy;
mod xray_shadowing_entry;

pub use xray_directory_listing::XrayDirectoryListing;
pub use xray_mounted_entry::XrayMountedEntry;
pub use xray_searched_source::XraySearchedSource;
pub use xray_shadowed_copy::XrayShadowedCopy;
pub use xray_shadowing_entry::XrayShadowingEntry;
