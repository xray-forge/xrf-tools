use std::io::Write;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_format_size;

#[derive(Debug, Serialize, Deserialize)]
pub struct OgfKinematicsChunk {
  pub source_chunk_id: u32,
  pub motion_refs: Vec<String>,
  /// Bytes the declared chunk size covers but the count never reaches.
  ///
  /// The engine reaches this chunk through `find_chunk`, which returns the declared size but leaves the cursor on the
  /// whole-file reader, so `CKinematicsAnimated::Load` reads exactly `count` strings and never looks at the rest
  /// (`xray-16/src/Layers/xrRender/SkeletonAnimated.cpp:796`, `src/xrCore/FS_impl.h:65`). These bytes are therefore
  /// always inert, whatever they hold — shipped Anomaly visuals carry a fifth motion reference here that the game has
  /// never loaded. Empty for a well-formed chunk.
  #[serde(default)]
  pub trailing: Vec<u8>,
}

impl OgfKinematicsChunk {
  pub const CHUNK_ID: u32 = 24;
  pub const CHUNK_ID_OLD: u32 = 19;
}

// todo: Conditional read + implement chunk RW.
// todo: Conditional read + implement chunk RW.
// todo: Conditional read + implement chunk RW.
impl OgfKinematicsChunk {
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>, chunk_id: u32) -> XrfResult<Self> {
    log::info!(
      "Reading motion refs chunk: {} bytes, chunk id {} ",
      reader.read_bytes_remain(),
      chunk_id
    );

    let mut motion_refs: Vec<String> = Vec::new();

    if chunk_id == OgfKinematicsChunk::CHUNK_ID {
      for _ in 0..reader.read_u32::<T>()? {
        motion_refs.push(reader.read_w1251_string()?)
      }
    } else {
      motion_refs.push(reader.read_w1251_string()?);
    }

    // No `assert_read` here, unlike every other payload parser: what the count does not reach is unread by the engine
    // too, so it is captured rather than refused. See the `trailing` field.
    let trailing: Vec<u8> = reader.read_remaining()?;

    Ok(Self {
      source_chunk_id: chunk_id,
      motion_refs,
      trailing,
    })
  }

  /// Writes the chunk back exactly as read, trailing bytes included.
  ///
  /// A caller that wants a well-formed chunk clears `trailing` before writing; dropping the bytes is a decision for the
  /// caller to make visibly, not something this type does on its behalf.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if self.source_chunk_id == OgfKinematicsChunk::CHUNK_ID {
      writer.write_u32::<T>(to_format_size(self.motion_refs.len(), "ogf motion refs")?)?;

      for motion_ref in &self.motion_refs {
        writer.write_w1251_string(motion_ref)?;
      }
    } else {
      if self.motion_refs.len() != 1 {
        return Err(XrfError::new_unexpected_error(
          "Motions ref chunk writing error, expected vector with 1 value",
        ));
      }

      writer.write_w1251_string(self.motion_refs.first().unwrap())?;
    }

    writer.write_all(&self.trailing)?;

    Ok(())
  }
}
