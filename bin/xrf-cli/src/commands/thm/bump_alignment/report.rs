use serde::Serialize;
use xrf_texture::{BumpAlignment, BumpAlignmentVerdict};

use crate::commands::thm::bump_alignment::bump_alignment_set::{BumpAlignmentOutcome, BumpAlignmentSet};
use crate::commands::thm::bump_alignment::bump_alignment_survey::BumpAlignmentSurvey;

/// What `thm bump-alignment` compared: the sets at a glance, then every set.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpAlignmentReport<'a> {
  summary: BumpAlignmentSummary,
  sets: &'a [BumpAlignmentSet],
}

/// The sets counted by what comparing them came to.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpAlignmentSummary {
  /// The lifts over chance the verdicts judge by.
  pub aligned_lift: f32,
  pub mismatched_lift: f32,
  pub descriptors: usize,
  pub sets: usize,
  pub measured: usize,
  pub aligned: usize,
  pub doubtful: usize,
  pub mismatched: usize,
  /// Of the mismatched, those whose bump is their diffuse's own by name: the sets a pack installed in part leaves.
  pub mismatched_own: usize,
  pub featureless: usize,
  /// Sets whose bump is another texture's.
  pub borrowed: usize,
  pub missing_diffuses: usize,
  pub missing_bumps: usize,
  pub unread: usize,
}

impl<'a> BumpAlignmentReport<'a> {
  pub fn new(survey: &'a BumpAlignmentSurvey, summary: BumpAlignmentSummary) -> Self {
    Self {
      summary,
      sets: &survey.sets,
    }
  }
}

impl BumpAlignmentSummary {
  pub fn of(survey: &BumpAlignmentSurvey) -> Self {
    let measured: Vec<&BumpAlignment> = survey.sets.iter().filter_map(BumpAlignmentSet::get_alignment).collect();
    let count = |verdict: BumpAlignmentVerdict| measured.iter().filter(|it| it.verdict == verdict).count();
    let count_outcome = |is: fn(&BumpAlignmentOutcome) -> bool| survey.sets.iter().filter(|it| is(&it.outcome)).count();

    Self {
      aligned_lift: BumpAlignmentVerdict::ALIGNED_LIFT,
      mismatched_lift: BumpAlignmentVerdict::MISMATCHED_LIFT,
      descriptors: survey.descriptors,
      sets: survey.sets.len(),
      measured: measured.len(),
      aligned: count(BumpAlignmentVerdict::Aligned),
      doubtful: count(BumpAlignmentVerdict::Doubtful),
      mismatched: count(BumpAlignmentVerdict::Mismatched),
      mismatched_own: survey
        .sets
        .iter()
        .filter(|it| !it.is_borrowed)
        .filter_map(BumpAlignmentSet::get_alignment)
        .filter(|it| it.verdict == BumpAlignmentVerdict::Mismatched)
        .count(),
      featureless: count(BumpAlignmentVerdict::Featureless),
      borrowed: survey.sets.iter().filter(|it| it.is_borrowed).count(),
      missing_diffuses: count_outcome(|it| matches!(it, BumpAlignmentOutcome::MissingDiffuse)),
      missing_bumps: count_outcome(|it| matches!(it, BumpAlignmentOutcome::MissingBump)),
      unread: count_outcome(|it| matches!(it, BumpAlignmentOutcome::Unread { .. })),
    }
  }
}
