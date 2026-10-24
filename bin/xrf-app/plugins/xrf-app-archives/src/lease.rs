//! Action exclusion shared by archive publication commands.

/// Packing and patching share one publication lane, independently of their output directory claims.
pub const PUBLISH_ACTION_GROUP: &str = "archives.publish";
