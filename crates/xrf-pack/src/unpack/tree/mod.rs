//! A selection of entries laid out below a destination root, every directory first and then every payload.

mod tree_entry;
mod tree_write_summary;
mod tree_writer;

pub(crate) use tree_entry::TreeEntry;
pub(crate) use tree_write_summary::TreeWriteSummary;
pub(crate) use tree_writer::TreeWriter;
