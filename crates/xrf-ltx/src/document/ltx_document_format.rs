use crate::LineSeparator;
use crate::document::ltx_document::LtxDocument;
use crate::document::ltx_item::LtxItemKind;
use crate::file::formatter::LtxFormatter;

impl LtxDocument {
  /// Render this document in canonical form.
  ///
  /// Reads the same parse everything else does, so formatting no longer scans the file a second time. Authored
  /// whitespace is normalized: blank runs are dropped and one blank line is placed before every section header but the
  /// first, which is what the previous formatting pass produced.
  pub fn to_formatted(&self) -> String {
    let mut formatted: String = String::new();

    for item in &self.items {
      match &item.kind {
        LtxItemKind::Comment { text } => LtxFormatter::write_comment(&mut formatted, text),
        LtxItemKind::Include { path, comment } => LtxFormatter::write_include(&mut formatted, path, comment.as_deref()),
        LtxItemKind::Section {
          comment,
          name,
          operation,
          parents,
        } => LtxFormatter::write_section_with_prefix(
          &mut formatted,
          operation.as_prefix(),
          name,
          Some(parents.clone()),
          comment.as_deref(),
        ),
        LtxItemKind::Key {
          comment,
          name,
          operation,
          value,
        } => LtxFormatter::write_key_value(
          &mut formatted,
          &format!("{}{name}", operation.as_prefix()),
          value.as_deref(),
          comment.as_deref(),
        ),
      }
    }

    if !formatted.ends_with(LineSeparator::CRLF.as_str()) {
      formatted.push_str(LineSeparator::CRLF.as_str());
    }

    formatted
  }
}
