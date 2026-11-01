use crate::camera::camera_view::CameraView;
use crate::lighting::wind::ambient_gust::AmbientGust;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::wetness::level_wetness::LevelWetness;
use crate::scene::static_scene::draw::static_sight::StaticSight;

/// What the level gives every unit readying a view's frame, taken from the scene once that frame: its clock, the wind
/// its ambient effects blow, how wet it is, what of it the view may see, and how sheltered the driving camera stands.
/// The scene's stores (its statics, grass, lights, particles and trace) stay the scene's, read by the units drawing
/// them; what the level places for every view to read, such as fog volumes, joins these values.
#[derive(Clone, Copy)]
pub struct LevelPrepare {
  /// Seconds the level has lived at this frame, on the frames' clock: what its clouds, sway, water, lights and effects
  /// move by.
  pub time: f32,
  pub gust: AmbientGust,
  pub wetness: LevelWetness,
  /// What of the scene drawn by passes of its own the view may see: none of the water's or the surface passes' work
  /// runs for what is out of sight.
  pub sight: StaticSight,
  /// The driving camera's smoothed hemi, none where it is not estimated.
  pub camera_hemi: Option<f32>,
}

impl LevelPrepare {
  /// What `scene` gives a view seeing it through `view` this frame.
  pub fn of(scene: &LevelScene, view: &CameraView) -> Self {
    Self {
      time: scene.get_time().as_secs_f32(),
      gust: scene.gust,
      wetness: scene.wetness,
      sight: scene.statics.get_sight(&view.get_frustum()),
      camera_hemi: scene.camera_hemi,
    }
  }
}
