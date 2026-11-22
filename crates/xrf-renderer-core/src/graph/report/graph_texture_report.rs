use serde::Serialize;

use crate::graph::graph_owner::GraphOwner;

/// One texture the graph names, imported or transient.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphTextureReport {
  pub label: String,
  /// Whose it was declared under, as a pass declared then is.
  pub owner: GraphOwner,
  /// What its declaration tagged it with (`FrameGraph::tag_texture`).
  pub tags: Vec<&'static str>,
}
