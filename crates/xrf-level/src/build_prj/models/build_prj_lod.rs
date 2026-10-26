use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::build_prj::models::build_prj_lod_face::BuildPrjLodFace;

/// One impostor a multi-use model is drawn as from afar, `b_lod` (`communicate.h`): eight views and their material.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjLod {
  pub faces: [BuildPrjLodFace; Self::FACES],
  /// Its material, by index into the material table.
  pub material: u32,
}

impl BuildPrjLod {
  /// Views an impostor holds.
  pub const FACES: usize = 8;

  /// Bytes one impostor occupies.
  pub const SERIALIZED_SIZE: u64 = Self::FACES as u64 * BuildPrjLodFace::SERIALIZED_SIZE + 4;
}

impl ChunkReadWrite for BuildPrjLod {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut faces: Vec<BuildPrjLodFace> = Vec::with_capacity(Self::FACES);

    for _ in 0..Self::FACES {
      faces.push(reader.read_xr::<T, _>()?);
    }

    Ok(Self {
      faces: faces
        .try_into()
        .map_err(|_| XrfError::new_invalid_error("An impostor was not read as its eight faces"))?,
      material: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for face in &self.faces {
      writer.write_xr::<T, _>(face)?;
    }

    writer.write_u32::<T>(self.material)?;

    Ok(())
  }
}
