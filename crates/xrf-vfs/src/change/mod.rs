//! Changing what a VFS snapshot holds, through two doors into one path: a transaction committed to disk
//! ([`XrayVfs::commit`](crate::XrayVfs::commit)), or changes already on disk applied
//! ([`XrayVfs::apply`](crate::XrayVfs::apply)). Both answer the next snapshot and what changed.

mod xray_change;
mod xray_change_set;
mod xray_commit;
mod xray_disk_check;
mod xray_disk_journal;
mod xray_disk_swap;
mod xray_journal_entry;
mod xray_swapped_file;
mod xray_transaction;
mod xray_transaction_target;
mod xray_transaction_write;

pub use xray_change::{XrayChange, XrayChangeKind};
pub use xray_change_set::XrayChangeSet;
pub(crate) use xray_disk_journal::XrayDiskJournal;
pub use xray_transaction::XrayTransaction;
