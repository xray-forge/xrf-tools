use std::time::Duration;

use xrf_report::Report;

use crate::verify::level_ai_graph_census::LevelAiGraphCensus;

/// A level AI map check's report and the counts behind it; no counts where the level ships no map or it could not be
/// read.
#[derive(Debug)]
pub struct LevelAiGraphVerifyResult {
  pub census: Option<LevelAiGraphCensus>,
  pub duration: Duration,
  pub report: Report,
}
