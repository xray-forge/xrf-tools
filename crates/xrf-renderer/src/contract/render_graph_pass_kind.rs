use serde::{Deserialize, Serialize};
use xrf_renderer_core::GraphPassKind;

/// What kind of pass a frame graph ran: a raster or compute pass, or an encoder pass recording copies and readbacks; a
/// bridge is an encoder pass the graph cannot check.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderGraphPassKind {
  Raster,
  Compute,
  Encoder,
  Bridge,
}

impl From<GraphPassKind> for RenderGraphPassKind {
  fn from(kind: GraphPassKind) -> Self {
    match kind {
      GraphPassKind::Raster => Self::Raster,
      GraphPassKind::Compute => Self::Compute,
      GraphPassKind::Encoder => Self::Encoder,
      GraphPassKind::Bridge => Self::Bridge,
    }
  }
}
