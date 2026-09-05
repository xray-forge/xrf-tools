use std::io::Write;

use serde::{Deserialize, Serialize};
use xrf_chunk::{CHUNK_ID_COMPRESSED_MASK, CHUNK_ID_MASK, ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// The preview picture a descriptor carries, `THM_CHUNK_DATA` in the engine (`ETextureParams.h`).
///
/// A `THUMB_WIDTH` by `THUMB_HEIGHT` block of `u32` pixels, both 128 (`ETextureParams.h`), written through
/// `CFS_CompressMark` so the payload on disk is the engine's own LZ stream rather than the pixels.
///
/// Kept as the bytes it was read as, and never rebuilt: the trunk SDK does not even write this chunk any more - its
/// `w_chunk` call is commented out in `ETextureThumbnail::Save` (`EThumbnailTexture.cpp`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmThumbnailChunk {
  /// Whether the payload is the engine's compressed stream, which is the only form seen in the wild.
  pub is_compressed: bool,
  /// The payload exactly as the file stores it.
  pub data: Vec<u8>,
}

impl ThmThumbnailChunk {
  pub const CHUNK_ID: u32 = 0x0811;

  /// Whether a chunk id names this chunk, in either of the two forms a writer can have marked it with.
  ///
  /// The engine matches ids the same way, masking the compression bit off before comparing
  /// (`xrCore/FS.h`), so a reader that demanded the marked spelling would miss a file the engine loads.
  pub const fn matches(id: u32) -> bool {
    id & CHUNK_ID_MASK == Self::CHUNK_ID
  }

  /// Reads the payload of a chunk found under `id`, which is what says whether it is compressed.
  pub fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>, id: u32) -> XrfResult<Self> {
    Ok(Self {
      is_compressed: id & CHUNK_ID_COMPRESSED_MASK != 0,
      data: reader.read_remaining()?,
    })
  }

  /// The id this chunk is written back under.
  pub const fn to_chunk_id(&self) -> u32 {
    if self.is_compressed {
      Self::CHUNK_ID | CHUNK_ID_COMPRESSED_MASK
    } else {
      Self::CHUNK_ID
    }
  }

  /// Writes the payload, which is copied rather than re-encoded.
  pub fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_all(&self.data)?;

    Ok(())
  }
}
