use xrf_error::{XrfError, XrfResult};

use crate::encoding::decode_xml_bytes;
use crate::options::XmlParseOptions;
use crate::xml_element::XmlElement;

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
  /// Returns a parsing error when the input is not a well-formed XML document.
  pub fn parse(input: &str, options: XmlParseOptions) -> XrfResult<Self> {
    let document: roxmltree::Document = roxmltree::Document::parse_with_options(
      input,
      roxmltree::ParsingOptions {
        allow_dtd: options.allow_dtd,
        ..roxmltree::ParsingOptions::default()
      },
    )
    .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse XML: {error}")))?;

    Ok(Self {
      root: XmlElement::from_node(document.root_element()),
    })
  }

  /// Decode and parse XML bytes according to their declaration, defaulting to UTF-8.
  ///
  /// # Errors
  ///
  /// Returns an encoding error for unsupported or invalid input encodings, or a parsing error for
  /// malformed XML.
  pub fn parse_bytes(input: &[u8], options: XmlParseOptions) -> XrfResult<Self> {
    Self::parse(&decode_xml_bytes(input)?, options)
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
