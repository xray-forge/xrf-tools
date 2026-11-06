//! How a typed record is read out of a config and written back into one, a section or a whole file at a time.
//!
//! A record reads its fields through [`LtxRead`] and [`LtxSectionRead`]: from an [`crate::Ltx`] when the caller holds
//! the config, or one section at a time through [`LtxRecordReader`] as a record file is parsed. It writes them through
//! [`LtxWrite`]: into an [`crate::Ltx`] when the caller keeps the config, or straight into a file's text through
//! [`LtxRecordWriter`] when it only writes the file.

pub(crate) mod file_import_export;
pub(crate) mod ltx_field_index;
pub(crate) mod ltx_import_export;
pub(crate) mod read;
pub(crate) mod write;

pub use crate::binding::file_import_export::FileImportExport;
pub(crate) use crate::binding::ltx_field_index::LtxFieldIndex;
pub use crate::binding::ltx_import_export::{LtxImportExport, META_TYPE_FIELD};
pub use crate::binding::read::{LtxRead, LtxRecord, LtxRecordReader, LtxSectionRead};
pub use crate::binding::write::{LtxRecordSection, LtxRecordWriter, LtxSectionWrite, LtxWrite};
