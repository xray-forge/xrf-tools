use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_utils::to_portable_path_string;

/// What one import run read, changed, and objected to.
///
/// The census counts and the findings answer different questions and are kept apart for that reason:
/// a count says how much moved, a finding says which file has a problem and why. `subject` on a
/// finding is what makes a per-file breakdown unnecessary.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectParseResult {
  /// Whether the run read every table or was stopped between them.
  ///
  /// Each source is written whole through a staged replace, so a stopped import leaves the ones already written
  /// complete and the rest untouched.
  pub outcome: xrf_job::JobOutcome,
  #[serde(with = "xrf_utils::duration_ms")]
  pub duration: Duration,
  /// The language this run filed every entry under.
  pub language: String,
  /// Whether the run computed its answer without writing it.
  ///
  /// A dry run's whole result is what it would have written, so the counts below mean "would" rather
  /// than "did" whenever this is set.
  pub is_dry_run: bool,
  pub census: ProjectParseCensus,
  /// The findings, in the shape every other XRF command reports them in.
  ///
  /// Sealed by [`ProjectParseResult::finalize`] once the run is over, so the crate's own result is
  /// what both the CLI and the desktop app deposit — neither has to restate the shape, and they
  /// cannot drift apart by restating it differently.
  pub report: Report,
  #[serde(skip_serializing)]
  findings: Vec<Finding>,
}

/// How much one import run moved.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ProjectParseCensus {
  /// String tables read out of the scope.
  pub files_read: u32,
  /// JSON sources created because nothing was there yet.
  pub files_created: u32,
  /// JSON sources rewritten because merging changed something in them.
  pub files_updated: u32,
  /// JSON sources left alone because merging changed nothing.
  pub files_unchanged: u32,
  /// Files skipped without being read, because they are not string tables or hold no entries.
  pub files_skipped: u32,
  /// Entries read out of the XML, before any merging.
  pub entries_read: u32,
  /// Ids this run introduced to their file.
  pub entries_inserted: u32,
  /// Placeholders this run replaced with text.
  pub entries_filled: u32,
  /// Ids whose text already matched what was read.
  pub entries_unchanged: u32,
  /// Ids whose existing text differed from what was read.
  ///
  /// Kept unless the run was told to overwrite, in which case this counts what was replaced.
  pub entries_conflicted: u32,
  /// `null` placeholders added for languages a file carries but a record did not.
  pub placeholders_added: u32,
}

impl ProjectParseResult {
  pub fn new(language: &str, is_dry_run: bool) -> Self {
    Self {
      outcome: xrf_job::JobOutcome::Completed,
      duration: Duration::ZERO,
      language: language.to_owned(),
      is_dry_run,
      census: ProjectParseCensus::default(),
      report: Report::new(Vec::new()),
      findings: Vec::new(),
    }
  }

  /// Seal the findings into the report, once nothing more can be added.
  ///
  /// Separate from recording them because the report carries the run's duration, which is not known
  /// until the run is over.
  pub(crate) fn finalize(&mut self, duration: Duration) {
    self.duration = duration;
    self.report = self.to_report();
  }

  pub fn get_findings(&self) -> &[Finding] {
    &self.findings
  }

  /// Whether anything was worth objecting to.
  ///
  /// A conflict is not a failure: keeping existing text is the documented behaviour, not a problem
  /// with the input. Only a file that could not be read or understood is.
  pub fn get_status(&self) -> Status {
    Status::from_is_valid(self.findings.is_empty())
  }

  pub fn to_report(&self) -> Report {
    Report::new(vec![CheckReport::new(
      CheckId::new("translations").expect("Expected a non-empty translation check ID"),
      self.get_status(),
      Some(self.duration),
      self.findings.clone(),
    )])
  }

  /// Record something about a file that was read through the VFS, named by its logical path.
  pub(crate) fn record_finding(&mut self, rule: &'static str, subject: &str, message: impl Into<String>) {
    self.findings.push(Finding::new(
      RuleId::new(rule).expect("Expected a non-empty translation rule ID"),
      Some(subject.to_owned()),
      message,
    ));
  }

  /// Record something about a file on the host, whose path is formatted for showing.
  pub(crate) fn record_path_finding(&mut self, rule: &'static str, path: &Path, message: impl Into<String>) {
    self.record_finding(rule, &to_portable_path_string(path), message);
  }
}
