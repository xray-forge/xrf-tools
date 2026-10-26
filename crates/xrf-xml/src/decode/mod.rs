//! Reading XML bytes before they are text: the declared encoding and the engine's `#include` splicing.

pub(crate) mod xml_encoding;
pub(crate) mod xml_includes;

pub use crate::decode::xml_encoding::XmlEncoding;
pub use crate::decode::xml_includes::XmlIncludes;
