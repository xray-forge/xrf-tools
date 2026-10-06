use serde::Serialize;

use crate::graph::record::GraphPassWork;

/// What kind of pass a report names; a bridge is an encoder pass the graph cannot check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GraphPassKind {
  Raster,
  Compute,
  Encoder,
  Bridge,
}

impl GraphPassKind {
  pub(crate) fn of(work: &GraphPassWork<'_>) -> Self {
    match work {
      GraphPassWork::Raster { .. } => Self::Raster,
      GraphPassWork::Compute { .. } => Self::Compute,
      GraphPassWork::Encoder { is_bridge: false, .. } => Self::Encoder,
      GraphPassWork::Encoder { is_bridge: true, .. } => Self::Bridge,
    }
  }
}
