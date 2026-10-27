use xrf_material::XraySurfaceDraw;

use crate::weather::render::LevelWeatherModel;

/// A thunderbolt's `lightning_model`, and how the shader it names composites it.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelThunderboltModel {
  /// The model as its file names it, under `meshes`.
  pub name: String,
  pub mesh: LevelWeatherModel,
  pub draw: XraySurfaceDraw,
}
