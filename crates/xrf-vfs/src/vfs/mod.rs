//! The VFS snapshot: ordered mounts searched first-hit-wins, the revisions of what they hold, and what it retains; and
//! the host owning the current one.

#[cfg(test)]
pub(crate) mod tests;
mod xray_vfs;
mod xray_vfs_host;

pub use xray_vfs::XrayVfs;
pub use xray_vfs_host::XrayVfsHost;
