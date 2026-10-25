#![doc = include_str!("../README.md")]

pub(crate) mod encoding;
pub(crate) mod escape;
pub(crate) mod include;
pub(crate) mod options;
pub(crate) mod repair;
pub(crate) mod serialize;
pub(crate) mod unescape;
pub(crate) mod xml_document;
pub(crate) mod xml_element;
pub(crate) mod xml_element_span;
pub(crate) mod xml_source_document;

pub use crate::encoding::{declared_xml_encoding, encoding_from_label};
pub use crate::escape::{escape_xml_attribute, escape_xml_text};
pub use crate::include::{expand_xml_includes, list_xml_includes};
pub use crate::options::XmlParseOptions;
pub use crate::serialize::serialize_xml;
pub use crate::xml_document::XmlDocument;
pub use crate::xml_element::XmlElement;
pub use crate::xml_element_span::XmlElementSpan;
pub use crate::xml_source_document::XmlSourceDocument;
