//! What the document records, and what standard lowering does with it.

use xrf_error::XrfResult;

use crate::dialect::LtxStandardDialect;
use crate::document::{LtxCheck, LtxDocument, LtxItem, LtxItemKind};
use crate::ltx::Ltx;

/// The kinds a document holds, rendered compactly so a test can assert the whole shape.
fn outline(document: &LtxDocument) -> Vec<String> {
  document
    .get_items()
    .iter()
    .map(|item| match &item.kind {
      LtxItemKind::Comment { text } => format!("{} comment {text}", item.span.line),
      LtxItemKind::Include { path, .. } => format!("{} include {path}", item.span.line),
      LtxItemKind::Section {
        name,
        operation,
        parents,
        ..
      } => format!(
        "{} section {}{name}{}",
        item.span.line,
        operation.as_prefix(),
        if parents.is_empty() {
          String::new()
        } else {
          format!(":{}", parents.join(","))
        }
      ),
      LtxItemKind::Key {
        name, operation, value, ..
      } => format!(
        "{} key {}{name}={}",
        item.span.line,
        operation.as_prefix(),
        value.as_deref().unwrap_or("<none>")
      ),
    })
    .collect()
}

#[test]
fn records_every_statement_with_the_line_it_was_written_on() -> XrfResult {
  let document: LtxDocument = Ltx::read_document_from_str(
    "; leading note\n#include \"other.ltx\"\nroot_key = 1\n\n[section]:parent\nkey = value ; trailing\nbare\n",
  )?;

  assert_eq!(
    outline(&document),
    vec![
      "1 comment leading note",
      "2 include other.ltx",
      "3 key root_key=1",
      "5 section section:parent",
      "6 key key=value",
      "7 key bare=<none>",
    ]
  );

  Ok(())
}

#[test]
fn a_gap_between_spans_is_how_a_blank_run_is_recorded() -> XrfResult {
  let document: LtxDocument = Ltx::read_document_from_str("[first]\n\n\n\n[second]\n")?;
  let lines: Vec<usize> = document.get_items().iter().map(|item| item.span.line).collect();

  // No blank-line statement kind exists; three blank lines are the distance from line 1 to line 5.
  assert_eq!(lines, vec![1, 5]);

  Ok(())
}

#[test]
fn the_document_accepts_dltx_syntax_that_standard_lowering_refuses() -> XrfResult {
  let patch: &str =
    "![wpn_ak74]\n!cost\n>ammo_class = ammo_new\n<ammo_class = ammo_old\n\n!![wpn_dropped]\n\n@[wpn_new]\ncost = 1\n";
  let document: LtxDocument = Ltx::read_document_from_str(patch)?;

  // Parsing is permissive: a prefix becomes an operation rather than part of a name.
  assert_eq!(
    outline(&document),
    vec![
      "1 section !wpn_ak74",
      "2 key !cost=<none>",
      "3 key >ammo_class=ammo_new",
      "4 key <ammo_class=ammo_old",
      "6 section !!wpn_dropped",
      "8 section @wpn_new",
      "9 key cost=1",
    ]
  );

  // Refusing them is the dialect's job, and the diagnostic names the flag that would evaluate them.
  let error: String = LtxStandardDialect::lower(&document)
    .expect_err("standard lowering to refuse a patch file")
    .to_string();

  assert!(error.contains("needs the dltx dialect"), "{error}");
  assert!(error.contains("--dltx"), "{error}");
  assert!(error.contains("![wpn_ak74]"), "{error}");

  Ok(())
}

#[test]
fn every_patch_operation_is_reported_where_it_was_written() -> XrfResult {
  for (statement, expected) in [
    ("!cost = 1\n", "field '!cost'"),
    (">ammo = a\n", "field '>ammo'"),
    ("<ammo = a\n", "field '<ammo'"),
    ("![section]\n", "section '![section]'"),
    ("@[section]\n", "section '@[section]'"),
    ("!![section]\n", "section '!![section]'"),
  ] {
    let error: String = LtxStandardDialect::lower(&Ltx::read_document_from_str(statement)?)
      .expect_err("standard lowering to refuse a patch operation")
      .to_string();

    assert!(error.contains(expected), "{statement} gave {error}");
    assert!(error.contains("1:1"), "{statement} gave {error}");
  }

  Ok(())
}

