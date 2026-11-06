use serde::{Deserialize, Serialize};

/// A vertex's baked colour, the red, green and blue bytes of its `D3DCOLOR` element; the element's fourth byte is the
/// hemisphere term, which the vertex keeps apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}
