//! What one search step means before anything is opened, and how a snapshot mounts it.

mod xray_mount_plan;
mod xray_planned_mount;
mod xray_skipped_mount;
mod xray_vfs_mount_plan;

pub(crate) use xray_mount_plan::XrayMountPlan;
pub(crate) use xray_planned_mount::XrayPlannedMount;
pub use xray_skipped_mount::XraySkippedMount;
