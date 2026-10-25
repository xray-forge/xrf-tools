//! Composing sources into a searchable order: what to mount, where, and how a path is turned into a plan.

mod mount_plan;
mod skipped_mount;
#[cfg(test)]
mod tests;
mod xray_mount;
mod xray_mount_id;
mod xray_mount_mode;
mod xray_mount_plan;
mod xray_planned_mount;
mod xray_root;
mod xray_root_kind;
mod xray_root_probe;
mod xray_roots;

pub use skipped_mount::XraySkippedMount;
pub use xray_mount::XrayMount;
pub use xray_mount_id::XrayMountId;
pub use xray_mount_mode::XrayMountMode;
pub(crate) use xray_mount_plan::XrayMountPlan;
pub(crate) use xray_planned_mount::XrayPlannedMount;
pub use xray_root::XrayRoot;
pub use xray_root_kind::XrayRootKind;
pub use xray_root_probe::XrayRootProbe;
pub use xray_roots::XrayRoots;
