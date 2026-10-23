//! How a VFS snapshot is read: the ordered search, its steps and their scopes, the plan a search is mounted from, and
//! what lookups and listings answer.

mod directory_listing;
#[cfg(test)]
mod tests;
mod xray_lookup_scope;
mod xray_mounted_entry;
mod xray_resolution;
mod xray_search;
mod xray_search_plan;
mod xray_search_step;
mod xray_shadowing_entry;

pub use directory_listing::XrayDirectoryListing;
pub use xray_lookup_scope::XrayLookupScope;
pub(crate) use xray_lookup_scope::XrayMountSelection;
pub use xray_mounted_entry::XrayMountedEntry;
pub use xray_resolution::XrayResolution;
pub use xray_search::{XrayReadHandle, XraySearch};
pub use xray_search_plan::XraySearchPlan;
pub use xray_search_step::{XraySearchStep, XraySearchedSource};
pub use xray_shadowing_entry::{XrayShadowedCopy, XrayShadowingEntry};
