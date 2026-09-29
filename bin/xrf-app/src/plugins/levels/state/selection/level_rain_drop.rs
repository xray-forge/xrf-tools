use serde::Serialize;
use xrf_level::{DetailModel, DetailVertex};

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;

/// The splash a raindrop leaves where it lands, `dm\rain.dm`: its mesh and the texture it draws with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelRainDrop {
  pub texture: LevelTextureReference,
  /// Three floats a vertex, in engine space.
  pub positions: Vec<f32>,
  /// Two floats a vertex.
  pub uvs: Vec<f32>,
  /// A triangle list.
  pub indices: Vec<u16>,
}

impl LevelRainDrop {
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
