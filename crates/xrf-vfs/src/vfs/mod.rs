//! The VFS snapshot: ordered mounts searched first-hit-wins, the revisions of what they hold, and the scoped view a
//! search step reads through.

#[cfg(test)]
pub(crate) mod tests;
mod xray_scoped_vfs;
mod xray_vfs;

pub use xray_scoped_vfs::XrayScopedVfs;
pub use xray_vfs::XrayVfs;
