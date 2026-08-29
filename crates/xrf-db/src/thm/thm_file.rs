use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, find_optional_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::thm::chunks::thm_bump_chunk::ThmBumpChunk;

/// Texture descriptor file, `STextureParams` in the engine (`ETextureParams.cpp`).
///
/// Reads only the parts the toolchain needs. A thm carries several chunks of authoring metadata,
/// but the one with runtime consequences is the bump declaration, so everything else stays
/// unparsed and any edit patches raw chunks rather than re-serializing, see
/// [`crate::ThmBumpProcessor`].
#[derive(Debug, Serialize, Deserialize)]
pub struct ThmFile {
  pub bump: Option<ThmBumpChunk>,
}

impl ThmFile {
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "THM file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads a descriptor from a chunk reader over any data source.
  ///
  /// The route an archived descriptor takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      bump: match find_optional_chunk_by_id(&chunks, ThmBumpChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, ThmBumpChunk>()?),
        None => None,
      },
    })
  }

  /// Bump texture this descriptor asks the engine to resolve, if any.
  pub fn used_bump_name(&self) -> Option<&str> {
    self
      .bump
      .as_ref()
      .filter(|bump| bump.is_used())
      .map(|bump| bump.name.as_str())
  }
}
