//! Where a document comes from.
//!
//! [`ltx_document_source::LtxDocumentSource`] is the port a dialect resolves through, so the rules are written once and
//! answered by whichever surface holds the configs - a directory on disk here, or an installation's mounted volumes in
//! `xrf-ltx-project`.

pub(crate) mod ltx_document_source;
pub(crate) mod ltx_filesystem_source;
pub(crate) mod ltx_include_mask;

pub use crate::source::ltx_document_source::LtxDocumentSource;
pub(crate) use crate::source::ltx_filesystem_source::LtxFilesystemSource;
pub use crate::source::ltx_include_mask::LtxIncludeMask;
