//! What a description answers: the entry, the format it was read as, why one was not, and what it resolves.

mod archive_bounds;
mod archive_describe_refusal;
mod archive_describe_scope;
mod archive_file_description;
mod archive_format_description;
mod archive_reference;
mod archive_reference_status;

pub use archive_bounds::ArchiveBounds;
pub use archive_describe_refusal::ArchiveDescribeRefusal;
pub use archive_describe_scope::ArchiveDescribeScope;
pub use archive_file_description::ArchiveFileDescription;
pub use archive_format_description::ArchiveFormatDescription;
pub use archive_reference::ArchiveReference;
pub use archive_reference_status::ArchiveReferenceStatus;
