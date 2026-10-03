use serde::Serialize;

use crate::weather::WeatherDescriptor;

/// A cycle mixed at one time of day, `CEnvDescriptorMixer` after `lerp`: what the frame is lit and fogged by.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherMix {
  /// Seconds since midnight, as asked.
  pub time: f32,
  /// Where it was seen from, in engine space, as asked.
  pub view: [f32; 3],
  /// The times of day of the keyframes either side.
  pub between: [f32; 2],
  /// How far from the first to the second.
  pub weight: f32,
  /// How many of the level's modifiers reach the view.
  pub modifiers: u32,
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
  /// `sun_shafts_intensity`: how dense the light shafts through the sun's shadow are.
  pub sun_shafts_intensity: f32,
  pub rain_density: f32,
  pub rain_color: [f32; 3],
  pub wind_velocity: f32,
  /// Radians.
  pub wind_direction: f32,
  pub tree_amplitude: f32,
  pub tree_speed: f32,
  pub tree_rotation: f32,
  pub tree_wave: [f32; 3],
  /// The collection struck with: the first keyframe's before halfway, the second's after.
  pub thunderbolt_collection: Option<String>,
  /// Seconds.
  pub thunderbolt_period: f32,
  /// Seconds.
  pub thunderbolt_duration: f32,
}

impl WeatherMix {
  /// The mix as one keyframe, which a keyframe set by hand starts from: the heavier keyframe's textures and whatever
  /// else is not mixed, the sun where it stands.
  pub fn to_descriptor(&self, heavier: &WeatherDescriptor) -> WeatherDescriptor {
    WeatherDescriptor {
      ambient_color: self.ambient_color,
      clouds_color: self.clouds_color,
      clouds_rotation: self.clouds_rotation,
      far_plane: self.far_plane,
      fog_color: self.fog_color,
      fog_density: self.fog_density,
      fog_distance: self.fog_distance,
      hemi_color: self.hemi_color,
      rain_color: self.rain_color,
      rain_density: self.rain_density,
      sky_color: self.sky_color,
      sky_rotation: self.sky_rotation,
      sun_color: self.sun_color,
      sun_direction: Some(self.sun_direction),
      thunderbolt_collection: self.thunderbolt_collection.clone(),
      thunderbolt_duration: self.thunderbolt_duration,
      thunderbolt_period: self.thunderbolt_period,
      time: self.time.round() as u32 % crate::weather::WeatherTime::DAY,
      tree_amplitude: self.tree_amplitude,
      tree_rotation: self.tree_rotation,
      tree_speed: self.tree_speed,
      tree_wave: self.tree_wave,
      water_intensity: self.water_intensity,
      sun_shafts_intensity: self.sun_shafts_intensity,
      wind_direction: self.wind_direction,
      wind_velocity: self.wind_velocity,
      ..heavier.clone()
    }
  }
}
