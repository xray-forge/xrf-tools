//! Turning what a tree declared into the values the engine would load, and explaining them.
//!
//! Parents first, then a section's own base merged with its overrides, then list operations, then section deletions,
//! the order `EvaluateSection` resolves in (`Xr_ini.cpp`).

pub(crate) mod dltx_diagnostic;
pub(crate) mod dltx_field_origin;
pub(crate) mod dltx_provenance;
pub(crate) mod dltx_resolve_result;
pub(crate) mod dltx_resolver;
