//! What two mounted worlds differ by, and the vocabulary the difference is reported in.
//!
//! - [`deferred`] — the pairs the merge hands to the parallel checksum phase.

mod archive_patch_change;
mod archive_patch_class;
mod archive_patch_comparison;
mod archive_patch_origin;
mod archive_patch_origins;
mod archive_patch_side;
mod deferred;

pub use archive_patch_change::ArchivePatchChange;
pub use archive_patch_class::ArchivePatchClass;
pub(crate) use archive_patch_comparison::ArchivePatchComparison;
pub use archive_patch_origin::ArchivePatchOrigin;
pub(crate) use archive_patch_origins::ArchivePatchOrigins;
pub use archive_patch_side::ArchivePatchSide;
pub(crate) use deferred::{ArchivePatchDecision, ArchivePatchPending};
