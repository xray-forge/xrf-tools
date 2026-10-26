use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::to_format_size;

use crate::build_prj::build_prj_file::BuildPrjFile;
use crate::build_prj::geometry::build_prj_face::BuildPrjFace;

/// One multi-use model, `b_mu_model` (`communicate.h`): geometry the compiler lights once and every placement shares.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjMuModel {
  pub name: String,
  pub vertices: Vec<Vector3d<f32>>,
  /// Its faces, naming its own vertices.
  pub faces: Vec<BuildPrjFace>,
  /// The impostor it is drawn as from afar, by index into the impostors; `u16::MAX` for none.
  pub lod: u16,
  /// A smoothing group per face, which version 18 added and xrLC asserts.
  pub smoothing_groups: Option<Vec<u32>>,
}

impl BuildPrjMuModel {
  /// Reads one model, with a smoothing group per face when the project's version stores them.
  ///
  /// # Errors
  ///
  /// Returns an error when the model is truncated or its name is not Windows-1251.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    has_smoothing_groups: bool,
  ) -> XrfResult<Self> {
    let name: String = reader.read_w1251_string()?;
    let vertex_count: u32 = reader.read_u32::<T>()?;
    let mut vertices: Vec<Vector3d<f32>> = reader.new_bounded_vec(
      u64::from(vertex_count),
      BuildPrjFile::VERTEX_SIZE,
      "multi-use model vertices",
    )?;

    for _ in 0..vertex_count {
      vertices.push(reader.read_xr::<T, _>()?);
    }

    let face_count: u32 = reader.read_u32::<T>()?;
    let mut faces: Vec<BuildPrjFace> = reader.new_bounded_vec(
      u64::from(face_count),
      BuildPrjFace::SERIALIZED_SIZE,
      "multi-use model faces",
    )?;

    for _ in 0..face_count {
      faces.push(reader.read_xr::<T, _>()?);
    }

    let lod: u16 = reader.read_u16::<T>()?;
    let smoothing_groups: Option<Vec<u32>> = if has_smoothing_groups {
      let mut groups: Vec<u32> =
        reader.new_bounded_vec(u64::from(face_count), 4, "multi-use model smoothing groups")?;

      for _ in 0..face_count {
        groups.push(reader.read_u32::<T>()?);
      }

      Some(groups)
    } else {
      None
    };

    Ok(Self {
      name,
      vertices,
      faces,
      lod,
      smoothing_groups,
    })
  }

  /// Writes one model in the layout [`Self::read`] reads.
  ///
  /// # Errors
  ///
  /// Returns an error when the smoothing groups do not match the faces one for one, a count does not fit its field, or
  /// the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if let Some(groups) = &self.smoothing_groups
      && groups.len() != self.faces.len()
    {
      return Err(XrfError::new_invalid_error(format!(
        "Multi-use model '{}' holds {} smoothing groups for {} faces",
        self.name,
        groups.len(),
        self.faces.len()
      )));
    }

    writer.write_w1251_string(&self.name)?;
    writer.write_u32::<T>(to_format_size(self.vertices.len(), "multi-use model vertices")?)?;

    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    writer.write_u32::<T>(to_format_size(self.faces.len(), "multi-use model faces")?)?;

    for face in &self.faces {
      writer.write_xr::<T, _>(face)?;
    }

    writer.write_u16::<T>(self.lod)?;

    for group in self.smoothing_groups.iter().flatten() {
      writer.write_u32::<T>(*group)?;
    }

    Ok(())
  }
}
