use serde::Serialize;

use crate::graph::report::graph_pass_report::GraphPassReport;
use crate::graph::report::graph_transient_report::GraphTransientReport;

/// What a compile made of a frame graph: the passes that run and where, those culled, and the transients and the pooled
/// resources they share.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphReport {
  pub passes: Vec<GraphPassReport>,
  pub culled: Vec<String>,
  pub groups: Vec<String>,
  pub render_pass_count: usize,
  pub transients: Vec<GraphTransientReport>,
  /// Bytes the transients would take apart.
  pub transient_bytes: u64,
  /// Pooled textures and buffers the transients share, and the bytes those take.
  pub pooled_count: usize,
  pub pooled_bytes: u64,
}
