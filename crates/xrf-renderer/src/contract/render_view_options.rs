use serde::{Deserialize, Serialize};

use crate::contract::render_asset_preview::RenderAssetPreview;
use crate::contract::render_show_flags::RenderShowFlags;
use crate::contract::render_view_features::RenderViewFeatures;
use crate::contract::render_view_mode::RenderViewMode;
use crate::contract::render_view_output::RenderViewOutput;

/// What one viewport draws its scene with, split by who owns each part: what it shows, how it shades it, each
/// feature's settings, what its frame is drawn at, and an asset viewer's staging. What of the world plays is the
/// world's own.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewOptions {
  pub show: RenderShowFlags,
  pub mode: RenderViewMode,
  pub features: RenderViewFeatures,
  pub output: RenderViewOutput,
  pub asset: RenderAssetPreview,
}
