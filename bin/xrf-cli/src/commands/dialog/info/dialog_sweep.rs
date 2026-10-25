//! Reads every dialog file mounted roots exposes and accounts for what came out.
//!
//! Lives in the CLI rather than in `xrf-dialog` so that crate stays a reader with no reporting
//! surface. The census is the point: it is what says whether the schema the reader models is the
//! schema the shipped files are written in.

use std::time::{Duration, Instant};

use xrf_dialog::{DialogFile, DialogParseIssue, DialogParseIssueKind, DialogPhrase, DialogProject};
use xrf_error::XrfResult;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_vfs::{XrayAsset, XrayRoots, XraySearch};

use crate::commands::dialog::info::dialog_sweep_census::DialogSweepCensus;
use crate::commands::dialog::info::dialog_sweep_result::DialogSweepResult;

/// Which roots to mount, and where to look inside it.
pub struct DialogSweep<'a> {
  roots: &'a XrayRoots,
  prefix: Option<&'a str>,
}

impl<'a> DialogSweep<'a> {
  pub fn new(roots: &'a XrayRoots, prefix: Option<&'a str>) -> Self {
    Self { roots, prefix }
  }

  /// Read every dialog file the mounted roots exposes.
  ///
  /// Reads through the VFS, so an installation sweeps as readily as a loose tree: on a real game the
  /// dialogs come out of `db\configs`, and a reader reaching for the filesystem reports them absent.
  ///
  /// A file that cannot be read is recorded and the sweep continues: the point of sweeping several
  /// reference trees is the tally, and stopping at the first unreadable file would never produce one.
  ///
  /// # Errors
  ///
  /// Returns an error when the path cannot be mounted, or when the prefix is not a logical path.
  pub fn run(&self) -> XrfResult<DialogSweepResult> {
    let started: Instant = Instant::now();

    let search: XraySearch = self.roots.to_search_plan()?.open()?.with_optional_prefix(self.prefix)?;

    let mut census: DialogSweepCensus = DialogSweepCensus::default();
    let mut read_findings: Vec<Finding> = Vec::new();
    let mut schema_findings: Vec<Finding> = Vec::new();

    for asset in DialogProject::list_dialog_assets(&search) {
      let logical_path: &str = asset.get_logical_path().as_str();

      census.files += 1;

      if asset.to_physical_path().is_none() {
        census.archived_files += 1;
      }

      match Self::read(&search, &asset) {
        Ok(file) => {
          DialogSweepCensus::count(&mut census.encodings, file.get_encoding().name());
          Self::census_file(&mut census, &file);

          for issue in file.get_issues() {
            schema_findings.push(Self::new_issue_finding(&asset, issue));
          }
        }
        Err(error) => {
          census.unreadable_files += 1;
          read_findings.push(Finding::new(
            Self::rule("unreadable"),
            Some(logical_path.to_owned()),
            error.to_string(),
          ));
        }
      }
    }

    let duration: Duration = started.elapsed();

    // Nothing swept is not a pass. A mistyped path mounts an empty roots and produces no findings, and
    // reporting that as success is how a sweep gets wired into CI and silently checks nothing.
    let status = |is_valid: bool| -> Status {
      if census.files == 0 {
        Status::Skipped
      } else {
        Status::from_is_valid(is_valid)
      }
    };

    Ok(DialogSweepResult {
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
    })
  }

  fn read(search: &XraySearch, asset: &XrayAsset) -> XrfResult<DialogFile> {
    DialogFile::read_from_bytes(&search.read_asset(asset)?)
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
  fn new_issue_finding(asset: &XrayAsset, issue: &DialogParseIssue) -> Finding {
    let rule: &str = match issue.get_kind() {
      DialogParseIssueKind::UnknownElement => "unknown-element",
      DialogParseIssueKind::UnknownAttribute => "unknown-attribute",
      DialogParseIssueKind::MissingId => "missing-id",
      DialogParseIssueKind::InvalidPriority => "invalid-priority",
    };

    Finding::new(
      Self::rule(rule),
      Some(asset.get_logical_path().as_str().to_owned()),
      issue.to_string(),
    )
  }

  fn check(id: &str) -> CheckId {
    CheckId::new(format!("dialog.{id}")).expect("dialog check ids are not empty")
  }

  fn rule(id: &str) -> RuleId {
    RuleId::new(format!("dialog.{id}")).expect("dialog rule ids are not empty")
  }
}
