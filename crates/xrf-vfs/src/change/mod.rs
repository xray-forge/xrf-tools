//! Changing what a VFS snapshot holds, through two doors into one path: a transaction committed to disk
//! ([`XrayVfs::commit`](crate::XrayVfs::commit)), or changes already on disk applied
//! ([`XrayVfs::apply`](crate::XrayVfs::apply)). Both answer the next snapshot and what changed.

mod xray_change;
mod xray_change_set;
mod xray_commit;
mod xray_disk_swap;
mod xray_transaction;

pub use xray_change::{XrayChange, XrayChangeKind};
pub use xray_change_set::XrayChangeSet;
pub use xray_transaction::XrayTransaction;
