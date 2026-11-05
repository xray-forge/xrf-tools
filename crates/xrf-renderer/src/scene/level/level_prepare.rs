use crate::camera::camera_view::CameraView;
use crate::lighting::wind::ambient_gust::AmbientGust;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::view::view_time::ViewTime;
use crate::scene::level::wetness::level_wetness::LevelWetness;
use crate::scene::static_scene::draw::static_sight::StaticSight;

/// What the level gives every unit readying a view's frame, taken from the scene once that frame: its clock as the view
/// reads it, the wind its ambient effects blow, how wet it is, what of it the view may see, and how sheltered the
/// driving camera stands. The scene's stores (its statics, grass, lights, particles and trace) stay the scene's, read by
/// the units drawing them; what the level places for every view to read, such as fog volumes, joins these values.
#[derive(Clone, Copy)]
pub struct LevelPrepare {
  /// How long the level has lived at this frame, on the frames' clock, and at the view's last frame: what its clouds,
  /// sway, water, lights and effects move by, each by its own phase of it, and what is stepped on from frame to frame
  /// steps by.
  pub time: ViewTime,
  pub gust: AmbientGust,
  pub wetness: LevelWetness,
  /// What of the scene drawn by passes of its own the view may see: none of the water's or the surface passes' work
  /// runs for what is out of sight.
  pub sight: StaticSight,
  /// The driving camera's smoothed hemi, none where it is not estimated.
  pub camera_hemi: Option<f32>,
}

impl LevelPrepare {
  /// What `scene` gives a view seeing it through `view` this frame, its last frame drawn at `previous`.
  pub fn of(scene: &LevelScene, view: &CameraView, previous: Option<LevelTime>) -> Self {
    Self {
      time: ViewTime {
        now: scene.get_time(),
        previous,
      },
      gust: scene.gust,
      wetness: scene.wetness,
      sight: scene.statics.get_sight(&view.get_frustum()),
      camera_hemi: scene.camera_hemi,
    }
  }
}
