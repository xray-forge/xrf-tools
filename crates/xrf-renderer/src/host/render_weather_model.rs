/// A model the weather draws, rain's splash or a thunderbolt: its mesh and the texture it draws with.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderWeatherModel {
  pub texture: String,
  /// Three floats a vertex, in engine space.
  pub positions: Vec<f32>,
  /// Two floats a vertex.
  pub uvs: Vec<f32>,
  /// A triangle list.
  pub indices: Vec<u16>,
}
