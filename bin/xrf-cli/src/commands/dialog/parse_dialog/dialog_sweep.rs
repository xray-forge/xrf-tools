//! Reads every dialog file under a path and accounts for what came out.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use xrf_dialog::{DialogFile, DialogParseIssue, DialogParseIssueKind, DialogPhrase, DialogProject};
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};

/// What a sweep counted, beside what it found.
///
/// These are the numbers the reader's premises rest on: that phrases nearly always sit in a
/// `phrase_list`, that a dialog may legitimately declare none, and that the element set is the one
/// `DialogElementKind` classifies.
#[derive(Debug, Default)]
pub struct DialogSweepCensus {
  pub files: usize,
  pub unreadable_files: usize,
  pub dialogs: usize,
  pub phrases: usize,
  pub dialogs_without_phrases: usize,
  pub dialogs_with_priority: usize,
  pub phrases_outside_phrase_list: usize,
  pub phrases_without_text: usize,
  pub final_phrases: usize,
  pub links: usize,
  pub largest_dialog_phrases: usize,
  pub largest_dialog_id: Option<String>,
  pub encodings: BTreeMap<String, usize>,
  pub dialog_elements: BTreeMap<String, usize>,
  pub phrase_elements: BTreeMap<String, usize>,
}

impl DialogSweepCensus {
  fn count(map: &mut BTreeMap<String, usize>, key: impl Into<String>) {
    *map.entry(key.into()).or_default() += 1;
  }
}

/// Outcome of a sweep: a finalized report and the counts behind it.
#[derive(Debug)]
pub struct DialogSweepResult {
  pub census: DialogSweepCensus,
  pub duration: Duration,
  pub report: Report,
}

pub struct DialogSweep<'a> {
  root: &'a Path,
}

impl<'a> DialogSweep<'a> {
  pub fn new(root: &'a Path) -> Self {
    Self { root }
  }

  /// Read every dialog file the path covers.
  ///
  /// A file that cannot be read is recorded and the sweep continues: the point of a sweep over four
  /// reference trees is the tally, and stopping at the first unreadable file would never produce one.
  pub fn run(&self) -> DialogSweepResult {
    let started: Instant = Instant::now();

    let mut census: DialogSweepCensus = DialogSweepCensus::default();
    let mut read_findings: Vec<Finding> = Vec::new();
    let mut schema_findings: Vec<Finding> = Vec::new();

    for path in self.dialog_paths() {
      census.files += 1;

      match DialogFile::read_from_path(&path) {
        Ok(file) => {
          DialogSweepCensus::count(&mut census.encodings, file.get_encoding().name());
          Self::census_file(&mut census, &file);

          for issue in file.get_issues() {
            schema_findings.push(Self::new_issue_finding(&path, issue));
          }
        }
        Err(error) => {
          census.unreadable_files += 1;
          read_findings.push(Finding::new(
            Self::rule("unreadable"),
            Some(path.display().to_string()),
            error.to_string(),
          ));
        }
      }
    }

    let duration: Duration = started.elapsed();

    // Nothing swept is not a pass. A mistyped path walks no files and produces no findings, and
    // reporting that as success is how a sweep gets wired into CI and silently checks nothing.
    let status = |is_valid: bool| -> Status {
      if census.files == 0 {
        Status::Skipped
      } else {
        Status::from_is_valid(is_valid)
      }
    };

    DialogSweepResult {
      report: Report::new(vec![
        CheckReport::new(
          Self::check("read"),
          status(read_findings.is_empty()),
          Some(duration),
          read_findings,
        ),
        CheckReport::new(
          Self::check("schema"),
          status(schema_findings.is_empty()),
          Some(duration),
          schema_findings,
        ),
      ]),
      census,
      duration,
    }
  }

  /// Every dialog file the sweep covers: one named file, or every dialog XML under a directory.
  ///
  /// Discovery belongs to the crate, so the sweep and a project open agree on what a dialog file is.
  /// A named file is taken as given, so a caller can read a file the filter would not have picked up.
  fn dialog_paths(&self) -> Vec<PathBuf> {
    if self.root.is_file() {
      return vec![self.root.to_path_buf()];
    }

    DialogProject::list_dialog_paths(self.root)
  }

  fn census_file(census: &mut DialogSweepCensus, file: &DialogFile) {
    for dialog in file.get_dialogs() {
      census.dialogs += 1;

      if dialog.get_priority().is_some() {
        census.dialogs_with_priority += 1;
      }

      for element in dialog.get_elements() {
        DialogSweepCensus::count(&mut census.dialog_elements, element.get_name());
      }

      let phrases: &[DialogPhrase] = dialog.get_phrases();

      if phrases.is_empty() {
        census.dialogs_without_phrases += 1;
      }

      if phrases.len() > census.largest_dialog_phrases {
        census.largest_dialog_phrases = phrases.len();
        census.largest_dialog_id = Some(dialog.get_id().to_owned());
      }

      for phrase in phrases {
        Self::census_phrase(census, phrase);
      }
    }
  }

  fn census_phrase(census: &mut DialogSweepCensus, phrase: &DialogPhrase) {
    census.phrases += 1;
    census.links += phrase.list_next().len();

    if !phrase.is_in_phrase_list() {
      census.phrases_outside_phrase_list += 1;
    }

    if phrase.get_text().is_none() {
      census.phrases_without_text += 1;
    }

    if phrase.is_final() {
      census.final_phrases += 1;
    }

    for element in phrase.get_elements() {
      DialogSweepCensus::count(&mut census.phrase_elements, element.get_name());
    }
  }

  /// Turn a reader issue into a finding, keyed by the rule that would judge it.
  fn new_issue_finding(path: &Path, issue: &DialogParseIssue) -> Finding {
    let rule: &str = match issue.get_kind() {
      DialogParseIssueKind::UnknownElement => "unknown-element",
      DialogParseIssueKind::UnknownAttribute => "unknown-attribute",
      DialogParseIssueKind::MissingId => "missing-id",
      DialogParseIssueKind::InvalidPriority => "invalid-priority",
    };

    Finding::new(Self::rule(rule), Some(path.display().to_string()), issue.to_string())
  }

  fn check(id: &str) -> CheckId {
    CheckId::new(format!("dialog.{id}")).expect("dialog check ids are not empty")
  }

  fn rule(id: &str) -> RuleId {
    RuleId::new(format!("dialog.{id}")).expect("dialog rule ids are not empty")
  }
}

/// Total findings across every check, which is what the failure line counts.
pub fn sum_findings(report: &Report) -> usize {
  report
    .checks()
    .iter()
    .map(|check| check.findings().len())
    .sum::<usize>()
}

/// Render a counted distribution as `name: count` pairs, in name order.
pub fn list_distribution(counts: &BTreeMap<String, usize>) -> Vec<String> {
  counts.iter().map(|(name, count)| format!("{name}: {count}")).collect()
}
