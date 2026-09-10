use xrf_ltx::{Ltx, LtxDocument};

use crate::text::LtxFileText;

/// Reads one config's authored text as the lines every other record anchors to.
///
/// Public and standalone, because lines need neither a project nor a resolution: a viewer showing a file it cannot
/// place still has something to show.
pub struct LtxTextReader {}

impl LtxTextReader {
  /// One config's authored text, laid out by the same spans every other record anchors to.
  ///
  /// Routed through the parser rather than split straight off the string, because the lines a viewer shows and the lines
  /// a finding marks have to be the same numbering. A statement the parser folds differently from a naive split would
  /// otherwise put a gutter mark one row away from the text it is about, and nothing downstream could tell.
  ///
  /// Infallible on purpose: a config that will not parse still opens, showing its text with the parse error marked, which
  /// is the only way a person can see what to fix. That case answers the raw split and says so through
  /// [`LtxFileText::is_normalized`].
  pub fn read(path: &str, contents: &str) -> LtxFileText {
    match Ltx::read_document_from_str_preserving_source(contents) {
      Ok(document) => LtxFileText {
        is_normalized: true,
        lines: Self::to_lines(&document, contents),
        path: String::from(path),
      },
      Err(_) => LtxFileText {
        is_normalized: false,
        lines: Self::split_lines(contents),
        path: String::from(path),
      },
    }
  }

  /// How many lines a file shows, which is the count an editor shows: one more than the newlines it holds.
  ///
  /// [`str::lines`] answers one fewer for the usual file. It reads a final newline as the terminator of the last line
  /// rather than as the start of the empty one after it, so a viewer agreeing with it draws a file a line shorter than
  /// the one on disk - and an editor writing those lines back would drop the newline the file ended with.
  ///
  /// An empty file has no lines at all rather than one empty one, so a viewer can say the file is empty instead of
  /// drawing a blank row and leaving the reader to guess.
  fn count_lines(contents: &str) -> usize {
    if contents.is_empty() {
      0
    } else {
      contents.bytes().filter(|byte| *byte == b'\n').count() + 1
    }
  }

  /// The lines of a file nothing could parse, split off the text itself.
  ///
  /// Split rather than [`str::lines`] for the reason above, which leaves this to strip the carriage returns of a CRLF
  /// file itself.
  fn split_lines(contents: &str) -> Vec<String> {
    if contents.is_empty() {
      return Vec::new();
    }

    contents
      .split('\n')
      .map(|line| String::from(line.strip_suffix('\r').unwrap_or(line)))
      .collect()
  }

  /// Places every statement at the line it was written on, leaving the gaps between them empty.
  ///
  /// The document holds no blank-line statement - a gap between consecutive spans is the blank run - so the lines are
  /// rebuilt by position rather than by iteration order.
  fn to_lines(document: &LtxDocument, contents: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::with_capacity(Self::count_lines(contents));

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
    lines.resize(lines.len().max(Self::count_lines(contents)), String::new());

    lines
  }
}