#[test]
fn a_plain_section_named_like_a_patch_target_is_not_an_operation() -> XrfResult {
  // Only a leading prefix is an operation. A `!` anywhere else is an ordinary character, which condlist values rely on.
  let document: LtxDocument = Ltx::read_document_from_str("[section]\ncondition = {!actor_in_zone} true\n")?;

  assert!(!document.get_items().iter().any(LtxItem::is_patch_operation));
  assert_eq!(
    LtxStandardDialect::lower(&document)?.get_from("section", "condition"),
    Some("{!actor_in_zone} true")
  );

  Ok(())
}

#[test]
fn one_parse_answers_resolution_formatting_and_the_include_list() -> XrfResult {
  let contents: &str = "; note\n#include \"a.ltx\"\n#include \"b.ltx\"\n\n[section]\nkey=value\n";
  let document: LtxDocument = Ltx::read_document_from_str(contents)?;

  assert_eq!(document.list_included(), vec!["a.ltx", "b.ltx"]);
  assert_eq!(
    document.to_formatted(),
    "; note\r\n#include \"a.ltx\"\r\n#include \"b.ltx\"\r\n\r\n[section]\r\nkey = value\r\n"
  );
  assert_eq!(
    LtxStandardDialect::lower(&document)?.get_from("section", "key"),
    Some("value")
  );

  // Each of the three is a read of the same document, so the answers cannot disagree.
  assert_eq!(document.to_formatted(), Ltx::format_from_str(contents)?);

  Ok(())
}

#[test]
fn formatting_a_patch_file_keeps_its_prefixes_outside_the_brackets() -> XrfResult {
  let document: LtxDocument = Ltx::read_document_from_str("![wpn_ak74]:base\n!cost\n>ammo_class=a,b\n")?;

  // The formatter needs no dialect: an operation is another statement to render, and `![x]` must not become `[!x]`.
  assert_eq!(
    document.to_formatted(),
    "![wpn_ak74]:base\r\n!cost\r\n>ammo_class = a,b\r\n"
  );

  Ok(())
}

#[test]
fn a_document_carries_no_line_text_unless_it_is_asked_to() -> XrfResult {
  let contents: &str = "  [section]\n\tkey   =   value\n";

  assert!(
    Ltx::read_document_from_str(contents)?
      .get_items()
      .iter()
      .all(|item| item.source.is_none())
  );

  let preserved: LtxDocument = Ltx::read_document_from_str_preserving_source(contents)?;

  // Indentation included, so an untouched line can be written back exactly as authored.
  assert_eq!(
    preserved
      .get_items()
      .iter()
      .map(|item| item.source.as_deref().expect("a preserved line"))
      .collect::<Vec<&str>>(),
    vec!["  [section]", "\tkey   =   value"]
  );

  Ok(())
}

#[test]
fn preserved_lines_rebuild_the_file_they_came_from() -> XrfResult {
  let contents: &str = "; note\n#include \"a.ltx\"\n\n[section]:parent ; header note\n  padded   =   value  \nbare\n";
  let preserved: LtxDocument = Ltx::read_document_from_str_preserving_source(contents)?;

  // Rebuilt from the spans, which say where each line sat, and the preserved text, which says what it held. Blank runs
  // come back as empty lines; a blank line that held whitespace is the one thing normalized away.
  let mut rebuilt: String = String::new();
  let mut next_line: usize = 1;

  for item in preserved.get_items() {
    while next_line < item.span.line {
      rebuilt.push('\n');
      next_line += 1;
    }

    rebuilt.push_str(item.source.as_deref().expect("a preserved line"));
    rebuilt.push('\n');
    next_line += 1;
  }

  assert_eq!(rebuilt, contents);

  Ok(())
}

#[test]
fn listing_includes_keeps_a_repeat_that_resolution_refuses() -> XrfResult {
  let contents: &str = "#include \"a.ltx\"
#include \"a.ltx\"
";

  // Scanning is not resolving. Project assembly reads include lists to find entry points, and a repeat is not its
  // business; the file is still refused the moment anything resolves it.
  assert_eq!(Ltx::read_included_from_str(contents)?, vec!["a.ltx", "a.ltx"]);
  assert!(Ltx::read_from_str(contents).is_err());

  Ok(())
}

#[test]
fn the_header_directive_is_recorded_on_the_document() -> XrfResult {
  let document: LtxDocument = Ltx::read_document_from_str("; @xrf-ltx skip-inheritance\n[child]:missing\n")?;

  assert!(document.is_check_skipped(LtxCheck::Inheritance));

  // And survives lowering, because the resolver is what acts on it.
  assert!(LtxStandardDialect::lower(&document)?.into_inherited().is_ok());

  Ok(())
}
