use glam::Vec4;

use crate::pass::pre_exposure::PreExposure;
use crate::pass::sky_harmonics::SkyHarmonics;

/// What a viewport's frame adds to its lighting before the uniform is written: the exposure, what is up of the sky,
/// and the clocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightingFrame {
  /// Whether the exposure adapts this frame.
  pub is_adapting: bool,
  /// The scale the frame's light is written at, and the last frame's.
  pub pre_exposure: PreExposure,
  /// How far from the first keyframe's sky to the second's, moved to the one up while the other is still going up.
  pub sky_blend: f32,
  /// Whether both irradiance cubes are up, else the lighting's stand-in lights the hemisphere.
  pub is_irradiance_up: bool,
  /// Seconds the clouds have drifted.
  pub clouds_time: f32,
  /// The sun's sprite colour times how far it has faded in, then half its side as a share of the distance it stands
  /// at; zero where none is drawn.
  pub sun_sprite: Vec4,
  /// The sky's irradiance the physical light lights the hemisphere by; none under the engine's.
  pub sky_harmonics: SkyHarmonics,
}
