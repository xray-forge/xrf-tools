//! What two mounted worlds differ by, and the vocabulary the difference is reported in.

mod archive_patch_change;
mod archive_patch_class;
mod archive_patch_comparison;
mod archive_patch_side;

pub use archive_patch_change::ArchivePatchChange;
pub use archive_patch_class::ArchivePatchClass;
pub(crate) use archive_patch_comparison::ArchivePatchComparison;
pub use archive_patch_side::ArchivePatchSide;
