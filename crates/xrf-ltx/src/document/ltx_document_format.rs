use crate::document::{LtxDocument, LtxItemKind, LtxStatementWriter};
use crate::syntax::LTX_LINE_SEPARATOR;

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
        LtxItemKind::Comment { text } => LtxStatementWriter::write_comment(&mut formatted, text),
        LtxItemKind::Include { path, comment } => {
          LtxStatementWriter::write_include(&mut formatted, path, comment.as_deref())
        }
        LtxItemKind::Section {
          comment,
          name,
          operation,
          parents,
        } => LtxStatementWriter::write_section_with_prefix(
          &mut formatted,
          operation.as_prefix(),
          name,
          parents,
          comment.as_deref(),
        ),
        LtxItemKind::Key {
          comment,
          name,
          operation,
          value,
        } => LtxStatementWriter::write_key_value(
          &mut formatted,
          &format!("{}{name}", operation.as_prefix()),
          value.as_deref(),
          comment.as_deref(),
        ),
      }
    }

    if !formatted.ends_with(LTX_LINE_SEPARATOR) {
      formatted.push_str(LTX_LINE_SEPARATOR);
    }

    formatted
  }
}
