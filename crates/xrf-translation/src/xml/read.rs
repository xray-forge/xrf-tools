use std::path::Path;

use xrf_error::XrfResult;
use xrf_xml::{XmlElementSpan, XmlParseOptions, XmlSourceDocument};

use crate::xml::encoding::{DecodedTranslation, TranslationIdentity, decode, read_decoded};

/// Read one string table off disk into its entries.
///
/// For a caller holding one file and no mounted roots; everything reading a tree goes through the VFS
/// and calls [`parse_string_table`] with the bytes it already has.
///
/// # Errors
///
/// Returns an IO error when the file cannot be read, an encoding error when its bytes do not decode,
/// and a parsing error when the document is malformed beyond what the engine accepts.
pub(crate) fn read_string_table(path: &Path) -> XrfResult<Vec<(String, String)>> {
  parse_decoded(read_decoded(path)?)
}

/// Parse a string table already in hand, such as an entry read out of an archive.
///
/// The identity is what says which language the bytes are written in, and is the whole reason this
/// takes two arguments: a `\`-separated logical path is not a host path, so the names are supplied
/// rather than derived here.
///
/// # Errors
///
/// Returns an encoding error when the bytes do not decode, and a parsing error when the document is
/// malformed beyond what the engine accepts.
pub(crate) fn parse_string_table(identity: TranslationIdentity, data: &[u8]) -> XrfResult<Vec<(String, String)>> {
  parse_decoded(decode(identity, data)?)
}

/// Pull the entries out of a decoded string table, tolerating whatever the engine tolerates.
fn parse_decoded(decoded: DecodedTranslation) -> XrfResult<Vec<(String, String)>> {
  let document: XmlSourceDocument = XmlSourceDocument::parse(decoded.text, XmlParseOptions::default())?;

  Ok(
    document
      .root()
      .children_named("string")
      .filter_map(|element| {
        let id: &str = element.attribute("id")?;
        // An entry with no text element is skipped by the engine too, with a message.
        let text: &XmlElementSpan = element.child_named("text")?;

        Some((id.to_owned(), text.text().to_owned()))
      })
      .collect(),
  )
}
