use crate::contract::render_enhanced_water_settings::RenderEnhancedWaterSettings;
use crate::lighting::render_wind::RenderWind;

/// How far the enhanced water's maps have scrolled, each frame's seconds at that frame's flow and at the pace its wind
/// sets. Summed rather than multiplied out, so a change of wind,
/// flow or the weather's clock changes how fast the maps move from then on, never where they stand.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaterFlow {
  /// Seconds at the flow, which the caustics scroll by.
  pub seconds: f32,
  /// Seconds at the normals' pace, `clamp(0.97 * wind, 0.45, 0.97)` with its floor at the calm flow.
  pub waves: f32,
  /// Seconds at the heights' pace, `clamp(0.67 * wind, 0.3, 0.67)` with its floor at the calm flow.
  pub heights: f32,
  /// How far the wind layer has scrolled: seconds at the wind's strength, along the way it blew then.
  pub gusts: [f32; 2],
  /// The clock the last frame was drawn at.
  clock: Option<f32>,
}

impl WaterFlow {
  /// The share of its strongest a wind's velocity is taken for.
  const WIND_SHARE: f32 = 0.001;

  /// Moves the maps on by the seconds since the last frame, at this frame's settings and wind.
  pub fn advance(&mut self, time: f32, settings: &RenderEnhancedWaterSettings, wind: RenderWind) {
    let elapsed: f32 = self.clock.map_or(0.0, |clock| (time - clock).max(0.0)) * settings.flow;
    let strength: f32 = (wind.velocity * Self::WIND_SHARE).clamp(0.0, 1.0);
    let pace = |most: f32, least: f32| (most * strength).max(least * settings.calm_flow).min(most);

    self.clock = Some(time);
    self.seconds += elapsed;
    self.waves += elapsed * pace(0.97, 0.45);
    self.heights += elapsed * pace(0.67, 0.3);
    // `Wind_Dir`: the way it blows, turned a quarter.
    let (sine, cosine): (f32, f32) = (wind.direction + std::f32::consts::FRAC_PI_2).sin_cos();

    self.gusts[0] += elapsed * strength * cosine;
    self.gusts[1] += elapsed * strength * sine;
  }
}
