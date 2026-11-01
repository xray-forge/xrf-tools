use glam::{Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::feature::lighting::render_shadow_settings::RenderShadowSettings;
use crate::feature::scene::drawable::per_frame_casters::PerFrameCasters;
use crate::scene::level::shadow::shadow_sway::ShadowSway;
use crate::scene::static_scene::static_scene::StaticScene;

/// What a level's frame hands its shadows as they ready: the scene and camera, the settings, the sun, the sway and what
/// moves every frame.
pub struct ShadowFrame<'a> {
  pub scene: &'a StaticScene,
  pub camera: &'a CameraView,
  pub settings: &'a RenderShadowSettings,
  /// Where the sun's light travels.
  pub sun_direction: Vec3,
  /// Whether the sun lights anything: one of no colour (a weather's night between the sun and the moon) casts no shadow
  /// to draw.
  pub is_sunlit: bool,
  /// What a camera off the viewport draws the static scene with: textured, bumped, the hemisphere's strength.
  pub switches: Vec4,
  /// How the trees sway, which has a still map drawn again where it shows.
  pub sway: ShadowSway<'a>,
  /// What the drawable kinds beside the static scene cast into the sun's cascades every frame (the grass's cards).
  pub casters: &'a [PerFrameCasters],
}
