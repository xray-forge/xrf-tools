//! The virtual file system: ordered mounts searched first-hit-wins, the lookup scope that narrows a search to some of
//! them, and the ordered probe that searches several scopes in turn.

mod directory_listing;
#[cfg(test)]
mod tests;
mod xray_lookup_scope;
mod xray_mounted_entry;
mod xray_probe;
mod xray_resolution;
mod xray_scoped_vfs;
mod xray_vfs;

pub use directory_listing::XrayDirectoryListing;
pub use xray_lookup_scope::XrayLookupScope;
pub use xray_mounted_entry::XrayMountedEntry;
pub use xray_probe::{XrayProbe, XrayProbeStep};
pub use xray_resolution::XrayResolution;
pub use xray_scoped_vfs::XrayScopedVfs;
pub use xray_vfs::XrayVfs;
