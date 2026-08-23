//! Composing sources into a searchable order: what to mount, where, and how a path is turned into a plan.

mod mount_plan;
mod open;
mod skipped_mount;
#[cfg(test)]
mod tests;
mod xray_mount;
mod xray_mount_mode;
mod xray_mount_plan;
mod xray_probe_plan;
mod xray_root;
mod xray_world_spec;

pub use skipped_mount::XraySkippedMount;
pub use xray_mount::{XrayMount, XrayMountId};
pub use xray_mount_mode::XrayMountMode;
pub use xray_mount_plan::{XrayMountPlan, XrayPlannedMount};
pub use xray_probe_plan::XrayProbePlan;
pub use xray_world_spec::{XrayWorldRoot, XrayWorldSpec};
