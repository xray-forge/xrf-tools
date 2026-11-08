use crate::contract::feature::weather::water::render_enhanced_water_settings::RenderEnhancedWaterSettings;
use crate::feature::weather::water::enhanced::water_weather::WaterWeather;
use crate::scene::level::view::view_time::ViewTime;

/// How far the enhanced water's maps have scrolled and its rain ripples have run, each frame's seconds at that frame's
/// flow and at the pace its wind and rain set. Summed rather than multiplied out, so a change of wind, rain, flow or
/// the weather's clock changes how fast the maps move from then on, never where they stand; summed in `f64`, so a
/// frame's few milliseconds still add to a day's sum, and read through `EnhancedWaterScroll`, which wraps each read by
/// its map's repeat.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct WaterFlow {
  /// Seconds at the flow, which the caustics scroll by.
  pub seconds: f64,
  /// Seconds at the normals' pace, `clamp(0.97 * wind, 0.45, 0.97)` with its floor at the calm flow.
  pub waves: f64,
  /// Seconds at the heights' pace, `clamp(0.67 * wind, 0.3, 0.67)` with its floor at the calm flow.
  pub heights: f64,
  /// How far the wind layer has scrolled: seconds at the wind's strength, along the way it blew then.
  pub gusts: [f64; 2],
  /// Seconds at the rain ripples' pace, `RIPPLE_RAIN` of the rain's density; the flow leaves them, as the rain does.
  pub ripples: f64,
}

impl WaterFlow {
  /// The share of its strongest a wind's velocity is taken for.
  const WIND_SHARE: f64 = 0.001;
  /// The rain ripples' pace by the rain's density: its density 1.6 times over, held between these.
  const RIPPLE_RAIN: (f64, f64, f64) = (1.6, 0.65, 1.0);

  /// Moves the maps on by the seconds since the view's last frame, never back, at this frame's settings and weather.
  pub(crate) fn advance(
    &mut self,
    time: ViewTime,
    settings: &RenderEnhancedWaterSettings,
    WaterWeather { wind, rain }: WaterWeather,
  ) {
    let step: f64 = time.get_advance();
    let elapsed: f64 = step * f64::from(settings.flow);
    let strength: f64 = (f64::from(wind.velocity) * Self::WIND_SHARE).clamp(0.0, 1.0);
    let calm_flow: f64 = f64::from(settings.calm_flow);
    let pace = |most: f64, least: f64| (most * strength).max(least * calm_flow).min(most);
    let (rain_scale, rain_least, rain_most): (f64, f64, f64) = Self::RIPPLE_RAIN;

    self.seconds += elapsed;
    self.waves += elapsed * pace(0.97, 0.45);
    self.heights += elapsed * pace(0.67, 0.3);
    self.ripples += step * (f64::from(rain) * rain_scale).clamp(rain_least, rain_most);
    // `Wind_Dir`: the way it blows, turned a quarter.
    let (sine, cosine): (f64, f64) = (f64::from(wind.direction) + std::f64::consts::FRAC_PI_2).sin_cos();

    self.gusts[0] += elapsed * strength * cosine;
    self.gusts[1] += elapsed * strength * sine;
  }
}
