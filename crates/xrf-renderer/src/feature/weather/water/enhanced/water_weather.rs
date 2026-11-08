use crate::lighting::wind::render_wind::RenderWind;

/// The weather the enhanced water's maps move by this frame: the wind, and the falling rain's density, none once it
/// stops.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaterWeather {
  pub wind: RenderWind,
  pub rain: f32,
}
