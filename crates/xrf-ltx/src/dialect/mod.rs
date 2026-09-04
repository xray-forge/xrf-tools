//! Which rules turn a tree of documents into a resolved config, and what a resolution answers with.

pub(crate) mod ltx_dialect;
pub(crate) mod ltx_resolution;
pub(crate) mod ltx_standard_dialect;
pub(crate) mod ltx_standard_lower;

pub use crate::dialect::ltx_dialect::LtxDialect;
pub use crate::dialect::ltx_resolution::{LtxFieldOrigin, LtxResolution, LtxResolutionDiagnostic};
pub use crate::dialect::ltx_standard_dialect::LtxStandardDialect;
