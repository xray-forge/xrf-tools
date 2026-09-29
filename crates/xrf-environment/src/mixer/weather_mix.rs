use serde::Serialize;

/// A cycle mixed at one time of day, `CEnvDescriptorMixer` after `lerp`: what the frame is lit and fogged by.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherMix {
  /// Seconds since midnight, as asked.
  pub time: f32,
  /// Where it was seen from, in engine space, as asked.
  pub view: [f32; 3],
  /// The keyframes either side, by index.
  pub keyframes: [usize; 2],
  /// How far from the first to the second.
  pub weight: f32,
  pub sky_color: [f32; 3],
  /// Radians.
  pub sky_rotation: f32,
  pub clouds_color: [f32; 4],
  /// Radians.
  pub clouds_rotation: f32,
  pub far_plane: f32,
  pub fog_color: [f32; 3],
  pub fog_density: f32,
  pub fog_distance: f32,
  pub fog_near: f32,
  pub fog_far: f32,
  pub hemi_color: [f32; 4],
  pub sun_color: [f32; 3],
  pub ambient_color: [f32; 3],
  /// Normalised, the way sunlight travels, in engine space.
  pub sun_direction: [f32; 3],
  pub water_intensity: f32,
  pub tree_amplitude: f32,
  pub tree_speed: f32,
  pub tree_rotation: f32,
  pub tree_wave: [f32; 3],
}
