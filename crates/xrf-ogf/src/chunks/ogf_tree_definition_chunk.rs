use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::{Matrix4x4, Vector3d};

/// The static lighting terms a tree is shaded with, the engine's `_5color`.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfTreeColor {
  pub rgb: Vector3d,
  pub hemi: f32,
  pub sun: f32,
}

impl ChunkReadWrite for OgfTreeColor {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      rgb: reader.read_xr::<T, _>()?,
      hemi: reader.read_f32::<T>()?,
      sun: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.rgb)?;
    writer.write_f32::<T>(self.hemi)?;
    writer.write_f32::<T>(self.sun)?;

    Ok(())
  }
}

/// Where a tree stands and how it is lit, `OGF_TREEDEF2`.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfTreeDefinitionChunk {
  pub transform: Matrix4x4,
  pub scale: OgfTreeColor,
  pub bias: OgfTreeColor,
}

impl OgfTreeDefinitionChunk {
  /// `OGF_TREEDEF2` (`xray-16/src/xrCore/FMesh.hpp`).
  pub const CHUNK_ID: u32 = 12;

  /// What the engine halves both terms by as it reads them.
  pub const COLOR_SCALE: f32 = 0.5;
}

impl ChunkReadWrite for OgfTreeDefinitionChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let definition: Self = Self {
      transform: reader.read_xr::<T, _>()?,
      scale: reader.read_xr::<T, _>()?,
      bias: reader.read_xr::<T, _>()?,
    };

    reader.assert_read("Expect all data to be read from ogf tree definition chunk")?;

    Ok(definition)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.transform)?;
    writer.write_xr::<T, _>(&self.scale)?;
    writer.write_xr::<T, _>(&self.bias)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::{Matrix4x4, Vector3d};
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::chunks::ogf_tree_definition_chunk::{OgfTreeColor, OgfTreeDefinitionChunk};

  fn color(hemi: f32, sun: f32) -> OgfTreeColor {
    OgfTreeColor {
      hemi,
      rgb: Vector3d {
        x: 0.25,
        y: 0.5,
        z: 0.75,
      },
      sun,
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: OgfTreeDefinitionChunk = OgfTreeDefinitionChunk {
      bias: color(0.3, 0.4),
      scale: color(0.1, 0.2),
      transform: Matrix4x4 {
        values: [
          1.0, 0.0, 0.0, 0.0, //
          0.0, 1.0, 0.0, 0.0, //
          0.0, 0.0, 1.0, 0.0, //
          12.5, 0.0, -30.25, 1.0,
        ],
      },
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    // A matrix and two five float colours, which is what `FTreeVisual::Load` reads in that order.
    assert_eq!(writer.bytes_written(), 64 + 20 + 20);

    writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      OgfTreeDefinitionChunk::CHUNK_ID,
    )?;

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    let read: OgfTreeDefinitionChunk = OgfTreeDefinitionChunk::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(read, original);
    assert_eq!(
      read.transform.get_translation().x,
      12.5,
      "the fourth row is where it stands"
    );

    Ok(())
  }
}
