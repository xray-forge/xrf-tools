use xrf_level::{DetailModel, DetailVertex};
use xrf_renderer::RenderWeatherModel;

use crate::parts::LevelTextureReference;

/// A detail model the weather draws, rain's splash or a thunderbolt: its mesh and the texture it draws with.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelWeatherModel {
  pub texture: LevelTextureReference,
  /// Three floats a vertex, in engine space.
  pub positions: Vec<f32>,
  /// Two floats a vertex.
  pub uvs: Vec<f32>,
  /// A triangle list.
  pub indices: Vec<u16>,
}

impl LevelWeatherModel {
  /// The model's mesh, its texture located as the level locates its own.
  pub fn of(model: &DetailModel, texture: LevelTextureReference) -> Self {
    Self {
      indices: model.indices.clone(),
      positions: model
        .vertices
        .iter()
        .flat_map(|vertex: &DetailVertex| [vertex.position.x, vertex.position.y, vertex.position.z])
        .collect(),
      texture,
      uvs: model
        .vertices
        .iter()
        .flat_map(|vertex: &DetailVertex| [vertex.u, vertex.v])
        .collect(),
    }
  }
}

impl From<LevelWeatherModel> for RenderWeatherModel {
  /// The model as the renderer takes it, its texture by reference.
  fn from(model: LevelWeatherModel) -> Self {
    Self {
      texture: model.texture.reference,
      positions: model.positions,
      uvs: model.uvs,
      indices: model.indices,
    }
  }
}
