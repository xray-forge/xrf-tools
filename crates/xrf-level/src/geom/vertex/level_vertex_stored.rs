use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;

/// One vertex as a level stores it, each element in its declared form: the step between the bytes and what xrLC
/// computed, both ways.
///
/// An element the declaration lacks is `None`. A `D3DCOLOR` element is its four bytes in file order: blue, green, red,
/// then the byte riding along (see [`crate::LevelVertex::DIRECTION_X`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelVertexStored {
  pub position: Vector3d,
  pub normal: Option<[u8; 4]>,
  pub tangent: Option<[u8; 4]>,
  pub binormal: Option<[u8; 4]>,
  pub color: Option<[u8; 4]>,
  /// The base coordinate's shorts: a tree's four, or a baked surface's two and zeroes after them.
  pub texture_coordinate: Option<[i16; LevelVertexLayout::TREE_COORDINATE_SHORTS]>,
  pub lightmap_coordinate: Option<[i16; 2]>,
}

impl LevelVertexStored {
  /// Writes the vertex into `bytes`, one stride of `layout`, each element at its offset.
  ///
  /// # Errors
  ///
  /// Returns an error when `bytes` is not one stride, or the vertex's elements are not exactly the layout's.
  pub fn write_into<T: ByteOrder>(&self, layout: &LevelVertexLayout, bytes: &mut [u8]) -> XrfResult {
    if bytes.len() != layout.get_stride() as usize {
      return Err(XrfError::new_invalid_error(format!(
        "A level vertex of stride {} was given {} bytes",
        layout.get_stride(),
        bytes.len()
      )));
    }

    let at: usize = usize::from(layout.get_position_offset());

    T::write_f32(&mut bytes[at..at + 4], self.position.x);
    T::write_f32(&mut bytes[at + 4..at + 8], self.position.y);
    T::write_f32(&mut bytes[at + 8..at + 12], self.position.z);

    Self::put_four(bytes, layout.get_normal_offset(), self.normal, "normal")?;
    Self::put_four(bytes, layout.get_tangent_offset(), self.tangent, "tangent")?;
    Self::put_four(bytes, layout.get_binormal_offset(), self.binormal, "binormal")?;
    Self::put_four(bytes, layout.get_color_offset(), self.color, "colour")?;

    match (layout.get_texture_coordinate_offset(), self.texture_coordinate) {
      (Some(offset), Some(shorts)) => {
        for (index, short) in shorts.iter().take(layout.get_texture_coordinate_shorts()).enumerate() {
          let at: usize = usize::from(offset) + index * 2;

          T::write_i16(&mut bytes[at..at + 2], *short);
        }
      }
      (None, None) => {}
      _ => return Err(Self::mismatch("base coordinate")),
    }

    match (layout.get_lightmap_coordinate_offset(), self.lightmap_coordinate) {
      (Some(offset), Some([u, v])) => {
        let at: usize = usize::from(offset);

        T::write_i16(&mut bytes[at..at + 2], u);
        T::write_i16(&mut bytes[at + 2..at + 4], v);
      }
      (None, None) => {}
      _ => return Err(Self::mismatch("lightmap coordinate")),
    }

    Ok(())
  }

  /// Puts a `D3DCOLOR` element's four bytes where the layout places it.
  fn put_four(bytes: &mut [u8], offset: Option<u16>, value: Option<[u8; 4]>, what: &str) -> XrfResult {
    match (offset, value) {
      (Some(offset), Some(value)) => {
        let at: usize = usize::from(offset);

        bytes[at..at + 4].copy_from_slice(&value);

        Ok(())
      }
      (None, None) => Ok(()),
      _ => Err(Self::mismatch(what)),
    }
  }

  fn mismatch(what: &str) -> XrfError {
    XrfError::new_invalid_error(format!(
      "A level vertex's {what} and its layout disagree on whether it is stored"
    ))
  }
}
