//! How a VFS snapshot is read: the ordered search, its steps and their scopes, the plan a search is mounted from, and
//! what lookups and listings answer.

mod listing;
mod step;
#[cfg(test)]
mod tests;
mod xray_read_handle;
mod xray_resolution;
mod xray_search;
mod xray_search_plan;

pub use listing::{XrayDirectoryListing, XrayMountedEntry, XraySearchedSource, XrayShadowedCopy, XrayShadowingEntry};
pub(crate) use step::XrayMountSelection;
pub use step::{XrayLookupScope, XraySearchKey, XraySearchStep};
pub use xray_read_handle::XrayReadHandle;
pub use xray_resolution::XrayResolution;
pub use xray_search::XraySearch;
pub use xray_search_plan::XraySearchPlan;
