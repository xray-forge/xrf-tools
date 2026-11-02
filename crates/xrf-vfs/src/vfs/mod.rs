//! The VFS snapshot: ordered mounts searched first-hit-wins, the revisions of what they hold, what it retains after
//! parsing and what it read; and the host owning the current one.

pub(crate) mod cache;
#[cfg(test)]
pub(crate) mod tests;
pub(crate) mod trace;
mod xray_vfs;
mod xray_vfs_host;

pub use cache::{XrayCachePolicy, XrayCacheStats};
pub use trace::{XrayReadTrace, XrayReadTraceEntry, XrayReadTraceHotPath, XrayReadTraceSummary};
pub use xray_vfs::XrayVfs;
pub use xray_vfs_host::XrayVfsHost;
