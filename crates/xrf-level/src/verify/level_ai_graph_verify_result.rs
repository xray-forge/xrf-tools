use std::time::Duration;

use xrf_report::Report;

use crate::verify::level_ai_graph_census::LevelAiGraphCensus;

/// What a check of a level's AI map concluded: the report a caller judges, and the counts behind it, none where the
/// level ships no map or it could not be read.
#[derive(Debug)]
pub struct LevelAiGraphVerifyResult {
  pub census: Option<LevelAiGraphCensus>,
  pub duration: Duration,
  pub report: Report,
}
