use glam::Vec4;

use crate::feature::image::exposure::reading::exposure_decisions::ExposureDecisions;
use crate::feature::weather::sky::irradiance::sky_harmonics::SkyHarmonics;
use crate::lighting::unit::daylight::Daylight;
use crate::lighting::unit::light_units::LightUnits;

/// What a viewport's frame adds to its lighting before the uniform is written: the exposure, how the present shows the
/// frame, what is up of the sky, and the clocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightingFrame {
  /// What the view's exposure decided for this frame.
  pub exposure: ExposureDecisions,
  /// How the present shows the frame (`DisplayMode::get_display`), as the view decided it for this frame.
  pub display: Vec4,
  /// The cd/m² a light of one in the engine's units stands for, the weather's and the local lights'.
  pub light_units: LightUnits,
  /// The world's light the physical levels light the sun by; none at the authored levels or under the engine's light.
  pub daylight: Option<Daylight>,
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
