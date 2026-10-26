use std::cmp::Reverse;
use std::collections::HashSet;
use std::fs;
use std::ops::Range;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, XRayEncoding, format_path};
use xrf_xml::{XmlElementSpan, XmlEscape, XmlSourceDocument};

use crate::language::translation_code_page::TranslationCodePage;
use crate::text::translation_edit::TranslationEdit;
use crate::xml::decoded_translation::DecodedTranslation;
use crate::xml::translation_identity::TranslationIdentity;
use crate::xml::translation_line_limit::TranslationLineLimit;
use crate::xml::translation_xml_reader::TranslationXmlReader;
use crate::xml::xml_layout::XmlLayout;

/// Applies edits to one string table, leaving every untouched byte as it was found.
pub(crate) struct TranslationXmlWriter;

impl TranslationXmlWriter {
  /// Apply edits to one string table file on disk.
  ///
  /// The file is re-read here rather than edited against a model held elsewhere: spans are only valid for the exact
  /// text they were parsed from, so re-reading is what keeps a save correct when the file moved on after it was opened.
  ///
  /// # Errors
  ///
  /// Whatever [`Self::render_edits`] returns, and an IO error when the file cannot be read or replaced.
  pub(crate) fn apply_edits(path: &Path, edits: &[TranslationEdit]) -> XrfResult {
    if edits.is_empty() {
      return Ok(());
    }

    let existing: Vec<u8> = fs::read(path)?;

    Ok(StagedFile::write(path, &Self::render_edits(path, &existing, edits)?)?)
  }

  /// The bytes a string table holding `existing` holds once `edits` are applied, every untouched byte as it was.
  ///
  /// `path` names the file, by its host path or its logical one, for the language its name implies and for refusals.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when a value cannot be represented in the file's encoding, a parsing error when
  /// `existing` is not a well-formed string table, and an invalid error when the edit leaves a line the game's XML
  /// reader refuses.
  pub(crate) fn render_edits(path: &Path, existing: &[u8], edits: &[TranslationEdit]) -> XrfResult<Vec<u8>> {
    let decoded: DecodedTranslation = DecodedTranslation::decode(TranslationIdentity::from_path(path), existing)?;
    let edited: String = Self::splice(path, decoded.text.clone(), edits, decoded.encoding)?;
    let written: Vec<u8> = decoded.encode(&edited)?;

    Self::ensure_loadable(path, existing, &written)?;

    Ok(written)
  }

  /// Takes the source by value because the ranges it produces only address that exact string, and the document it is
  /// parsed into now owns the pairing rather than leaving it to a caller to honour.
  pub(crate) fn splice(
    path: &Path,
    source: String,
    edits: &[TranslationEdit],
    encoding: XRayEncoding,
  ) -> XrfResult<String> {
    let document: XmlSourceDocument = XmlSourceDocument::parse(source)?;
    let source: &str = document.source();
    let root: &XmlElementSpan = document.root();
    let strings: Vec<&XmlElementSpan> = root.children_named(TranslationXmlReader::STRING_ELEMENT).collect();
    let layout: XmlLayout = XmlLayout::detect(source, &strings);

    let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
    let mut appended: String = String::new();

    for edit in edits {
      match edit {
        TranslationEdit::Set { id, value } => {
          // A string table holds one line per entry, so a multi-line value renders to the literal `\n` the engine reads
          // as a break. The array form only survives in a JSON source.
          let text: String = value.to_single_line();

          Self::validate_encodable(path, encoding, id, "id", id)?;
          Self::validate_encodable(path, encoding, id, "text", &text)?;

          // Last occurrence wins, because that is the one the engine's string table resolves to.
          match Self::occurrences(&strings, id).last() {
            Some(element) => replacements.push((Self::text_content_range(path, element, id)?, XmlEscape::text(&text))),
            None => appended.push_str(&layout.render_entry(id, &text)),
          }
        }
        TranslationEdit::Remove { id } => {
          // Every occurrence, not only the winner: leaving a shadowed duplicate behind would quietly promote an older
          // value to being the one the engine uses.
          for element in Self::occurrences(&strings, id) {
            replacements.push((XmlLayout::removal_range(source, element.element_range()), String::new()));
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
          format_path(path)
        )));
      }
    }

    let mut edited: String = source.to_owned();

    for (range, value) in replacements {
      edited.replace_range(range, &value);
    }

    Ok(edited)
  }

  /// Refuse an edit that leaves a line the game's XML reader refuses, which a line the file already held is not: a
  /// table the game never loaded is not made worse by a save that does not touch its long line.
  fn ensure_loadable(path: &Path, existing: &[u8], written: &[u8]) -> XrfResult {
    let before: HashSet<&[u8]> = TranslationLineLimit::find_overlong(existing)
      .into_iter()
      .map(|(_, line)| line)
      .collect();

    match TranslationLineLimit::find_overlong(written)
      .into_iter()
      .find(|(_, line)| !before.contains(line))
    {
      Some((number, line)) => Err(XrfError::new_invalid_error(format!(
        "Translation '{}' would not load: {}",
        format_path(path),
        TranslationLineLimit::describe(number, line.len())
      ))),
      None => Ok(()),
    }
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
      .child_named(TranslationXmlReader::TEXT_ELEMENT)
      .and_then(|text| text.content_range().cloned())
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Translation '{}' entry '{id}' has no text element to edit",
          format_path(path)
        ))
      })
  }

  fn validate_encodable(path: &Path, encoding: XRayEncoding, id: &str, field: &str, value: &str) -> XrfResult {
    if let Some(character) = TranslationCodePage::find_unencodable(value, encoding) {
      return Err(XrfError::new_encoding_error(format!(
        "Translation '{}' entry '{id}' {field} cannot be encoded as {}: {}",
        format_path(path),
        encoding.name(),
        TranslationCodePage::describe(character),
      )));
    }

    Ok(())
  }
}
