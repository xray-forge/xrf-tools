use std::path::Path;
use std::time::Duration;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_utils::to_portable_path_string;

/// What one language is missing from one file.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ProjectVerifyLanguageSummary {
  /// The source this counts, as the project addresses it.
  pub file: String,
  pub language: String,
  /// Ids the file holds, which is the same for every language of that file.
  pub checked: u32,
  /// Ids with no text for this language, counting an explicit `null` as missing.
  pub missing: u32,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectVerifyResult {
  #[serde(with = "xrf_utils::duration_ms")]
  pub duration: Duration,
  pub checked_translations_count: u32,
  pub missing_translations_count: u32,
  /// Per file and language, in discovery order.
  pub languages: Vec<ProjectVerifyLanguageSummary>,
  #[serde(skip_serializing)]
  findings: Vec<Finding>,
  /// Keyed by file and language so repeated calls for one file accumulate into one row.
  #[serde(skip_serializing)]
  summary_index: IndexMap<(String, String), usize>,
}

impl ProjectVerifyResult {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn findings(&self) -> &[Finding] {
    &self.findings
  }

  pub fn status(&self) -> Status {
    Status::from_is_valid(self.missing_translations_count == 0)
  }

  pub fn to_report(&self) -> Report {
    Report::new(vec![CheckReport::new(
      CheckId::new("translations").expect("Expected a non-empty translation check ID"),
      self.status(),
      Some(self.duration),
      self.findings.clone(),
    )])
  }

  pub(crate) fn merge(&mut self, other: Self) {
    self.checked_translations_count += other.checked_translations_count;
    self.missing_translations_count += other.missing_translations_count;
    self.findings.extend(other.findings);

    for summary in other.languages {
      self.record_language_summary(summary);
    }
  }

  /// Note how much of one file one language holds, whether or not anything is missing.
  ///
  /// A complete language is worth a row: "0 missing" is the answer somebody is looking for as often as
  /// a number, and a table that only lists failures cannot distinguish a language that is finished from
  /// one the project does not carry at all.
  pub(crate) fn record_language_summary(&mut self, summary: ProjectVerifyLanguageSummary) {
    let key: (String, String) = (summary.file.clone(), summary.language.clone());

    match self.summary_index.get(&key) {
      Some(&index) => {
        let existing: &mut ProjectVerifyLanguageSummary = &mut self.languages[index];

        existing.checked += summary.checked;
        existing.missing += summary.missing;
      }
      None => {
        self.summary_index.insert(key, self.languages.len());
        self.languages.push(summary);
      }
    }
  }

  /// Count one missing translation, and describe it when the caller asked for detail.
  ///
  /// `is_detailed` decides whether the finding is built at all rather than whether it is reported. A
  /// desktop run wants the aggregate and would otherwise pay to assemble 150,000 findings on its way
  /// to discarding them.
  pub(crate) fn record_missing_translation(&mut self, path: &Path, key: &str, language: &str, is_detailed: bool) {
    self.missing_translations_count += 1;

    if !is_detailed {
      return;
    }

    self.findings.push(Finding::new(
      RuleId::new("translations.missing").expect("Expected a non-empty translation rule ID"),
      Some(to_portable_path_string(path)),
      format!("Missing translation for key '{key}' in language '{language}'"),
    ));
  }
}
