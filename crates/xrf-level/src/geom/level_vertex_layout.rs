use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::geom::level_geom_vertex_buffer::LevelGeomVertexBuffer;
use crate::geom::level_geom_vertex_element::LevelGeomVertexElement;

/// Where each attribute of a level vertex sits, and how to turn its bytes back into what xrLC had.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexLayout {
  pub stride: u32,
  position: u16,
  normal: Option<u16>,
  tangent: Option<u16>,
  binormal: Option<u16>,
  texture_coordinate: Option<LevelVertexCoordinate>,
  lightmap_coordinate: Option<u16>,
  color: Option<u16>,
}

/// Where a texture coordinate sits and how wide the element holding it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexCoordinate {
  offset: u16,
  is_tree: bool,
}

impl LevelVertexLayout {
  /// `D3DDECLUSAGE_POSITION`.
  const USAGE_POSITION: u8 = 0;
  /// `D3DDECLUSAGE_NORMAL`.
  const USAGE_NORMAL: u8 = 3;
  /// `D3DDECLUSAGE_TEXCOORD`.
  const USAGE_TEXCOORD: u8 = 5;
  /// `D3DDECLUSAGE_TANGENT`.
  const USAGE_TANGENT: u8 = 6;
  /// `D3DDECLUSAGE_BINORMAL`.
  const USAGE_BINORMAL: u8 = 7;
  /// `D3DDECLUSAGE_COLOR`.
  const USAGE_COLOR: u8 = 10;

  /// `D3DDECLTYPE_FLOAT3`.
  const KIND_FLOAT3: u8 = 2;
  /// `D3DDECLTYPE_D3DCOLOR`.
  const KIND_D3DCOLOR: u8 = 4;
  /// `D3DDECLTYPE_SHORT2`.
  const KIND_SHORT2: u8 = 6;
  /// `D3DDECLTYPE_SHORT4`.
  const KIND_SHORT4: u8 = 7;

  /// What a base coordinate is divided by: `unpack_tc_base` scales by `32.0 / 32768.0` (`common_functions.h`).
  pub const BASE_QUANT: f32 = 1024.0;

  /// The same for a lightmap coordinate: `unpack_tc_lmap` scales by `1.0 / 32768.0` (`common_functions.h`).
  pub const LIGHTMAP_QUANT: f32 = 32768.0;

  /// The same for a tree: `FTreeVisual_quant` is `32768 / FTreeVisual_tile`, and the tile is 16 (`FTreeVisual.h`).
  pub const TREE_QUANT: f32 = 2048.0;

  /// What a byte riding in a tangent or binormal alpha is divided by, which is the engine's 255 from
  /// `unpack_D3DCOLOR` rather than the 255.5 the compiler wrote it with.
  const BASE_FRACTION: f32 = 255.0;

