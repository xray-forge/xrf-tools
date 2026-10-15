use glam::{Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::static_scene::static_scene::StaticScene;

/// What a level's frame hands its shadows as they ready: the scene and camera, the settings, the sway, the grass and
/// the cull's counts.
pub struct ShadowFrame<'a> {
  pub scene: &'a StaticScene,
  pub camera: &'a CameraView,
  pub settings: &'a RenderShadowSettings,
  /// Where the sun's light travels.
  pub sun_direction: Vec3,
  /// What a camera off the viewport draws the static scene with: textured, bumped, the hemisphere's strength.
  pub switches: Vec4,
  /// How the trees sway, which has a still map drawn again where it shows.
  pub sway: ShadowSway<'a>,
  /// Metres from the camera the grass casts the sun's shadow to through the cascades (`GrassCards`), none where it
  /// casts none there.
  pub grass_reach: Option<f32>,
}
