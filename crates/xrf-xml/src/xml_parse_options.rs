use xrf_error::{XrfError, XrfResult};

use crate::xml_nesting::XmlNesting;

/// Parsing behavior shared by X-Ray XML readers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct XmlParseOptions {
  /// Whether a document declaring a DTD is accepted. Off by default, because a DTD is an input the
  /// engine never writes and a parser feature worth not exposing to untrusted game data.
  pub allow_dtd: bool,
}

impl XmlParseOptions {
  /// How deep elements may nest, the root counting as one.
  ///
  /// The parser recurses once per open element, and both trees are built, compared, cloned and dropped the same way,
  /// so an unbounded depth would let a few kilobytes of nested tags overflow the stack and abort the process rather
  /// than fail the parse. An unoptimized parse takes over ten kilobytes of stack per level. Shipped game data nests
  /// eight levels at most; the bound leaves four times that while staying inside a one-megabyte main thread.
  pub const MAX_DEPTH: usize = 32;

  /// Parse `input` under these options, refusing a document nested deeper than [`Self::MAX_DEPTH`].
  ///
  /// # Errors
  ///
  /// Returns a parsing error for malformed XML, a DTD these options refuse, or nesting past the bound.
  pub(crate) fn parse(self, input: &str) -> XrfResult<roxmltree::Document<'_>> {
    XmlNesting::assert_within(input, Self::MAX_DEPTH)?;

    roxmltree::Document::parse_with_options(
      input,
      roxmltree::ParsingOptions {
        allow_dtd: self.allow_dtd,
        ..roxmltree::ParsingOptions::default()
      },
    )
    .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse XML: {error}")))
  }
}
