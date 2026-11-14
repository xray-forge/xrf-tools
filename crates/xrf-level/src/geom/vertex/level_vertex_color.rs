use serde::{Deserialize, Serialize};

/// A vertex's baked colour, the bytes of its `D3DCOLOR` element: the red, green and blue light, and in the fourth byte
/// the sun term, as xrLC writes it (`r1v_vert`, `xrLC/OGF_Face_Save.cpp:120`) and the renderers read it
/// (`deffer_base_flat.vs`). The hemisphere term rides in the normal's fourth byte instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub sun: u8,
}
