use serde::{Deserialize, Serialize};

use crate::contract::render_graph_pass_kind::RenderGraphPassKind;

/// One pass a viewport's frame ran, as the frame graph placed it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphPass {
  pub name: String,
  pub kind: RenderGraphPassKind,
  /// The encode group it is recorded in.
  pub group: String,
  /// The render pass it draws in, which the raster passes beside it may share.
  pub render_pass: Option<u32>,
}
