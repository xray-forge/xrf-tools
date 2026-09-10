//! What the authored view renders, and what it admits it lost.

use crate::text::{LtxFileText, LtxTextReader};

#[test]
fn every_statement_lands_on_the_line_it_was_written_on() {
  let text: LtxFileText = LtxTextReader::read("system.ltx", "; header\n\n[wpn_base]\ncost = 100\n");

  assert_eq!(text.path, "system.ltx");
  assert_eq!(
    text.lines,
    vec![
      String::from("; header"),
      String::new(),
      String::from("[wpn_base]"),
      String::from("cost = 100"),
      String::new(),
    ]
  );
  assert!(text.is_normalized);
}

#[test]
fn a_line_holding_only_whitespace_comes_back_empty() {
  let text: LtxFileText = LtxTextReader::read("system.ltx", "[wpn_base]\n   \ncost = 100\n");

  assert_eq!(text.lines[1], "", "the one thing the document cannot carry");
  assert!(text.is_normalized, "and the record says so");
}

#[test]
fn a_config_that_will_not_parse_answers_its_raw_lines() {
  let text: LtxFileText = LtxTextReader::read("broken.ltx", "[wpn_base]\ncost = 100\n[unclosed\n");

  assert_eq!(
    text.lines,
    vec![
      String::from("[wpn_base]"),
      String::from("cost = 100"),
      String::from("[unclosed"),
      String::new(),
    ]
  );
  assert!(!text.is_normalized, "nothing went through the parser");
}

#[test]
fn the_newline_a_file_ends_with_starts_a_line_of_its_own() {
  // What every editor shows and what `str::lines` does not: the final newline opens the empty line after it. A viewer
  // that dropped it would draw a file a line shorter than the one on disk, and an editor writing these lines back
  // would drop the newline the file ended with.
  let ended: LtxFileText = LtxTextReader::read(
    "system.ltx",
    "[wpn_base]
cost = 100
",
  );
  let unended: LtxFileText = LtxTextReader::read(
    "system.ltx",
    "[wpn_base]
cost = 100",
  );

  assert_eq!(
    ended.lines,
    vec![String::from("[wpn_base]"), String::from("cost = 100"), String::new()]
  );
  assert_eq!(
    unended.lines,
    vec![String::from("[wpn_base]"), String::from("cost = 100")]
  );
}

#[test]
fn the_lines_join_back_into_the_file_they_came_from() {
  // The contract an editor will write through. It holds for text the parser can carry exactly, which is why the one
  // thing it cannot - a line of only whitespace - is recorded in `is_normalized`.
  for contents in [
    "[a]
b = 1
",
    "[a]
b = 1",
    "; only a comment
",
    "


",
    "[a]

[b]
",
  ] {
    let text: LtxFileText = LtxTextReader::read("system.ltx", contents);

    assert_eq!(
      text.lines.join(
        "
"
      ),
      contents,
      "{contents:?} came back as {:?}",
      text.lines
    );
  }
}

#[test]
fn an_empty_file_has_no_lines_at_all() {
  // Not one empty line: a viewer with nothing to draw should say the file is empty rather than draw a blank row.
  let text: LtxFileText = LtxTextReader::read("empty.ltx", "");

  assert!(text.lines.is_empty());
  assert!(text.is_normalized, "there was nothing to fail on");
}
