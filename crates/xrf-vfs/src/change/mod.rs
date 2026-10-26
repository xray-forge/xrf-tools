//! Changing what a VFS snapshot holds, through two doors into one path: a transaction committed to disk
//! ([`XrayVfs::commit`](crate::XrayVfs::commit)), or changes already on disk applied
//! ([`XrayVfs::apply`](crate::XrayVfs::apply)). Both answer the next snapshot and what changed.

mod disk;
mod transaction;
mod xray_change;
mod xray_change_kind;
mod xray_change_set;
mod xray_vfs_commit;
mod xray_vfs_disk_check;

pub(crate) use disk::XrayDiskJournal;
pub use transaction::XrayTransaction;
pub use xray_change::XrayChange;
pub use xray_change_kind::XrayChangeKind;
pub use xray_change_set::XrayChangeSet;
