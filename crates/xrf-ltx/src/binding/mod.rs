//! How a typed record is read out of a config and written back into one, a section or a whole file at a time.
//!
//! A record reads its fields through [`crate::Section::read_field`] and [`crate::Section::read_optional_field`].

pub(crate) mod file_import_export;
pub(crate) mod ltx_import_export;

pub use crate::binding::file_import_export::FileImportExport;
pub use crate::binding::ltx_import_export::{LtxImportExport, META_TYPE_FIELD};