  /// Reads a buffer's declaration into the offsets a decoder needs.
  ///
  /// # Errors
  ///
  /// Returns an error when the declaration names a type `D3DDECLTYPE` does not, carries no position, or stores an
  /// attribute in a type xrLC never writes it as - all of which would otherwise decode into plausible nonsense.
  pub fn of(buffer: &LevelGeomVertexBuffer) -> XrfResult<Self> {
    let stride: u32 = buffer.get_vertex_size().ok_or_else(|| {
      XrfError::new_invalid_error("Unexpected level vertex declaration naming a type D3DDECLTYPE does not")
    })?;

    let mut layout: Self = Self {
      binormal: None,
      color: None,
      lightmap_coordinate: None,
      normal: None,
      position: 0,
      stride,
      tangent: None,
      texture_coordinate: None,
    };
    let mut has_position: bool = false;

    for element in &buffer.declaration {
      if !element.is_fed_from(LevelGeomVertexBuffer::VERTEX_STREAM) {
        continue;
      }

      match (element.usage, element.usage_index, element.kind) {
        (Self::USAGE_POSITION, 0, Self::KIND_FLOAT3) => {
          layout.position = element.offset;
          has_position = true;
        }
        (Self::USAGE_NORMAL, 0, Self::KIND_D3DCOLOR) => layout.normal = Some(element.offset),
        (Self::USAGE_TANGENT, 0, Self::KIND_D3DCOLOR) => layout.tangent = Some(element.offset),
        (Self::USAGE_BINORMAL, 0, Self::KIND_D3DCOLOR) => layout.binormal = Some(element.offset),
        (Self::USAGE_COLOR, 0, Self::KIND_D3DCOLOR) => layout.color = Some(element.offset),
        (Self::USAGE_TEXCOORD, 0, kind @ (Self::KIND_SHORT2 | Self::KIND_SHORT4)) => {
          layout.texture_coordinate = Some(LevelVertexCoordinate {
            is_tree: kind == Self::KIND_SHORT4,
            offset: element.offset,
          });
        }
        (Self::USAGE_TEXCOORD, 1, Self::KIND_SHORT2) => layout.lightmap_coordinate = Some(element.offset),
        (usage, index, kind) => {
          return Err(XrfError::new_not_implemented_error(format!(
            "Unexpected level vertex element, usage {usage} index {index} of type {kind}, which xrLC does not write"
          )));
        }
      }
    }

    if !has_position {
      return Err(XrfError::new_invalid_error(
        "Unexpected level vertex declaration carrying no position",
      ));
    }

    Ok(layout)
  }

  /// Whether the buffer is the fast path's, which carries positions and nothing else.
  pub const fn is_fastpath(&self) -> bool {
    self.normal.is_none() && self.texture_coordinate.is_none()
  }

  /// Whether the buffer's surfaces are lit from a lightmap rather than from baked vertex colour.
  pub const fn is_lightmapped(&self) -> bool {
    self.lightmap_coordinate.is_some()
  }

  /// Whether the buffer holds tree geometry, whose coordinate element carries wind parameters too.
  pub const fn is_tree(&self) -> bool {
    matches!(self.texture_coordinate, Some(coordinate) if coordinate.is_tree)
  }

  /// The quantization a base coordinate of this buffer was written with.
  pub const fn get_base_quant(&self) -> f32 {
    if self.is_tree() {
      Self::TREE_QUANT
    } else {
      Self::BASE_QUANT
    }
  }

  /// Rebuilds a base coordinate from its 16-bit part and the low byte that rides in a tangent or binormal alpha.
  pub fn rebuild_coordinate(&self, primary: i16, fraction: u8) -> f32 {
    (f32::from(primary) + f32::from(fraction) / Self::BASE_FRACTION) / self.get_base_quant()
  }

  /// Where the position of a vertex sits, relative to the start of that vertex.
  pub const fn get_position_offset(&self) -> u16 {
    self.position
  }

  /// Where the normal sits, or `None` for the fast path.
  pub const fn get_normal_offset(&self) -> Option<u16> {
    self.normal
  }

  /// Where the tangent sits, whose alpha carries the low byte of the base `u`.
  pub const fn get_tangent_offset(&self) -> Option<u16> {
    self.tangent
  }

  /// Where the binormal sits, whose alpha carries the low byte of the base `v`.
  pub const fn get_binormal_offset(&self) -> Option<u16> {
    self.binormal
  }

  /// Where the base texture coordinate sits.
  pub const fn get_texture_coordinate_offset(&self) -> Option<u16> {
    match self.texture_coordinate {
      Some(coordinate) => Some(coordinate.offset),
      None => None,
    }
  }

  /// Where the lightmap coordinate sits.
  pub const fn get_lightmap_coordinate_offset(&self) -> Option<u16> {
    self.lightmap_coordinate
  }

  /// Where the baked vertex colour sits.
  pub const fn get_color_offset(&self) -> Option<u16> {
    self.color
  }

  /// The element count a declaration may carry, kept beside the usages it is validated against.
  pub const fn get_maximum_elements() -> usize {
    LevelGeomVertexElement::MAXIMUM_LENGTH
  }
}
