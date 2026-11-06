//! Writing records: the trait an export writes through, and the writer that appends a record file's text.

pub(crate) mod ltx_record_section;
pub(crate) mod ltx_record_writer;
pub(crate) mod ltx_section_write;
pub(crate) mod ltx_write;

pub use crate::binding::write::ltx_record_section::LtxRecordSection;
pub use crate::binding::write::ltx_record_writer::LtxRecordWriter;
pub use crate::binding::write::ltx_section_write::LtxSectionWrite;
pub use crate::binding::write::ltx_write::LtxWrite;
