use byteorder::ByteOrder;
use xrf_math::Vector3d;

use crate::data::geometry::ogf_fvf::OgfFvf;

/// One vertex of a static visual, laid out as its flexible vertex format declares.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfStaticVertex {
  pub position: Vector3d,
  pub normal: Option<Vector3d>,
  pub point_size: Option<f32>,
  /// Packed `D3DCOLOR`, as stored.
  pub diffuse: Option<u32>,
  pub specular: Option<u32>,
  /// Each texture coordinate set, as many floats as the format declares for it.
  pub texture_coordinates: Vec<Vec<f32>>,
}

impl OgfStaticVertex {
  /// Read one vertex out of a slice exactly [`OgfFvf::get_stride`] long.
  pub fn read_from_slice<T: ByteOrder>(vertex: &[u8], format: &OgfFvf) -> Self {
    let mut cursor: &[u8] = vertex;

    Self {
      position: Self::take_vector::<T>(&mut cursor),
      normal: format.has_normal.then(|| Self::take_vector::<T>(&mut cursor)),
      point_size: format.has_point_size.then(|| T::read_f32(Self::take(&mut cursor))),
      diffuse: format.has_diffuse.then(|| T::read_u32(Self::take(&mut cursor))),
      specular: format.has_specular.then(|| T::read_u32(Self::take(&mut cursor))),
      texture_coordinates: format
        .texture_coordinate_sizes
        .iter()
        .map(|size| (0..*size).map(|_| T::read_f32(Self::take(&mut cursor))).collect())
        .collect(),
    }
  }

  fn take_vector<T: ByteOrder>(cursor: &mut &[u8]) -> Vector3d {
    Vector3d::new(
      T::read_f32(Self::take(cursor)),
      T::read_f32(Self::take(cursor)),
      T::read_f32(Self::take(cursor)),
    )
  }

  /// The next four bytes, advancing past them.
  fn take<'a>(cursor: &mut &'a [u8]) -> &'a [u8] {
    let (word, rest): (&[u8], &[u8]) = cursor.split_at(4);

    *cursor = rest;

    word
  }
}
