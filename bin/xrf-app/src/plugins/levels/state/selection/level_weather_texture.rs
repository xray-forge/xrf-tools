use serde::Serialize;

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;

/// A texture the game's weather names, which a hand-set keyframe may be given.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWeatherTexture {
  /// The reference, and what it resolves to beside the level.
  pub texture: LevelTextureReference,
  /// How many keyframes of the cycles and effects name it.
  pub uses: u32,
}
