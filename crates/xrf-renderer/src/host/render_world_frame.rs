use std::sync::Arc;

use glam::Vec3;

use crate::camera::camera_frame::CameraFrame;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_scene_update::RenderSceneUpdate;
use crate::host::render_streaming_progress::RenderStreamingProgress;
use crate::lighting::render_lighting::RenderLighting;

/// What the world gives a viewport's frame: the level it shows and what changed in it, where its camera stands, and one
/// frame's environment.
#[derive(Default)]
pub struct RenderWorldFrame {
  /// The level the viewport shows, whose scene the updates go to.
  pub level: Option<Arc<dyn RenderLevelSource>>,
  pub updates: Vec<RenderSceneUpdate>,
  pub streaming: RenderStreamingProgress,
  pub camera: CameraFrame,
  pub lighting: RenderLighting,
  /// The level's weather the lighting was played from, whose rain, bolts and models the frame draws.
  pub weather: Option<Arc<RenderLevelWeather>>,
  /// Game seconds the weather's clock runs a real second.
  pub clock_rate: f32,
  /// Every skinned object's bones as segments in renderer space, child then parent, which the skeleton overlay draws.
  pub skeleton_segments: Vec<(Vec3, Vec3)>,
}
