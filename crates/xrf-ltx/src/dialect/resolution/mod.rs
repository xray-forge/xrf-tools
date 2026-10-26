//! What resolving one root answers: the config, where its fields came from, and what the dialect noticed.

pub(crate) mod ltx_field_origin;
pub(crate) mod ltx_provenance;
pub(crate) mod ltx_resolution;
pub(crate) mod ltx_resolution_diagnostic;
pub(crate) mod ltx_resolve_request;

pub use crate::dialect::resolution::ltx_field_origin::LtxFieldOrigin;
pub use crate::dialect::resolution::ltx_provenance::LtxProvenance;
pub use crate::dialect::resolution::ltx_resolution::LtxResolution;
pub use crate::dialect::resolution::ltx_resolution_diagnostic::LtxResolutionDiagnostic;
pub use crate::dialect::resolution::ltx_resolve_request::LtxResolveRequest;
