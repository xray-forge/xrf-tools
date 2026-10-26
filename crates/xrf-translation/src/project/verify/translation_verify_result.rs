use std::path::Path;
use std::time::Duration;

use indexmap::IndexMap;
use serde::Serialize;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_utils::to_portable_path_string;

use crate::project::describe::translation_finding::TranslationFinding;
use crate::project::verify::translation_verify_language_summary::TranslationVerifyLanguageSummary;

/// What a check of translation sources found: how complete each file is per language, and what the game would refuse
/// or mangle in them.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationVerifyResult {
  /// Whether the run checked every source or was stopped between them.
  pub outcome: xrf_job::JobOutcome,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  pub checked_translations_count: u32,
  pub missing_translations_count: u32,
  /// Per file and language, in discovery order.
  pub languages: Vec<TranslationVerifyLanguageSummary>,
  /// What the game would refuse or mangle: a character a language's code page cannot hold, a line past what its XML
  /// reader takes, an id two sources define, two sources building to one table.
  ///
  /// Listed in full however the run was asked to report, because there are few of them and each one is a table the
  /// game shows wrong or will not load. Unlike a missing translation, which the game shows as its id, every one fails
  /// the run.
  pub problems: Vec<TranslationFinding>,
  #[serde(skip_serializing)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  findings: Vec<Finding>,
  /// Keyed by file and language so repeated calls for one file accumulate into one row.
  #[serde(skip_serializing)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  summary_index: IndexMap<(String, String), usize>,
}

impl TranslationVerifyResult {
  /// The check missing translations are judged by.
  pub const COMPLETENESS_CHECK: &str = "translations";

  /// The check what the game would refuse or mangle is judged by.
  pub const ENGINE_CHECK: &str = "translations.engine";

  pub fn new() -> Self {
    Self::default()
  }

  /// The missing translations, when the run was asked to describe them.
  pub fn findings(&self) -> &[Finding] {
    &self.findings
  }

  /// The verdict of both checks: complete, and nothing the game would refuse or mangle.
  pub fn status(&self) -> Status {
    Status::from_is_valid(self.missing_translations_count == 0 && self.problems.is_empty())
  }

  /// The verdict of the engine check alone, which a run judges whether or not it judges completeness.
  pub fn get_engine_status(&self) -> Status {
    Status::from_is_valid(self.problems.is_empty())
  }

  /// Both checks, as a run that judges completeness concludes on.
  pub fn to_report(&self) -> Report {
    Report::new(vec![self.to_completeness_check(), self.to_engine_check()])
  }

  /// The engine check alone, as a run that leaves completeness unjudged concludes on.
  pub fn to_engine_report(&self) -> Report {
    Report::new(vec![self.to_engine_check()])
  }

  pub(crate) fn merge(&mut self, other: Self) {
    self.checked_translations_count += other.checked_translations_count;
    self.missing_translations_count += other.missing_translations_count;
    self.findings.extend(other.findings);
    self.problems.extend(other.problems);

    for summary in other.languages {
      self.record_language_summary(summary);
    }
  }

  /// Note how much of one file one language holds, whether or not anything is missing.
  ///
  /// A complete language is worth a row: "0 missing" is the answer somebody is looking for as often as
  /// a number, and a table that only lists failures cannot distinguish a language that is finished from
  /// one the project does not carry at all.
  pub(crate) fn record_language_summary(&mut self, summary: TranslationVerifyLanguageSummary) {
    let key: (String, String) = (summary.file.clone(), summary.language.clone());

    match self.summary_index.get(&key) {
      Some(&index) => {
        let existing: &mut TranslationVerifyLanguageSummary = &mut self.languages[index];

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
      RuleId::of_static("translations.missing"),
      Some(to_portable_path_string(path)),
      format!("Missing translation for key '{key}' in language '{language}'"),
    ));
  }

  fn to_completeness_check(&self) -> CheckReport {
    CheckReport::new(
      CheckId::of_static(Self::COMPLETENESS_CHECK),
      Status::from_is_valid(self.missing_translations_count == 0),
      Some(self.duration),
      self.findings.clone(),
    )
  }

  fn to_engine_check(&self) -> CheckReport {
    CheckReport::new(
      CheckId::of_static(Self::ENGINE_CHECK),
      self.get_engine_status(),
      Some(self.duration),
      self
        .problems
        .iter()
        .map(|problem| {
          Finding::new(
            RuleId::new(problem.rule.clone()).expect("a translation problem names its rule by a non-empty literal"),
            problem.subject.clone(),
            problem.message.clone(),
          )
        })
        .collect(),
    )
  }
}
