//! The source-owning tree, whose ranges address the text it was parsed from, and the tolerant parse behind it.

pub(crate) mod xml_element_span;
pub(crate) mod xml_repair;
pub(crate) mod xml_source_document;
pub(crate) mod xml_unescape;

pub use crate::source::xml_element_span::XmlElementSpan;
pub use crate::source::xml_source_document::XmlSourceDocument;
