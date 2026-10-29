use std::collections::HashMap;

use glam::Vec3;
use xrf_engine_target::XrayEngine;

use crate::camera::camera_view::CameraView;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::static_scene::static_scene::StaticScene;

/// What a view's frame hands its scene's local lights: the camera and settings, the engine whose rules give out their
/// shafts, what has their shadow faces drawn again, and the campfires fading their idle lights.
pub struct LightsFrame<'a> {
  pub camera: &'a CameraView,
  pub settings: &'a RenderLightsSettings,
  pub engine: XrayEngine,
  /// The progressive meshes' `start` and `end` screen areas, which a shadowed light fades between.
  pub lod: (f32, f32),
  /// The static scene, whose changes within a light's reach have its faces drawn again.
  pub statics: &'a StaticScene,
  /// How the trees sway, which has a face over them drawn again.
  pub sway: &'a ShadowSway<'a>,
  /// The metres about the eye the grass's cards stand in, where they cast the sun's shadow this frame, which the
  /// lights near the eye cast theirs from too.
  pub grass_reach: Option<f32>,
  /// How much of each campfire's idle light shows, by its id.
  pub campfire_shares: &'a HashMap<u16, f32>,
  /// Whether the view is its scene's first, which draws the lights' faces with its own grass cards.
  pub is_scene_first: bool,
  /// Whether the view lights in linear light, the lights' authored colours decoded (`RenderLightModel::Physical`).
  pub is_physical: bool,
  /// Whether the lights' specular weight follows their light (`RenderViewOptions::is_specular_linear`).
  pub is_specular_linear: bool,
  /// The weather's ambient in the engine's units, which each light is anchored over at the anchored levels
  /// (`anchored_light`); none elsewhere.
  pub anchor: Option<Vec3>,
  /// The scale a local light in the engine's units is written at: its unit (`LightUnits::local`) by the lights' gain
  /// (`LightUnits::light_gain`) at the pre-exposure.
  pub light_scale: f32,
  /// The scene's seconds, which the lights' animations and shafts' noise move by.
  pub time: f32,
}
