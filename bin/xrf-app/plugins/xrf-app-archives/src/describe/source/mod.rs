//! What a describer reads an entry through, whichever subject holds it, and which describer reads it.

mod archive_describe_source;
mod archive_described_format;
mod archive_entry_container;
mod archive_entry_reader;

pub(crate) use archive_describe_source::ArchiveDescribeSource;
pub(crate) use archive_described_format::ArchiveDescribedFormat;
pub(crate) use archive_entry_container::ArchiveEntryContainer;
pub(crate) use archive_entry_reader::ArchiveEntryReader;
