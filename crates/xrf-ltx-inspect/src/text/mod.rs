//! One config's lines, as the authored view renders them.

pub(crate) mod ltx_file_text;
pub(crate) mod ltx_text_reader;

pub use crate::text::ltx_file_text::LtxFileText;
pub use crate::text::ltx_text_reader::read_text;
