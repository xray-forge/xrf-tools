//! Which rules turn a tree of documents into a resolved config, and what a resolution answers with.

pub(crate) mod ltx_dialect;
pub(crate) mod ltx_field_origin;
pub(crate) mod ltx_open_section;
pub(crate) mod ltx_provenance;
pub(crate) mod ltx_resolution;
pub(crate) mod ltx_resolution_diagnostic;
pub(crate) mod ltx_resolve_request;
pub(crate) mod ltx_standard_dialect;
pub(crate) mod ltx_standard_lower;
pub(crate) mod ltx_text_interner;

pub use crate::dialect::ltx_dialect::LtxDialect;
pub use crate::dialect::ltx_field_origin::LtxFieldOrigin;
pub(crate) use crate::dialect::ltx_open_section::LtxOpenSection;
pub use crate::dialect::ltx_provenance::LtxProvenance;
pub use crate::dialect::ltx_resolution::LtxResolution;
pub use crate::dialect::ltx_resolution_diagnostic::LtxResolutionDiagnostic;
pub use crate::dialect::ltx_resolve_request::LtxResolveRequest;
pub use crate::dialect::ltx_standard_dialect::LtxStandardDialect;
pub use crate::dialect::ltx_text_interner::LtxTextInterner;
