use std::sync::Arc;

use crate::camera::camera_frame::CameraFrame;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_scene_id::RenderSceneId;
use crate::lighting::render_lighting::RenderLighting;

/// What the world gives a viewport's frame: the scene it asks to show, where its camera stands, and one frame's
/// environment.
#[derive(Default)]
pub struct RenderViewFrame {
  /// The scene the viewport asks to show, none for nothing.
  pub scene: Option<RenderSceneId>,
  pub camera: CameraFrame,
  pub lighting: RenderLighting,
  /// The level's weather the lighting was played from, whose rain, bolts and models the frame draws.
  pub weather: Option<Arc<RenderLevelWeather>>,
  /// Game seconds the weather's clock runs a real second.
  pub clock_rate: f32,
}
