use serde::{Deserialize, Serialize};

/// How a screenshot of a viewport is drawn: its finished frame whole, without its overlays or its selection's outline,
/// and its level objects' helpers over it where `is_helped`, as its window draws them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderScreenshot {
  pub is_helped: bool,
}
