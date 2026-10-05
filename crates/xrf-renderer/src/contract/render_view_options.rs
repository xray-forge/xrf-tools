use serde::{Deserialize, Serialize};

use crate::contract::render_asset_preview::RenderAssetPreview;
use crate::contract::render_show_flags::RenderShowFlags;
use crate::contract::render_view_features::RenderViewFeatures;
use crate::contract::render_view_mode::RenderViewMode;
use crate::contract::render_view_output::RenderViewOutput;
use crate::contract::render_world_toggles::RenderWorldToggles;

/// What one viewport draws its scene with, split by who owns each part: what it shows, how it shades it, each
/// feature's settings, what its frame is drawn at, what of the world plays, and an asset viewer's staging.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct RenderViewOptions {
  pub show: RenderShowFlags,
  pub mode: RenderViewMode,
  pub features: RenderViewFeatures,
  pub output: RenderViewOutput,
  /// What of the level plays; the world's to own once it is a layer of its own.
  pub world: RenderWorldToggles,
  pub asset: RenderAssetPreview,
}
