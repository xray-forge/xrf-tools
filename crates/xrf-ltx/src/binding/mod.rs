//! How a typed record is read out of a config and written back into one, a section or a whole file at a time.
//!
//! A record reads its fields through [`crate::Section::read_field`] and [`crate::Section::read_optional_field`], and
//! writes them through [`LtxWrite`]: into an [`crate::Ltx`] when the caller keeps the config, or straight into a file's
//! text through [`LtxRecordWriter`] when it only writes the file.

pub(crate) mod file_import_export;
pub(crate) mod ltx_import_export;
pub(crate) mod ltx_record_section;
pub(crate) mod ltx_record_writer;
pub(crate) mod ltx_section_write;
pub(crate) mod ltx_write;

pub use crate::binding::file_import_export::FileImportExport;
pub use crate::binding::ltx_import_export::{LtxImportExport, META_TYPE_FIELD};
pub use crate::binding::ltx_record_section::LtxRecordSection;
pub use crate::binding::ltx_record_writer::LtxRecordWriter;
pub use crate::binding::ltx_section_write::LtxSectionWrite;
pub use crate::binding::ltx_write::LtxWrite;
