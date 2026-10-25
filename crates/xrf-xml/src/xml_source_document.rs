use xrf_error::XrfResult;

use crate::options::XmlParseOptions;
use crate::repair::repair_for_parsing;
use crate::xml_element_span::XmlElementSpan;

/// A parsed document together with the text it was parsed from.
///
/// The two travel together on purpose: every range an element reports is a byte offset into this
/// exact string, and separating them would leave offsets pointing into whatever the caller still
/// happened to be holding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlSourceDocument {
  source: String,
  root: XmlElementSpan,
}

impl XmlSourceDocument {
  /// Parse a document, keeping every element's position and the text those positions address.
  ///
  /// A strict parse is tried first, so well-formed documents are unaffected. Only on failure is a
  /// same-length repaired copy parsed instead - shipped game data contains comment banners and bare
  /// ampersands that XML forbids and the engine's own reader accepts.
  ///
  /// # Errors
  ///
  /// Returns a parsing error when neither the input nor its repaired copy is well-formed.
  pub fn parse(source: String, options: XmlParseOptions) -> XrfResult<Self> {
    let root: XmlElementSpan = match XmlElementSpan::parse_exact(&source, &source, options) {
      Ok(parsed) => parsed,
      // The repaired attempt's error is reported, not the strict one: it names whatever is still
      // wrong once the tolerated constructs are out of the way, which is what a caller has to act on.
      Err(_) => XmlElementSpan::parse_exact(&source, &repair_for_parsing(&source), options)?,
    };

    Ok(Self { source, root })
  }

  /// The text every range in this document addresses.
  pub fn source(&self) -> &str {
    &self.source
  }

  pub fn root(&self) -> &XmlElementSpan {
    &self.root
  }

  /// Take the source back, for a caller that is about to splice it.
  pub fn into_source(self) -> String {
    self.source
  }
}
