//! One resolution, indexed and sliceable a page of sections at a time.

pub(crate) mod ltx_resolved_diagnostic;
pub(crate) mod ltx_resolved_index;
pub(crate) mod ltx_resolved_reader;
pub(crate) mod ltx_resolved_section;

pub use crate::resolved::ltx_resolved_diagnostic::LtxResolvedDiagnostic;
pub use crate::resolved::ltx_resolved_index::{LtxResolvedIndex, LtxResolvedIndexEntry};
pub(crate) use crate::resolved::ltx_resolved_reader::LtxResolvedReader;
pub use crate::resolved::ltx_resolved_section::{LtxResolvedField, LtxResolvedFieldOrigin, LtxResolvedSection};
