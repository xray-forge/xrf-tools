use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkTrailing, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{encode_w1251_bytes_to_string, to_format_size};

use crate::ogf::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;

/// Bytes past the last well-formed root chunk of a visual that the engine's loader never reads.
///
/// Distinct from trailing bytes, which are merely present: residue is trailing bytes that have been *accounted for*.
/// Anything unaccounted stays an error, so this type never records "some junk was at the end".
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OgfResidue {
  /// Offset of the first residue byte from the start of the file.
  pub position: u64,
  pub bytes: Vec<u8>,
  pub cause: OgfResidueCause,
}

/// Why residue is inert to the engine, which is the only reason it is tolerated at all.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OgfResidueCause {
  /// Too few bytes to be a chunk header.
  ///
  /// The engine's own walk reads four bytes here regardless, two of them past the buffer, because
  /// `VERIFY(Pos + cnt <= Size)` (`xray-16/src/xrCore/FS.cpp:367`) is debug-only. Tolerated because shipped assets
  /// carry it, not because it is correct.
  TrailingFragment,
  /// Completes a motion reference path the declared size of chunk 24 or 19 cut in half.
  ///
  /// The count-governed read never reaches it, so the path is never loaded. Reported because normalizing discards it
  /// and this is the only record that it was there.
  SplitMotionRef { path: String },
}

/// The eight bytes of id and size that open every chunk.
const CHUNK_HEADER_SIZE: u64 = 8;

impl OgfResidue {
  /// Walks the root chunks of a visual, accounting for bytes the engine's loader never reads.
  ///
  /// The seam every root read of an ogf goes through. Trailing bytes are tolerated in exactly two shapes and refused
  /// otherwise, with the error the strict walk raised; the rule is that bytes are accepted only when they can be
  /// explained, never because they are short or at the end. Nested walks do not come here and stay strict.
  pub fn read_root_chunks<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<(Vec<ChunkReader<D>>, Option<Self>)> {
    let (chunks, trailing) = reader.read_children_with_trailing()?;

    let Some(trailing) = trailing else {
      return Ok((chunks, None));
    };

    let residue: Self = Self::account_for::<T, _>(&chunks, trailing)?;

    Ok((chunks, Some(residue)))
  }

  /// Explains trailing bytes, or returns the error a strict walk would have raised on them.
  fn account_for<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
    trailing: ChunkTrailing<D>,
  ) -> XrfResult<Self> {
    let ChunkTrailing {
      position,
      size,
      mut data,
      error,
    } = trailing;

    let mut bytes: Vec<u8> = vec![0; usize::try_from(size).unwrap_or(usize::MAX)];

    if data.read_exact(&mut bytes).is_err() {
      return Err(error);
    }

    if size < CHUNK_HEADER_SIZE {
      return Ok(Self {
        position,
        bytes,
        cause: OgfResidueCause::TrailingFragment,
      });
    }

    match Self::read_split_motion_ref::<T, _>(chunks, &bytes) {
      Some(path) => Ok(Self {
        position,
        bytes,
        cause: OgfResidueCause::SplitMotionRef { path },
      }),
      None => Err(error),
    }
  }

  /// The motion reference path these bytes complete, when they complete one.
  ///
  /// Requires the motion refs chunk to be the *last* root chunk: the split only exists because the declared size cut a
  /// string that continues immediately after it, so anything further away is a different file with a different defect.
  fn read_split_motion_ref<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
    bytes: &[u8],
  ) -> Option<String> {
    let last: &ChunkReader<D> = chunks.last()?;

    if last.id != OgfKinematicsChunk::CHUNK_ID && last.id != OgfKinematicsChunk::CHUNK_ID_OLD {
      return None;
    }

    let kinematics: OgfKinematicsChunk = OgfKinematicsChunk::read::<T, _>(&mut last.clone(), last.id).ok()?;

    if kinematics.trailing.is_empty() {
      return None;
    }

    let mut combined: Vec<u8> = kinematics.trailing;

    combined.extend_from_slice(bytes);

    // Exactly one NUL-terminated string: a terminator at the very end and nowhere before it. Two paths, or none,
    // describe something this rule has not been shown and must not guess at.
    let (terminator, path) = combined.split_last()?;

    if *terminator != 0 || path.is_empty() || path.contains(&0) {
      return None;
    }

    encode_w1251_bytes_to_string(path).ok()
  }
}

/// Rewrites a visual into bytes the engine loads identically, with nothing left unread.
///
/// Three edits, all derived from what the read already identified: drop the trailing bytes inside the motion refs
/// chunk, reduce its declared size to match, and drop the root residue. Nothing else moves.
///
/// This is byte surgery on the source rather than a re-run of a writer, which is what lets the patchers use it as the
/// target of their losslessness guard: a serializer bug still shows up as a mismatch against it.
///
/// # Errors
///
/// Returns the strict walk's error for a visual whose trailing bytes cannot be accounted for. A visual with none is
/// returned unchanged, byte for byte.
pub fn normalize_ogf_bytes<T: ByteOrder>(original: &[u8]) -> XrfResult<Vec<u8>> {
  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(original)?;
  let (chunks, residue) = OgfResidue::read_root_chunks::<T, _>(&mut reader)?;

  let kinematics: Option<(u64, u64, u64)> = match chunks
    .iter()
    .find(|it| it.id == OgfKinematicsChunk::CHUNK_ID || it.id == OgfKinematicsChunk::CHUNK_ID_OLD)
  {
    Some(chunk) => match OgfKinematicsChunk::read::<T, _>(&mut chunk.clone(), chunk.id)?
      .trailing
      .len() as u64
    {
      0 => None,
      length => Some((chunk.position, chunk.size, length)),
    },
    None => None,
  };

  if kinematics.is_none() && residue.is_none() {
    return Ok(original.to_vec());
  }

  let end: usize = match &residue {
    Some(residue) => usize::try_from(residue.position)
      .map_err(|_| XrfError::new_invalid_error("Residue offset exceeds the supported range"))?,
    None => original.len(),
  };

  let Some((position, size, trailing_length)) = kinematics else {
    return Ok(original[..end].to_vec());
  };

  let header_at: usize = usize::try_from(position.saturating_sub(CHUNK_HEADER_SIZE))
    .map_err(|_| XrfError::new_invalid_error("Chunk offset exceeds the supported range"))?;
  let payload_at: usize = header_at + CHUNK_HEADER_SIZE as usize;
  let kept: usize = usize::try_from(size.saturating_sub(trailing_length))
    .map_err(|_| XrfError::new_invalid_error("Chunk size exceeds the supported range"))?;
  let payload_end: usize = payload_at.saturating_add(usize::try_from(size).unwrap_or(usize::MAX));

  if payload_end > end {
    return Err(XrfError::new_invalid_error(
      "Refused to normalize an ogf whose motion refs chunk lies outside its own bytes",
    ));
  }

  let mut size_field: [u8; 4] = [0; 4];

  T::write_u32(&mut size_field, to_format_size(kept, "ogf chunk payload")?);

  let mut normalized: Vec<u8> = Vec::with_capacity(end);

  normalized.extend_from_slice(&original[..header_at + 4]);
  normalized.extend_from_slice(&size_field);
  normalized.extend_from_slice(&original[payload_at..payload_at + kept]);
  normalized.extend_from_slice(&original[payload_end..end]);

  Ok(normalized)
}
