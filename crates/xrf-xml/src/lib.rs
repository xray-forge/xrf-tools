#![doc = include_str!("../README.md")]

pub(crate) mod bytes;
pub(crate) mod decode;
pub(crate) mod document;
pub(crate) mod source;
pub(crate) mod write;
pub(crate) mod xml_nesting;
pub(crate) mod xml_parse_options;

pub use crate::decode::{XmlEncoding, XmlIncludes};
pub use crate::document::{XmlDocument, XmlElement};
pub use crate::source::{XmlElementSpan, XmlSourceDocument};
pub use crate::write::{XmlEscape, XmlSerializer};
pub use crate::xml_parse_options::XmlParseOptions;
