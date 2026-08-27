use std::path::Path;

use xrf_report::Status;

use crate::project::verify::result::{ProjectVerifyLanguageSummary, ProjectVerifyResult};

#[test]
fn reports_missing_translations_as_findings() {
  let mut result: ProjectVerifyResult = ProjectVerifyResult::new();

  result.record_missing_translation(Path::new("translations/dialogs.json"), "st_dialog_hello", "ukr", true);

  let report = result.to_report();

  assert_eq!(result.status(), Status::Failed);
  assert_eq!(report.status(), Status::Failed);
  assert_eq!(report.checks()[0].id().as_str(), "translations");
  assert_eq!(
    report.checks()[0].findings()[0].rule_id().as_str(),
    "translations.missing"
  );
  assert_eq!(
    report.checks()[0].findings()[0].subject(),
    Some("translations/dialogs.json")
  );
}

#[test]
fn a_project_with_nothing_missing_passes() {
  let result: ProjectVerifyResult = ProjectVerifyResult::new();

  assert_eq!(result.status(), Status::Passed);
  assert!(result.findings().is_empty());
}

#[test]
fn merging_accumulates_counts_and_findings() {
  let mut first: ProjectVerifyResult = ProjectVerifyResult::new();
  let mut second: ProjectVerifyResult = ProjectVerifyResult::new();

  first.checked_translations_count = 2;
  first.record_missing_translation(Path::new("a.json"), "st_a", "ukr", true);

  second.checked_translations_count = 3;
  second.record_missing_translation(Path::new("b.json"), "st_b", "pol", true);

  first.merge(second);

  assert_eq!(first.checked_translations_count, 5);
  assert_eq!(first.missing_translations_count, 2);
  assert_eq!(first.findings().len(), 2);
}

#[test]
fn counting_without_detail_leaves_the_findings_out() {
  // A desktop run wants the aggregate. Building 149,979 findings on the way to discarding them is the
  // cost this switch exists to avoid, so the count has to move without them.
  let mut result: ProjectVerifyResult = ProjectVerifyResult::new();

  result.record_missing_translation(Path::new("a.json"), "st_a", "ukr", false);

  assert_eq!(result.missing_translations_count, 1);
  assert!(result.findings().is_empty());
  assert_eq!(result.status(), Status::Failed);
}

#[test]
fn a_language_summary_is_kept_for_a_complete_language_too() {
  // "0 missing" is an answer. A table that only listed failures could not tell a finished language
  // from one the project does not carry at all.
  let mut result: ProjectVerifyResult = ProjectVerifyResult::new();

  result.record_language_summary(ProjectVerifyLanguageSummary {
    file: String::from("a.json"),
    language: String::from("eng"),
    checked: 3,
    missing: 0,
  });

  assert_eq!(result.languages.len(), 1);
  assert_eq!(result.languages[0].missing, 0);
  assert_eq!(result.status(), Status::Passed);
}

#[test]
fn merging_keeps_one_row_per_file_and_language() {
  let mut first: ProjectVerifyResult = ProjectVerifyResult::new();
  let mut second: ProjectVerifyResult = ProjectVerifyResult::new();

  first.record_language_summary(ProjectVerifyLanguageSummary {
    file: String::from("a.json"),
    language: String::from("eng"),
    checked: 2,
    missing: 1,
  });
  second.record_language_summary(ProjectVerifyLanguageSummary {
    file: String::from("b.json"),
    language: String::from("eng"),
    checked: 3,
    missing: 0,
  });

  first.merge(second);

  assert_eq!(first.languages.len(), 2);
  assert_eq!(first.languages[0].file, "a.json");
  assert_eq!(first.languages[1].file, "b.json");
}
