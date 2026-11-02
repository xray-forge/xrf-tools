//! One resolution, indexed and sliceable a page of sections at a time.

pub(crate) mod ltx_resolved_diagnostic;
pub(crate) mod ltx_resolved_field;
pub(crate) mod ltx_resolved_field_origin;
pub(crate) mod ltx_resolved_index;
pub(crate) mod ltx_resolved_index_entry;
pub(crate) mod ltx_resolved_reader;
pub(crate) mod ltx_resolved_section;

pub use crate::resolved::ltx_resolved_diagnostic::LtxResolvedDiagnostic;
pub use crate::resolved::ltx_resolved_field::LtxResolvedField;
pub use crate::resolved::ltx_resolved_field_origin::LtxResolvedFieldOrigin;
pub use crate::resolved::ltx_resolved_index::LtxResolvedIndex;
pub use crate::resolved::ltx_resolved_index_entry::LtxResolvedIndexEntry;
pub(crate) use crate::resolved::ltx_resolved_reader::LtxResolvedReader;
pub use crate::resolved::ltx_resolved_section::LtxResolvedSection;
