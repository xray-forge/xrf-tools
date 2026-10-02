use crate::host::render_weather_model::RenderWeatherModel;

/// What rain is drawn with, as `dxRainRender` loads it: the streaks' texture and the splash's model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderRain {
  pub streak: String,
  /// None where the model is not there, which draws no splashes.
  pub drop: Option<RenderWeatherModel>,
}
