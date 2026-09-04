//! Where a document comes from.
//!
//! [`ltx_document_source::LtxDocumentSource`] is the port a dialect resolves through, so the rules are written once and
//! answered by whichever surface holds the configs - a directory on disk, or an installation's mounted volumes.

pub(crate) mod ltx_document_source;
pub(crate) mod ltx_filesystem_source;
pub(crate) mod ltx_include_source;
pub(crate) mod ltx_vfs_source;

pub use crate::source::ltx_document_source::LtxDocumentSource;
pub use crate::source::ltx_filesystem_source::LtxFilesystemSource;
pub(crate) use crate::source::ltx_include_source::LtxIncludeSource;
pub(crate) use crate::source::ltx_vfs_source::LtxVfsSource;
