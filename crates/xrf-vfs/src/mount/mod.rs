//! Composing sources into a searchable order: what to mount, where, and how a path is turned into a plan.

pub(crate) mod fsgame;
mod plan;
mod root;
#[cfg(test)]
mod tests;
mod xray_mount;
mod xray_mount_id;
mod xray_mount_mode;

pub use fsgame::{FsgameDeclaration, FsgameFile};
pub use plan::XraySkippedMount;
pub(crate) use plan::{XrayMountPlan, XrayPlannedMount};
pub use root::{XrayRoot, XrayRootKind, XrayRootProbe, XrayRoots};
pub use xray_mount::XrayMount;
pub use xray_mount_id::XrayMountId;
pub use xray_mount_mode::XrayMountMode;
