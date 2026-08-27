use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_xml::{XmlElementSpan, XmlParseOptions, XmlSourceDocument};

use crate::xml::encoding::{DecodedTranslation, TranslationIdentity, decode, read_decoded};

/// The root element every string table has, and the two elements an entry is made of.
const STRING_TABLE_ELEMENT: &str = "string_table";
const STRING_ELEMENT: &str = "string";
const TEXT_ELEMENT: &str = "text";

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

/// Parse a string table, refusing a document that is not one.
///
/// The tolerant readers answer an empty list for any well-formed XML, because a root element they do
/// not recognise simply has no `string` children. That is right for a tree walk, which wants one bad
/// file to cost its own strings — but an importer writing a JSON file per input needs to tell "this
/// table is empty" from "this is not a table", or it writes `{}` for every unrelated XML that happens
/// to sit in a language directory, OpenXRay's own marker file included.
///
/// # Errors
///
/// Returns an encoding error when the bytes do not decode, a parsing error when the document is
/// malformed, and an invalid error when its root is not `string_table`.
pub(crate) fn parse_required_string_table(
  identity: TranslationIdentity,
  data: &[u8],
) -> XrfResult<Vec<(String, String)>> {
  let decoded: DecodedTranslation = decode(identity, data)?;
  let document: XmlSourceDocument = XmlSourceDocument::parse(decoded.text, XmlParseOptions::default())?;
  let root: &XmlElementSpan = document.root();

  if root.name() != STRING_TABLE_ELEMENT {
    return Err(XrfError::new_invalid_error(format!(
      "Expected a '{STRING_TABLE_ELEMENT}' document, found '{}'",
      root.name()
    )));
  }

  Ok(collect_entries(root))
}

/// Pull the entries out of a decoded string table, tolerating whatever the engine tolerates.
fn parse_decoded(decoded: DecodedTranslation) -> XrfResult<Vec<(String, String)>> {
  let document: XmlSourceDocument = XmlSourceDocument::parse(decoded.text, XmlParseOptions::default())?;

  Ok(collect_entries(document.root()))
}

fn collect_entries(root: &XmlElementSpan) -> Vec<(String, String)> {
  root
    .children_named(STRING_ELEMENT)
    .filter_map(|element| {
      let id: &str = element.attribute("id")?;
      // An entry with no text element is skipped by the engine too, with a message.
      let text: &XmlElementSpan = element.child_named(TEXT_ELEMENT)?;

      Some((id.to_owned(), text.text().to_owned()))
    })
    .collect()
}
