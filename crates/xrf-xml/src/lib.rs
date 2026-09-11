#![doc = include_str!("../README.md")]

pub(crate) mod dom;
pub(crate) mod encoding;
pub(crate) mod escape;
pub(crate) mod options;
pub(crate) mod repair;
pub(crate) mod serialize;
pub(crate) mod spans;

pub use crate::dom::{XmlDocument, XmlElement};
pub use crate::encoding::{declared_xml_encoding, encoding_from_label};
pub use crate::escape::{escape_xml_attribute, escape_xml_text};
pub use crate::options::XmlParseOptions;
pub use crate::serialize::serialize_xml;
pub use crate::spans::{XmlElementSpan, XmlSourceDocument};
