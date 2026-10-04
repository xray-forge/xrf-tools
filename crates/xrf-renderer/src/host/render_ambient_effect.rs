use std::time::Duration;

use crate::host::render_wind_blast::RenderWindBlast;

/// An effect an ambient plays near the camera, `CEnvAmbient::SEffect`: a particle system standing where the camera
/// stood plus an offset, and the wind it brings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderAmbientEffect {
  /// The `particles.xr` effect or group it plays.
  pub particles: String,
  /// How long it plays before it is stopped, `life_time`.
  pub life_time: Duration,
  /// Where it stands from the camera, in engine space.
  pub offset: [f32; 3],
  /// How often the wind gusts while it plays, `wind_gust_factor`: the frequency of the wind's noise.
  pub wind_gust_factor: f32,
  pub wind_blast: RenderWindBlast,
}
