//! What the authored view renders, and what it admits it lost.

use crate::text::{LtxFileText, read_text};

#[test]
fn every_statement_lands_on_the_line_it_was_written_on() {
  let text: LtxFileText = read_text("system.ltx", "; header\n\n[wpn_base]\ncost = 100\n");

  assert_eq!(text.path, "system.ltx");
  assert_eq!(
    text.lines,
    vec![
      String::from("; header"),
      String::new(),
      String::from("[wpn_base]"),
      String::from("cost = 100"),
    ]
  );
  assert!(text.is_normalized);
}

#[test]
fn a_line_holding_only_whitespace_comes_back_empty() {
  let text: LtxFileText = read_text("system.ltx", "[wpn_base]\n   \ncost = 100\n");

  assert_eq!(text.lines[1], "", "the one thing the document cannot carry");
  assert!(text.is_normalized, "and the record says so");
}

#[test]
fn a_config_that_will_not_parse_answers_its_raw_lines() {
  let text: LtxFileText = read_text("broken.ltx", "[wpn_base]\ncost = 100\n[unclosed\n");

  assert_eq!(
    text.lines,
    vec![
      String::from("[wpn_base]"),
      String::from("cost = 100"),
      String::from("[unclosed"),
    ]
  );
  assert!(!text.is_normalized, "nothing went through the parser");
}
