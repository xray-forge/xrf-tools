//! Which rules turn a tree of documents into a resolved config, and what a resolution answers with.

pub(crate) mod ltx_dialect;
pub(crate) mod ltx_text_interner;
pub(crate) mod resolution;
pub(crate) mod standard;

pub use crate::dialect::ltx_dialect::LtxDialect;
pub use crate::dialect::ltx_text_interner::LtxTextInterner;
pub use crate::dialect::resolution::{
  LtxFieldOrigin, LtxProvenance, LtxResolution, LtxResolutionDiagnostic, LtxResolveRequest,
};
pub(crate) use crate::dialect::standard::LtxOpenSection;
pub use crate::dialect::standard::LtxStandardDialect;
