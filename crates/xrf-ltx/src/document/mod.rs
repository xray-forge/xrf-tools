//! One LTX file as it was written: the scan that reads it, the statements it turned out to be, and the canonical
//! rendering of those statements back to text.

pub(crate) mod item;
pub(crate) mod ltx_check;
pub(crate) mod ltx_document;
pub(crate) mod ltx_document_format;
pub(crate) mod ltx_parser;
pub(crate) mod ltx_statement_writer;

#[cfg(test)]
mod tests;

pub use crate::document::item::{LtxItem, LtxItemKind, LtxKeyOperation, LtxSectionOperation, LtxSpan};
pub use crate::document::ltx_check::LtxCheck;
pub use crate::document::ltx_document::LtxDocument;
pub use crate::document::ltx_parser::LtxParser;
pub(crate) use crate::document::ltx_statement_writer::LtxStatementWriter;
