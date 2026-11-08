//! Action exclusion shared by archive publication commands.

/// Packing and patching share one publication lane, independently of their output directory claims.
pub(crate) const PUBLISH_ACTION_GROUP: &str = "archives.publish";
