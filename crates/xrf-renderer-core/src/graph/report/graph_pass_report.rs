use serde::Serialize;

use crate::graph::report::graph_pass_kind::GraphPassKind;

/// One surviving pass as the compile placed it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphPassReport {
  pub name: String,
  pub kind: GraphPassKind,
  pub group: String,
  /// Whose it is: a view of a frame drawing several, or the frame's own.
  pub owner: u32,
  /// The render pass it draws in, which raster passes beside it may share.
  pub render_pass: Option<usize>,
}
