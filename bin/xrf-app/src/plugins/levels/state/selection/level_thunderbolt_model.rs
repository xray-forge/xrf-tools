use serde::Serialize;
use xrf_material::XraySurfaceDraw;

use crate::plugins::levels::state::selection::level_weather_model::LevelWeatherModel;

/// A thunderbolt's `lightning_model`, and how the shader it names composites it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelThunderboltModel {
  /// The model as its file names it, under `meshes`.
  pub name: String,
  pub mesh: LevelWeatherModel,
  pub draw: XraySurfaceDraw,
}
