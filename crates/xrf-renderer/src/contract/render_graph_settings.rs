use serde::{Deserialize, Serialize};
use xrf_renderer_core::GraphCompileOptions;

/// Which of the frame graph's mechanisms every frame compiles with, each to be turned off alone, or all for serial mode,
/// to bisect a difference in a capture.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphSettings {
  /// Drops passes whose effects nothing reads.
  pub is_culling: bool,
  /// Lets transients whose lifetimes do not overlap share one texture or buffer.
  pub is_pooling: bool,
  /// Draws consecutive raster passes into the same attachments in one render pass.
  pub is_merging: bool,
  /// Records each encode group into an encoder of its own, in parallel; off, the whole frame is one.
  pub is_grouping: bool,
}

impl Default for RenderGraphSettings {
  fn default() -> Self {
    Self {
      is_culling: true,
      is_pooling: true,
      is_merging: true,
      is_grouping: true,
    }
  }
}

impl RenderGraphSettings {
  /// The compile options they ask for.
  pub fn to_options(self) -> GraphCompileOptions {
    GraphCompileOptions {
      is_culling: self.is_culling,
      is_pooling: self.is_pooling,
      is_merging: self.is_merging,
      is_grouping: self.is_grouping,
    }
  }
}
