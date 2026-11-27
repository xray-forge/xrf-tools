use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderSettings, RenderUpscalingSettings, RenderViewFeatures};

use crate::game_input_settings::GameInputSettings;
use crate::game_window_settings::GameWindowSettings;

/// What the game draws with, how its window opens and how the player's input reads, as a preset sets them and the
/// settings file and the arguments change them. Each group is the renderer's own contract where it has one, so a renderer setting is a game setting.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSettings {
  /// What every frame draws with: the backend, the GPU, the textures' filtering and quality.
  pub render: RenderSettings,
  /// What the game's view draws: its lighting, shadows, reflections, weather and the rest.
  pub view: RenderViewFeatures,
  /// What the view's scene is drawn at and how its frame is upscaled.
  pub upscaling: RenderUpscalingSettings,
  pub window: GameWindowSettings,
  /// How the player's input reads: the mouse's sensitivity and the keys the player bound.
  pub input: GameInputSettings,
}
