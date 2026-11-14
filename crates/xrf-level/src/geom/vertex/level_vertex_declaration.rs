use crate::geom::buffers::level_geom_vertex_element::LevelGeomVertexElement;
use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;

/// The vertex declarations xrLC stores a level's static geometry in (`xrLC/OGF_Face_Save.cpp:46`).
pub struct LevelVertexDeclaration;

impl LevelVertexDeclaration {
  /// `r1_decl_lmap`, 32 bytes: position, normal, tangent, binormal, base and lightmap coordinates.
  pub fn get_lightmapped() -> Vec<LevelGeomVertexElement> {
    vec![
      Self::element(0, LevelVertexLayout::KIND_FLOAT3, LevelVertexLayout::USAGE_POSITION, 0),
      Self::element(12, LevelVertexLayout::KIND_D3DCOLOR, LevelVertexLayout::USAGE_NORMAL, 0),
      Self::element(
        16,
        LevelVertexLayout::KIND_D3DCOLOR,
        LevelVertexLayout::USAGE_TANGENT,
        0,
      ),
      Self::element(
        20,
        LevelVertexLayout::KIND_D3DCOLOR,
        LevelVertexLayout::USAGE_BINORMAL,
        0,
      ),
      Self::element(24, LevelVertexLayout::KIND_SHORT2, LevelVertexLayout::USAGE_TEXCOORD, 0),
      Self::element(28, LevelVertexLayout::KIND_SHORT2, LevelVertexLayout::USAGE_TEXCOORD, 1),
    ]
  }

  /// `r1_decl_vert`, 32 bytes: position, normal, tangent, binormal, colour and base coordinate.
  pub fn get_vertex_lit() -> Vec<LevelGeomVertexElement> {
    vec![
      Self::element(0, LevelVertexLayout::KIND_FLOAT3, LevelVertexLayout::USAGE_POSITION, 0),
      Self::element(12, LevelVertexLayout::KIND_D3DCOLOR, LevelVertexLayout::USAGE_NORMAL, 0),
      Self::element(
        16,
        LevelVertexLayout::KIND_D3DCOLOR,
        LevelVertexLayout::USAGE_TANGENT,
        0,
      ),
      Self::element(
        20,
        LevelVertexLayout::KIND_D3DCOLOR,
        LevelVertexLayout::USAGE_BINORMAL,
        0,
      ),
      Self::element(24, LevelVertexLayout::KIND_D3DCOLOR, LevelVertexLayout::USAGE_COLOR, 0),
      Self::element(28, LevelVertexLayout::KIND_SHORT2, LevelVertexLayout::USAGE_TEXCOORD, 0),
    ]
  }

  /// `x_decl_vert`, 12 bytes: the fast path's position alone.
  pub fn get_fast_path() -> Vec<LevelGeomVertexElement> {
    vec![Self::element(
      0,
      LevelVertexLayout::KIND_FLOAT3,
      LevelVertexLayout::USAGE_POSITION,
      0,
    )]
  }

  /// A tree's (`xrMU_Model_export_geometry.cpp:14`), 32 bytes: position, normal, tangent, binormal, and the base
  /// coordinate with its wind terms.
  pub fn get_tree() -> Vec<LevelGeomVertexElement> {
    vec![
      Self::element(0, LevelVertexLayout::KIND_FLOAT3, LevelVertexLayout::USAGE_POSITION, 0),
      Self::element(12, LevelVertexLayout::KIND_D3DCOLOR, LevelVertexLayout::USAGE_NORMAL, 0),
      Self::element(
        16,
        LevelVertexLayout::KIND_D3DCOLOR,
        LevelVertexLayout::USAGE_TANGENT,
        0,
      ),
      Self::element(
        20,
        LevelVertexLayout::KIND_D3DCOLOR,
        LevelVertexLayout::USAGE_BINORMAL,
        0,
      ),
      Self::element(24, LevelVertexLayout::KIND_SHORT4, LevelVertexLayout::USAGE_TEXCOORD, 0),
    ]
  }

  /// An element of stream 0 read by `D3DDECLMETHOD_DEFAULT`.
  const fn element(offset: u16, kind: u8, usage: u8, usage_index: u8) -> LevelGeomVertexElement {
    LevelGeomVertexElement {
      stream: 0,
      offset,
      kind,
      method: 0,
      usage,
      usage_index,
    }
  }
}
