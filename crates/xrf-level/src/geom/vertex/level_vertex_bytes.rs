use byteorder::ByteOrder;
use xrf_math::Vector3d;

use crate::geom::vertex::level_vertex::LevelVertex;
use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;

/// One vertex as the level stores it, a stride of bytes, read through the declaration saying where each element sits.
#[derive(Clone, Copy, Debug)]
pub struct LevelVertexBytes<'a> {
  layout: &'a LevelVertexLayout,
  /// Exactly one stride, which is what lets every element the layout places be sliced without a check.
  bytes: &'a [u8],
}

impl<'a> LevelVertexBytes<'a> {
  /// A vertex of `layout`, one stride of `bytes`.
  pub(crate) fn new(layout: &'a LevelVertexLayout, bytes: &'a [u8]) -> Self {
    debug_assert_eq!(bytes.len(), layout.get_stride() as usize, "a vertex is one stride");

    Self { layout, bytes }
  }

  /// The three floats of the position element.
  pub fn get_position<T: ByteOrder>(&self) -> Vector3d {
    let at: usize = self.layout.get_position_offset() as usize;

    Vector3d {
      x: T::read_f32(&self.bytes[at..at + 4]),
      y: T::read_f32(&self.bytes[at + 4..at + 8]),
      z: T::read_f32(&self.bytes[at + 8..at + 12]),
    }
  }

  /// The normal's `D3DCOLOR` bytes, its fourth the hemisphere term, or `None` for the fast path.
  pub fn get_normal(&self) -> Option<[u8; 4]> {
    self.layout.get_normal_offset().map(|offset| self.take_four(offset))
  }

  /// The tangent's `D3DCOLOR` bytes, its fourth the low byte of the base `u`.
  pub fn get_tangent(&self) -> Option<[u8; 4]> {
    self.layout.get_tangent_offset().map(|offset| self.take_four(offset))
  }

  /// The binormal's `D3DCOLOR` bytes, its fourth the low byte of the base `v`.
  pub fn get_binormal(&self) -> Option<[u8; 4]> {
    self.layout.get_binormal_offset().map(|offset| self.take_four(offset))
  }

  /// The baked colour's `D3DCOLOR` bytes, its fourth the sun term.
  pub fn get_color(&self) -> Option<[u8; 4]> {
    self.layout.get_color_offset().map(|offset| self.take_four(offset))
  }

  /// The base coordinate's shorts as stored: a tree's four, its wind terms after its coordinate, or a baked surface's
  /// two and zeroes after them.
  pub fn get_texture_coordinate<T: ByteOrder>(&self) -> Option<[i16; LevelVertexLayout::TREE_COORDINATE_SHORTS]> {
    let offset: u16 = self.layout.get_texture_coordinate_offset()?;
    let mut shorts: [i16; LevelVertexLayout::TREE_COORDINATE_SHORTS] = [0; LevelVertexLayout::TREE_COORDINATE_SHORTS];

    for (index, short) in shorts
      .iter_mut()
      .take(self.layout.get_texture_coordinate_shorts())
      .enumerate()
    {
      *short = self.take_short::<T>(offset + index as u16 * 2);
    }

    Some(shorts)
  }

  /// The lightmap coordinate's two shorts as stored.
  pub fn get_lightmap_coordinate<T: ByteOrder>(&self) -> Option<[i16; 2]> {
    self
      .layout
      .get_lightmap_coordinate_offset()
      .map(|offset| [self.take_short::<T>(offset), self.take_short::<T>(offset + 2)])
  }

  /// What xrLC had before it quantized the vertex.
  pub fn decode<T: ByteOrder>(&self) -> LevelVertex {
    let (normal, hemi): (Vector3d, u8) = self
      .get_normal()
      .map_or((Vector3d::new(0.0, 0.0, 0.0), 0), LevelVertex::decode_direction);
    let tangent: Option<(Vector3d, u8)> = self.get_tangent().map(LevelVertex::decode_direction);
    let binormal: Option<(Vector3d, u8)> = self.get_binormal().map(LevelVertex::decode_direction);

    // The low byte of each base coordinate rides in a tangent or binormal alpha, so the coordinate is rebuilt from
    // two elements rather than one. A tree carries both but adds neither: `deffer_tree_*.vs` scales `I.tc` by
    // `consts` alone.
    let fraction = |direction: &Option<(Vector3d, u8)>| match direction {
      Some((_, fraction)) if !self.layout.is_tree() => *fraction,
      _ => 0,
    };
    let (fraction_u, fraction_v): (u8, u8) = (fraction(&tangent), fraction(&binormal));

    LevelVertex {
      binormal: binormal.map(|(direction, _)| direction),
      color: self.get_color().map(LevelVertex::decode_color),
      hemi,
      lightmap_coordinate: self.get_lightmap_coordinate::<T>().map(|[u, v]| {
        (
          f32::from(u) / LevelVertexLayout::LIGHTMAP_QUANT,
          f32::from(v) / LevelVertexLayout::LIGHTMAP_QUANT,
        )
      }),
      normal,
      position: self.get_position::<T>(),
      tangent: tangent.map(|(direction, _)| direction),
      texture_coordinate: self.get_texture_coordinate::<T>().map_or((0.0, 0.0), |[u, v, ..]| {
        (
          self.layout.rebuild_coordinate(u, fraction_u),
          self.layout.rebuild_coordinate(v, fraction_v),
        )
      }),
    }
  }

  /// The four bytes of a `D3DCOLOR` element.
  fn take_four(&self, offset: u16) -> [u8; 4] {
    let at: usize = offset as usize;
    let mut bytes: [u8; 4] = [0; 4];

    bytes.copy_from_slice(&self.bytes[at..at + 4]);

    bytes
  }

  /// One signed 16-bit component of a coordinate element.
  fn take_short<T: ByteOrder>(&self, offset: u16) -> i16 {
    let at: usize = offset as usize;

    T::read_i16(&self.bytes[at..at + 2])
  }
}
