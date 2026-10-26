//! One place a search looks: its label, the mounts and subtree its scope admits, and the key the steps make together.

mod xray_lookup_scope;
mod xray_mount_selection;
mod xray_search_key;
mod xray_search_step;

pub use xray_lookup_scope::XrayLookupScope;
pub(crate) use xray_mount_selection::XrayMountSelection;
pub use xray_search_key::XraySearchKey;
pub use xray_search_step::XraySearchStep;
