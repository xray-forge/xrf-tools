//! Producing XML: escaping values spliced into a document, and serializing a model into a new one.

pub(crate) mod xml_escape;
pub(crate) mod xml_serializer;

pub use crate::write::xml_escape::XmlEscape;
pub use crate::write::xml_serializer::XmlSerializer;
