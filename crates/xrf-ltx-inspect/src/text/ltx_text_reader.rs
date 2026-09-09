use xrf_ltx::{Ltx, LtxDocument};

use crate::text::LtxFileText;

/// One config's authored text, laid out by the same spans every other record anchors to.
///
/// Routed through the parser rather than split straight off the string, because the lines a viewer shows and the lines
/// a finding marks have to be the same numbering. A statement the parser folds differently from a naive split would
/// otherwise put a gutter mark one row away from the text it is about, and nothing downstream could tell.
///
/// Infallible on purpose: a config that will not parse still opens, showing its text with the parse error marked, which
/// is the only way a person can see what to fix. That case answers the raw split and says so through
/// [`LtxFileText::is_normalized`].
pub fn read_text(path: &str, contents: &str) -> LtxFileText {
  match Ltx::read_document_from_str_preserving_source(contents) {
    Ok(document) => LtxFileText {
      is_normalized: true,
      lines: to_lines(&document, contents),
      path: String::from(path),
    },
    Err(_) => LtxFileText {
      is_normalized: false,
      lines: contents.lines().map(String::from).collect(),
      path: String::from(path),
    },
  }
}

/// Places every statement at the line it was written on, leaving the gaps between them empty.
///
/// The document holds no blank-line statement - a gap between consecutive spans is the blank run - so the lines are
/// rebuilt by position rather than by iteration order.
fn to_lines(document: &LtxDocument, contents: &str) -> Vec<String> {
  let mut lines: Vec<String> = Vec::with_capacity(contents.lines().count());

  for (item, source) in document.get_items().iter().zip(document.get_source_lines()) {
    let line: usize = item.span.get_line();

    if line == 0 {
      continue;
    }

    if lines.len() < line {
      lines.resize(line, String::new());
    }

    lines[line - 1] = String::from(&**source);
  }

  // A file ending in blank lines has no statement to place there, and a viewer that dropped them would show a shorter
  // file than the one on disk.
  lines.resize(lines.len().max(contents.lines().count()), String::new());

  lines
}
