//! The semantic tree, detached from the input it was parsed from.

pub(crate) mod xml_document;
pub(crate) mod xml_element;

pub use crate::document::xml_document::XmlDocument;
pub use crate::document::xml_element::XmlElement;
