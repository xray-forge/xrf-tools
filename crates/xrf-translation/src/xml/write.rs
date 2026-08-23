use std::cmp::Reverse;
use std::ops::Range;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, encode_string_to_bytes};
use xrf_xml::{XmlElementSpan, XmlParseOptions, XmlSourceDocument, escape_xml_text};

use crate::edit::TranslationEdit;
use crate::language::find_unencodable_character;
use crate::staged_write::write_file_staged;
use crate::xml::encoding::{DecodedTranslation, read_decoded};
use crate::xml::layout::{XmlLayout, removal_range};

/// Apply edits to one string table file, leaving every untouched byte as it was found.
///
/// The file is re-read here rather than edited against a model held elsewhere: spans are only valid
/// for the exact text they were parsed from, so re-reading is what keeps a save correct when the file
/// moved on after it was opened.
///
/// # Errors
///
/// Returns an encoding error when a value cannot be represented in the file's encoding, a parsing
/// error when the file is not a well-formed string table, and an IO error when it cannot be read or
/// replaced.
pub fn apply_edits(path: &Path, edits: &[TranslationEdit]) -> XrfResult {
  if edits.is_empty() {
    return Ok(());
  }

  let decoded: DecodedTranslation = read_decoded(path)?;
  let encoding: XRayEncoding = decoded.encoding;
  let edited: String = splice_edits(path, decoded.text, edits, encoding)?;

  let mut written: Vec<u8> = decoded.byte_order_mark;

  written.extend(encode_string_to_bytes(&edited, encoding)?);

  write_file_staged(path, &written)
}

/// Takes the source by value because the ranges it produces only address that exact string, and the
/// document it is parsed into now owns the pairing rather than leaving it to a caller to honour.
pub(crate) fn splice_edits(
  path: &Path,
  source: String,
  edits: &[TranslationEdit],
  encoding: XRayEncoding,
) -> XrfResult<String> {
  let document: XmlSourceDocument = XmlSourceDocument::parse(source, XmlParseOptions::default())?;
  let source: &str = document.source();
  let root: &XmlElementSpan = document.root();
  let strings: Vec<&XmlElementSpan> = root.children_named("string").collect();
  let layout: XmlLayout = XmlLayout::detect(source, &strings);

  let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
  let mut appended: String = String::new();

  for edit in edits {
    match edit {
      TranslationEdit::Set { id, value } => {
        // A string table holds one line per entry, so a multi-line value renders to the literal `\n`
        // the engine reads as a break. The array form only survives in a JSON source.
        let text: String = value.to_single_line();

        validate_encodable(path, encoding, id, "id", id)?;
        validate_encodable(path, encoding, id, "text", &text)?;

        // Last occurrence wins, because that is the one the engine's string table resolves to.
        match occurrences(&strings, id).last() {
          Some(element) => replacements.push((text_content_range(path, element, id)?, escape_xml_text(&text))),
          None => appended.push_str(&layout.render_entry(id, &text)),
        }
      }
      TranslationEdit::Remove { id } => {
        // Every occurrence, not only the winner: leaving a shadowed duplicate behind would quietly
        // promote an older value to being the one the engine uses.
        for element in occurrences(&strings, id) {
          replacements.push((removal_range(source, element.element_range()), String::new()));
        }
      }
    }
  }

  if !appended.is_empty() {
    let at: usize = layout.insertion_offset(root, &strings);

    replacements.push((at..at, appended));
  }

  // Highest offset first, so applying one replacement never shifts the ranges still to come.
  replacements.sort_by_key(|(range, _)| Reverse(range.start));

  for pair in replacements.windows(2) {
    if pair[1].0.end > pair[0].0.start {
      return Err(XrfError::new_invalid_error(format!(
        "Translation edits for '{}' overlap in the same region",
        path.display()
      )));
    }
  }

  let mut edited: String = source.to_owned();

  for (range, value) in replacements {
    edited.replace_range(range, &value);
  }

  Ok(edited)
}

fn occurrences<'a>(strings: &[&'a XmlElementSpan], id: &str) -> Vec<&'a XmlElementSpan> {
  strings
    .iter()
    .filter(|element| element.attribute("id") == Some(id))
    .copied()
    .collect()
}

fn text_content_range(path: &Path, element: &XmlElementSpan, id: &str) -> XrfResult<Range<usize>> {
  element
    .child_named("text")
    .and_then(|text| text.content_range().cloned())
    .ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "Translation '{}' entry '{id}' has no text element to edit",
        path.display()
      ))
    })
}

fn validate_encodable(path: &Path, encoding: XRayEncoding, id: &str, field: &str, value: &str) -> XrfResult {
  if let Some(character) = find_unencodable_character(value, encoding) {
    return Err(XrfError::new_encoding_error(format!(
      "Translation '{}' entry '{id}' {field} cannot be encoded as {}: '{character}' (U+{:04X})",
      path.display(),
      encoding.name(),
      character as u32,
    )));
  }

  Ok(())
}
