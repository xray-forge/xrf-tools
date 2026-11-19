use serde::{Deserialize, Serialize};

/// A vertex's baked colour, its `D3DCOLOR` element: red, green and blue light, and the sun term in the fourth byte, as
/// xrLC writes it (`r1v_vert`) and the renderers read it (`deffer_base_flat.vs`). The hemisphere term rides in the
/// normal's fourth byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub sun: u8,
}
