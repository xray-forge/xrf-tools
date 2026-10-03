use xrf_material::XraySurfaceDraw;

use crate::host::render_weather_model::RenderWeatherModel;

/// A bolt's `lightning_model`: its mesh in engine space, from its top down its length of one, and how its shader
/// composites it.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderThunderboltModel {
  pub mesh: RenderWeatherModel,
  pub draw: XraySurfaceDraw,
}
