use std::path::Path;

use xrf_error::XrfResult;
use xrf_xml::{XmlElementSpan, XmlParseOptions, XmlSourceDocument};

use crate::xml::encoding::{DecodedTranslation, read_decoded};

/// Read one string table into its entries, tolerating whatever the engine tolerates.
///
/// # Errors
///
/// Returns an encoding error when the bytes do not decode, and a parsing error when the document is
/// malformed beyond what the engine accepts.
pub(crate) fn read_string_table(path: &Path) -> XrfResult<Vec<(String, String)>> {
  let decoded: DecodedTranslation = read_decoded(path)?;
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
