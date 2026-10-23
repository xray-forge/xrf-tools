//! The VFS snapshot: ordered mounts searched first-hit-wins, the revisions of what they hold, and what it retains.

#[cfg(test)]
pub(crate) mod tests;
mod xray_vfs;

pub use xray_vfs::XrayVfs;
