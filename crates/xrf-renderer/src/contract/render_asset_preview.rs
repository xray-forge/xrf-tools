use serde::{Deserialize, Serialize};

use crate::contract::render_asset_lighting::RenderAssetLighting;
use crate::contract::render_backdrop_squares::RenderBackdropSquares;

/// How an asset viewer stages what it shows: its own light, the backdrop behind it, and how its surfaces are textured
/// where it checks their coordinates or they name no texture.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct RenderAssetPreview {
  /// An asset viewer's light, in place of the weather's; none for a level.
  pub lighting: Option<RenderAssetLighting>,
  /// What shows where nothing was drawn and neither the sky nor the fog is, each channel zero to one; none for the
  /// level viewer's own.
  pub backdrop: Option<[f32; 3]>,
  /// The backdrop laid out as a checkerboard with a second colour, as behind a picture with alpha; none for a plain one.
  pub backdrop_squares: Option<RenderBackdropSquares>,
  /// Times a uv checker repeats over a surface's base coordinate, drawn in place of its textures; zero for none.
  pub checker: f32,
  /// The colour a surface naming no base texture is drawn, each channel zero to one; none for white.
  pub plain_color: Option<[f32; 3]>,
}
