use glam::Vec3;
use xrf_engine_target::XrayEngine;

use crate::camera::camera_view::CameraView;
use crate::contract::feature::lighting::lights::render_lights_settings::RenderLightsSettings;
use crate::feature::scene::drawable::casters::light_casters::LightCasters;
use crate::scene::level::level_time::LevelTime;

/// What a view's frame hands its scene's local lights: the camera and settings, the engine whose rules give out their
/// shafts, what has their shadow faces drawn again, and how its lights are lit.
#[derive(Clone, Copy)]
pub struct LightsFrame {
  pub camera: CameraView,
  pub settings: RenderLightsSettings,
  pub engine: XrayEngine,
  /// The progressive meshes' `start` and `end` screen areas, which a shadowed light fades between.
  pub lod: (f32, f32),
  /// The wind's amplitude the trees lean by, which has a face over them drawn again; a face's last draw is timed by
  /// `time`.
  pub sway_amplitude: f32,
  /// What the drawable kinds beside the static scene cast into the faces of the lights near the eye (the grass's cards).
  pub casters: Option<LightCasters>,
  /// Whether the view lights in linear light, the lights' authored colours decoded (`RenderLightModel::Physical`).
  pub is_physical: bool,
  /// Whether the lights' specular weight follows their light (`RenderViewOptions::is_specular_linear`).
  pub is_specular_linear: bool,
  /// The weather's ambient in the engine's units, which each light is anchored over at the anchored levels
  /// (`anchored_light`); none elsewhere.
  pub anchor: Option<Vec3>,
  /// Whether the lamps fall off by the windowed inverse square (`RenderViewOptions::is_falloff_inverse_square`), whose
  /// intensity follows the square of a light's range.
  pub is_falloff_inverse_square: bool,
  /// The scale a local light in the engine's units is written at: its unit (`LightUnits::local`) by the lights' gain
  /// (`LightUnits::light_gain`) at the pre-exposure.
  pub light_scale: f32,
  /// The level's clock, which the lights' animations and shafts' noise move by and their faces' draws are timed by.
  pub time: LevelTime,
}

impl LightsFrame {
  /// Whether it casts the lights' shadows, its faces asked for.
  pub fn is_shadowing(&self) -> bool {
    self.settings.is_enabled && self.settings.is_shadowed
  }
}
