//! Turning what a tree declared into the values the engine would load, and explaining them.
//!
//! Parents first, then a section's own base merged with its overrides, then list operations, then section deletions.
//! See the compatibility matrix, section 4.

pub(crate) mod dltx_diagnostic;
pub(crate) mod dltx_provenance;
pub(crate) mod dltx_resolve_result;
pub(crate) mod dltx_resolver;
