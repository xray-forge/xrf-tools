use byteorder::{ByteOrder, WriteBytesExt};
use xrf_chunk::ChunkWriter;
use xrf_error::{XrfError, XrfResult};
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

  /// Writes the vertex laid out as `format` declares, the inverse of [`Self::read_from_slice`].
  ///
  /// # Errors
  ///
  /// Returns an error when the vertex does not hold what `format` declares: a field it lacks or carries beyond it, or a
  /// texture coordinate set of another size.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter, format: &OgfFvf) -> XrfResult {
    let is_laid_out: bool = format.has_normal == self.normal.is_some()
      && format.has_point_size == self.point_size.is_some()
      && format.has_diffuse == self.diffuse.is_some()
      && format.has_specular == self.specular.is_some()
      && self
        .texture_coordinates
        .iter()
        .map(Vec::len)
        .eq(format.texture_coordinate_sizes.iter().copied());

    if !is_laid_out {
      return Err(XrfError::new_invalid_error(format!(
        "A static vertex does not hold what vertex format {:#x} declares",
        format.raw
      )));
    }

    Self::write_vector::<T>(writer, &self.position)?;

    if let Some(normal) = &self.normal {
      Self::write_vector::<T>(writer, normal)?;
    }

    if let Some(point_size) = self.point_size {
      writer.write_f32::<T>(point_size)?;
    }

    for color in [self.diffuse, self.specular].into_iter().flatten() {
      writer.write_u32::<T>(color)?;
    }

    for value in self.texture_coordinates.iter().flatten() {
      writer.write_f32::<T>(*value)?;
    }

    Ok(())
  }

  fn write_vector<T: ByteOrder>(writer: &mut ChunkWriter, vector: &Vector3d) -> XrfResult {
    writer.write_f32::<T>(vector.x)?;
    writer.write_f32::<T>(vector.y)?;
    writer.write_f32::<T>(vector.z)?;

    Ok(())
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
