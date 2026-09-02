use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkTrailing};
use xrf_error::XrfResult;
use xrf_utils::encode_w1251_bytes_to_string;

use crate::ogf::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::ogf::residue::ogf_residue_cause::OgfResidueCause;

/// The eight bytes of id and size that open every chunk.
pub(super) const CHUNK_HEADER_SIZE: u64 = 8;

/// Bytes past the last well-formed root chunk of a visual that the engine's loader never reads.
///
/// Distinct from trailing bytes, which are merely present: residue is trailing bytes that have been *accounted for*.
/// Anything unaccounted stays an error, so this type never records "some junk was at the end".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfResidue {
  /// Offset of the first residue byte from the start of the file.
  pub position: u64,
  pub bytes: Vec<u8>,
  pub cause: OgfResidueCause,
}

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
