use xrf_error::XrfResult;

use crate::document::{LtxDocument, LtxItemKind, LtxStatementWriter};
use crate::syntax::{LTX_LINE_SEPARATOR, leaves_quote_open, new_open_quote_error};

impl LtxDocument {
  /// Render this document in canonical form.
  ///
  /// Reads the same parse everything else does, so formatting no longer scans the file a second time. Authored
  /// whitespace is normalized: blank runs are dropped and one blank line is placed before every section header but the
  /// first, which is what the previous formatting pass produced.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section and key of a value in a section that leaves a quote open, which the parser
  /// refuses, so a document built by hand cannot be formatted into a file that does not read back.
  pub fn to_formatted(&self) -> XrfResult<String> {
    let mut formatted: String = String::new();
    // The section the fields belong to; none before the first header, where an open quote reads as one line.
    let mut section: Option<&str> = None;

    for item in &self.items {
      match &item.kind {
        LtxItemKind::Comment { text } => LtxStatementWriter::write_comment(&mut formatted, text),
        LtxItemKind::Include { path, comment } => {
          LtxStatementWriter::write_include(&mut formatted, path, comment.as_deref());
        }
        LtxItemKind::Section {
          comment,
          name,
          operation,
          parents,
        } => {
          section = Some(name);
          LtxStatementWriter::write_section_with_prefix(
            &mut formatted,
            operation.as_prefix(),
            name,
            parents,
            comment.as_deref(),
          );
        }
        LtxItemKind::Key {
          comment,
          name,
          operation,
          value,
        } => {
          if let (Some(section), Some(value)) = (section, value.as_deref())
            && leaves_quote_open(value)
          {
            return Err(new_open_quote_error(section, name));
          }

          LtxStatementWriter::write_key_value(
            &mut formatted,
            &format!("{}{name}", operation.as_prefix()),
            value.as_deref(),
            comment.as_deref(),
          );
        }
      }
    }

    if !formatted.ends_with(LTX_LINE_SEPARATOR) {
      formatted.push_str(LTX_LINE_SEPARATOR);
    }

    Ok(formatted)
  }
}
