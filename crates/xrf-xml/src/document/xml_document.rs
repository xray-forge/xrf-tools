use xrf_error::XrfResult;

use crate::decode::XmlEncoding;
use crate::document::XmlElement;
use crate::xml_parse_options::XmlParseOptions;

/// A parsed XML document detached from its input buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlDocument {
  root: XmlElement,
}

impl XmlDocument {
  /// Parse a UTF-8 XML string.
  ///
  /// # Errors
  ///
  /// Returns a parsing error when the input is not a well-formed XML document, or nests elements deeper than
  /// [`XmlParseOptions::MAX_DEPTH`].
  pub fn parse(input: &str, options: XmlParseOptions) -> XrfResult<Self> {
    Ok(Self {
      root: XmlElement::from_node(options.parse(input)?.root_element()),
    })
  }

  /// Decode and parse XML bytes according to their declaration, defaulting to UTF-8.
  ///
  /// # Errors
  ///
  /// Returns an encoding error for unsupported or invalid input encodings, or a parsing error for
  /// malformed XML.
  pub fn parse_bytes(input: &[u8], options: XmlParseOptions) -> XrfResult<Self> {
    Self::parse(&XmlEncoding::decode(input)?, options)
  }

  /// Returns the document element.
  pub fn root(&self) -> &XmlElement {
    &self.root
  }

  /// Iterates over the root and all descendants whose tag name equals `name`.
  ///
  /// Matching is case-sensitive and follows document order.
  pub fn elements_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a XmlElement> + 'a {
    std::iter::once(&self.root)
      .chain(self.root.descendants())
      .filter(move |element| element.name() == name)
  }
}

#[cfg(test)]
mod tests;
