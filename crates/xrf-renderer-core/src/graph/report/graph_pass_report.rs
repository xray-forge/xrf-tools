use serde::Serialize;

use crate::graph::graph_owner::GraphOwner;
use crate::graph::report::graph_pass_kind::GraphPassKind;

/// One surviving pass as the compile placed it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphPassReport {
  pub name: String,
  pub kind: GraphPassKind,
  pub group: String,
  /// Whose it is: a view of a frame drawing several, or the frame's own.
  pub owner: GraphOwner,
  /// The render pass it draws in, which raster passes beside it may share.
  pub render_pass: Option<usize>,
  /// The textures it reads and those it writes, attachments among them, as places in the report's `textures`.
  pub reads: Vec<usize>,
  pub writes: Vec<usize>,
}
