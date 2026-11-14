use xrf_math::Vector3d;

use crate::geom::vertex::level_vertex::LevelVertex;
use crate::geom::vertex::level_vertex_base_coordinate::LevelVertexBaseCoordinate;
use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;

/// What xrLC computes for a static vertex, turned into the stored form as `OGF_Face_Save.cpp` turns it: each step
/// rounds down and clamps, so every value [`LevelVertex`] decodes quantizes back to the bytes it came from.
pub struct LevelVertexQuantizer;

impl LevelVertexQuantizer {
  /// A direction's `D3DCOLOR` element with `extra` in its fourth byte, `u8_vec4`: each component of `[-1, 1]` taken to
  /// `floor((c + 1) * 127.5)` in `[0, 255]`, stored blue, green, red, extra. xrLC normalizes a normal, and a basis
  /// vector safely (`base_basis::set`), before this; the caller does as much.
  pub fn get_direction(direction: &Vector3d, extra: u8) -> [u8; 4] {
    let component = |value: f32| ((value + 1.0) * (0.5 * 255.0)).floor().clamp(0.0, 255.0) as u8;
    let mut bytes: [u8; 4] = [0; 4];

    bytes[LevelVertex::DIRECTION_X] = component(direction.x);
    bytes[LevelVertex::DIRECTION_Y] = component(direction.y);
    bytes[LevelVertex::DIRECTION_Z] = component(direction.z);
    bytes[LevelVertex::DIRECTION_EXTRA] = extra;

    bytes
  }

  /// A base coordinate's stored parts, `s24_tc_base`: scaled by [`LevelVertexLayout::BASE_QUANT`], the whole part
  /// clamped to a short, and the rest times 255.5 rounded down to a byte.
  pub fn get_base_coordinate(value: f32) -> LevelVertexBaseCoordinate {
    let rebased: f32 = value * LevelVertexLayout::BASE_QUANT;
    let primary: f32 = rebased.floor().clamp(f32::from(i16::MIN), f32::from(i16::MAX));

    LevelVertexBaseCoordinate {
      primary: primary as i16,
      fraction: (255.5 * (rebased - primary)).floor().clamp(0.0, 255.0) as u8,
    }
  }

  /// A lightmap coordinate's short, `s16_tc_lmap`: scaled by [`LevelVertexLayout::LIGHTMAP_QUANT`] and rounded down.
  pub fn get_lightmap_coordinate(value: f32) -> i16 {
    (value * LevelVertexLayout::LIGHTMAP_QUANT)
      .floor()
      .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16
  }

  /// A tree's coordinate short, `QC` (`xrMU_Model_export_geometry.cpp:8`): scaled by
  /// [`LevelVertexLayout::TREE_QUANT`] and rounded down, a base coordinate and the wind fraction alike.
  pub fn get_tree_coordinate(value: f32) -> i16 {
    (value * LevelVertexLayout::TREE_QUANT)
      .floor()
      .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16
  }

  /// A light term's byte, `u8_clr`: `floor(value * 255)` in `[0, 255]`.
  pub fn get_light_term(value: f32) -> u8 {
    (value * 255.0).floor().clamp(0.0, 255.0) as u8
  }
}
