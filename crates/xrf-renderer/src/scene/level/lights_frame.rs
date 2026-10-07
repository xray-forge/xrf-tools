use std::collections::HashMap;

use xrf_engine_target::XrayEngine;

use crate::camera::camera_view::CameraView;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::scene::level::shadow_sway::ShadowSway;

/// What a view's frame hands its scene's local lights: the camera and settings, the engine whose rules give out their
/// shafts, what has their shadow faces drawn again, and the campfires fading their idle lights.
pub struct LightsFrame<'a> {
  pub camera: &'a CameraView,
  pub settings: &'a RenderLightsSettings,
  pub engine: XrayEngine,
  /// The progressive meshes' `start` and `end` screen areas, which a shadowed light fades between.
  pub lod: (f32, f32),
  /// What the scene holds, which a face drawn with less is drawn again for.
  pub contents: usize,
  /// How the trees sway, which has a face over them drawn again.
  pub sway: &'a ShadowSway<'a>,
  /// How much of each campfire's idle light shows, by its id.
  pub campfire_shares: &'a HashMap<u16, f32>,
}
