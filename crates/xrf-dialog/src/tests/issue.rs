use xrf_error::XrfResult;
use xrf_utils::new_windows1251_encoder;

use crate::file::DialogFile;
use crate::issue::{DialogParseIssue, DialogParseIssueKind};

fn parse(source: &str) -> XrfResult<DialogFile> {
  DialogFile::parse(String::from(source), new_windows1251_encoder())
}

fn kinds(file: &DialogFile) -> Vec<DialogParseIssueKind> {
  file.get_issues().iter().map(DialogParseIssue::get_kind).collect()
}

#[test]
fn keeps_an_unknown_phrase_element_and_reports_it() -> XrfResult {
  let file: DialogFile = parse(
    r#"<game_dialogs><dialog id="d"><phrase_list><phrase id="0"><go_back>1</go_back></phrase></phrase_list></dialog></game_dialogs>"#,
  )?;

  // Preserved: an editor that dropped what it does not know would silently delete a mod's own data.
  let elements = file.get_dialogs()[0].get_phrases()[0].get_elements();

  assert_eq!(elements.len(), 1);
  assert_eq!(elements[0].get_name(), "go_back");

  // And reported, so validation can decide whether it matters.
  assert_eq!(kinds(&file), vec![DialogParseIssueKind::UnknownElement]);
  assert_eq!(file.get_issues()[0].get_subject(), "go_back");
  assert_eq!(file.get_issues()[0].get_dialog_id(), Some("d"));
  assert_eq!(file.get_issues()[0].get_phrase_id(), Some("0"));

  Ok(())
}

#[test]
fn reports_a_phrase_element_used_at_dialog_level() -> XrfResult {
  let file: DialogFile = parse(r#"<game_dialogs><dialog id="d"><next>1</next></dialog></game_dialogs>"#)?;

  assert_eq!(kinds(&file), vec![DialogParseIssueKind::UnknownElement]);
  assert_eq!(file.get_issues()[0].get_subject(), "next");
  assert_eq!(file.get_issues()[0].get_phrase_id(), None);

  Ok(())
}

#[test]
fn accepts_action_at_both_levels() -> XrfResult {
  // Two shipped dialogs carry a dialog-level `action`; treating it as phrase-only would report
  // every one of them.
  let file: DialogFile = parse(
    r#"<game_dialogs><dialog id="d"><action>dialogs.break_dialog</action><phrase_list><phrase id="0"><action>dialogs.break_dialog</action></phrase></phrase_list></dialog></game_dialogs>"#,
  )?;

  assert!(file.get_issues().is_empty());

  Ok(())
}

#[test]
fn reports_an_unknown_attribute() -> XrfResult {
  let file: DialogFile = parse(r#"<game_dialogs><dialog id="d" weight="3"/></game_dialogs>"#)?;

  assert_eq!(kinds(&file), vec![DialogParseIssueKind::UnknownAttribute]);
  assert_eq!(file.get_issues()[0].get_subject(), "weight");

  Ok(())
}

#[test]
fn skips_and_reports_a_dialog_with_no_id() -> XrfResult {
  let file: DialogFile = parse(r#"<game_dialogs><dialog/><dialog id="kept"/></game_dialogs>"#)?;

  assert_eq!(file.get_dialogs().len(), 1);
  assert_eq!(file.get_dialogs()[0].get_id(), "kept");
  assert_eq!(kinds(&file), vec![DialogParseIssueKind::MissingId]);

  Ok(())
}

#[test]
fn skips_and_reports_a_phrase_with_no_id() -> XrfResult {
  let file: DialogFile = parse(
    r#"<game_dialogs><dialog id="d"><phrase_list><phrase><text>orphan</text></phrase><phrase id="0"/></phrase_list></dialog></game_dialogs>"#,
  )?;

  assert_eq!(file.get_dialogs()[0].get_phrases().len(), 1);
  assert_eq!(kinds(&file), vec![DialogParseIssueKind::MissingId]);
  assert_eq!(file.get_issues()[0].get_dialog_id(), Some("d"));

  Ok(())
}

#[test]
fn keeps_a_dialog_whose_priority_is_not_an_integer() -> XrfResult {
  let file: DialogFile = parse(r#"<game_dialogs><dialog id="d" priority="high"/></game_dialogs>"#)?;

  assert_eq!(file.get_dialogs().len(), 1);
  assert_eq!(file.get_dialogs()[0].get_priority(), None);
  assert_eq!(kinds(&file), vec![DialogParseIssueKind::InvalidPriority]);
  assert_eq!(file.get_issues()[0].get_subject(), "high");

  Ok(())
}

#[test]
fn reports_an_unexpected_root_without_refusing_the_file() -> XrfResult {
  let file: DialogFile = parse(r#"<dialogs><dialog id="d"/></dialogs>"#)?;

  assert_eq!(file.get_dialogs().len(), 1);
  assert_eq!(kinds(&file), vec![DialogParseIssueKind::UnknownElement]);
  assert_eq!(file.get_issues()[0].get_subject(), "dialogs");

  Ok(())
}

#[test]
fn reads_a_file_whose_comments_xml_forbids() -> XrfResult {
  // A comment body may not contain `--`, and shipped banners are made of little else. This is the
  // bug that has kept the Ray of Hope editor from opening such files since 2022; the repair pass in
  // `xrf-xml` blanks the dashes byte for byte, so ranges still address the original text.
  let source: &str = r#"<game_dialogs>
    <!--- FINAL DIALOGS --->
    <!-- ================= -->
    <dialog id="after_banner"><phrase_list><phrase id="0"><text>key</text></phrase></phrase_list></dialog>
  </game_dialogs>"#;
  let file: DialogFile = parse(source)?;

  assert_eq!(file.get_dialogs().len(), 1);
  assert!(file.get_issues().is_empty());
  assert_eq!(file.get_source(), source);

  let phrase = &file.get_dialogs()[0].get_phrases()[0];

  assert_eq!(
    &file.get_source()[phrase.get_range().clone()],
    "<phrase id=\"0\"><text>key</text></phrase>"
  );

  Ok(())
}

#[test]
fn reads_a_file_holding_a_bare_ampersand() -> XrfResult {
  // Not a reference, and dialog text is full of them.
  let file: DialogFile = parse(
    r#"<game_dialogs><dialog id="d"><phrase_list><phrase id="0"><text>Duty & Freedom</text></phrase></phrase_list></dialog></game_dialogs>"#,
  )?;

  assert_eq!(file.get_dialogs().len(), 1);

  Ok(())
}

#[test]
fn formats_an_issue_with_where_it_was_found() {
  let issue: DialogParseIssue = DialogParseIssue::new(
    DialogParseIssueKind::UnknownElement,
    Some(String::from("zat_b30_owl")),
    Some(String::from("12")),
    String::from("go_back"),
    0..0,
  );

  assert_eq!(issue.to_string(), "zat_b30_owl#12: unknown element 'go_back'");
}
