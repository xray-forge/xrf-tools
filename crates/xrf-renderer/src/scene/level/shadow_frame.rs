use glam::Vec3;

use crate::camera::camera_view::CameraView;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::pass::static_cull_params::StaticCullParams;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::texture::texture_cache::TextureCache;

/// What a level's frame hands its sun shadow: the scene and camera, the settings, and what its culls bind besides
/// their own lists.
pub struct ShadowFrame<'a> {
  pub scene: &'a StaticScene,
  pub camera: &'a CameraView,
  pub settings: &'a RenderShadowSettings,
  /// Where the sun's light travels.
  pub sun_direction: Vec3,
  /// How the trees sway, which has a still map drawn again where it shows.
  pub sway: ShadowSway<'a>,
  pub cull_params: &'a wgpu::Buffer,
  pub params: &'a StaticCullParams,
  /// The camera's depth pyramid and occlusion view, which a shadow's cull binds unread.
  pub pyramid: &'a wgpu::TextureView,
  pub occlusion: &'a wgpu::Buffer,
  pub targets_epoch: u64,
  pub textures: &'a TextureCache,
}
