//! The places a caller names to read from, and what a named path turns out to be.

mod xray_root;
mod xray_root_kind;
mod xray_root_probe;
mod xray_roots;

pub use xray_root::XrayRoot;
pub use xray_root_kind::XrayRootKind;
pub use xray_root_probe::XrayRootProbe;
pub use xray_roots::XrayRoots;
