use crate::camera::camera_view::CameraView;
use crate::lighting::wind::ambient_gust::AmbientGust;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::view::view_time::ViewTime;
use crate::scene::level::wetness::level_wetness::LevelWetness;
use crate::scene::static_scene::draw::static_sight::StaticSight;

/// Per-frame level state every unit preparing a view reads: clock, ambient wind, wetness, sight and camera hemi.
///
/// The scene's stores (statics, grass, lights, particles, trace) stay on the scene, read by the units that draw them.
#[derive(Clone, Copy)]
pub(crate) struct LevelPrepare {
  /// The level's clock at this frame and at the view's last frame, which animations and per-frame steps read.
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
  /// Takes the state for `view` from `scene`, the view's last frame drawn at `previous`.
  pub(crate) fn of(scene: &LevelScene, view: &CameraView, previous: Option<LevelTime>) -> Self {
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
