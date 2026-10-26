use byteorder::ByteOrder;
use xrf_chunk::{
  CHUNK_HEADER_SIZE, CHUNK_ID_COMPRESSED_MASK, CHUNK_ID_MASK, ChunkDataSource, ChunkReader, ChunkTrailing,
};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::decode_w1251_bytes_to_string;

use crate::chunks::skeleton::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::residue::ogf_chunk_access::OgfChunkAccess;
use crate::residue::ogf_residue_cause::OgfResidueCause;

/// Where a visual departs from a well-formed root chunk tree in a way the engine's loader does not notice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfResidue {
  /// Offset of the first residue byte from the start of the file, or of the chunk header whose size is wrong.
  pub position: u64,
  /// The bytes the engine never reads, none for a size the payload contradicts.
  pub bytes: Vec<u8>,
  pub cause: OgfResidueCause,
}

impl OgfResidue {
  /// Walks the root chunks of a visual as the engine reaches them, accounting for whatever a strict walk would refuse.
  ///
  /// # Errors
  ///
  /// Returns the strict walk's error for bytes this cannot account for, and a not-implemented error for a compressed
  /// chunk the engine would decompress.
  pub fn read_root_chunks<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<(Vec<ChunkReader<D>>, Option<Self>)> {
    let (mut chunks, trailing) = reader.read_children_with_trailing_including_compressed()?;

    Self::unmark_found_chunks(&mut chunks)?;

    let Some(trailing) = trailing else {
      return Ok((chunks, None));
    };

    let residue: Self = Self::account_for::<T, _>(reader, &mut chunks, trailing)?;

    Ok((chunks, Some(residue)))
  }

  /// Drops the compression mark from every chunk the engine finds, which ignores it.
  fn unmark_found_chunks<D: ChunkDataSource>(chunks: &mut [ChunkReader<D>]) -> XrfResult {
    for chunk in chunks
      .iter_mut()
      .filter(|chunk| chunk.id & CHUNK_ID_COMPRESSED_MASK != 0)
    {
      let id: u32 = chunk.id & CHUNK_ID_MASK;

      match OgfChunkAccess::of(id) {
        Some(OgfChunkAccess::Sized | OgfChunkAccess::Found) => chunk.id = id,
        Some(OgfChunkAccess::Opened) => {
          return Err(XrfError::new_not_implemented_error(format!(
            "Compressed ogf chunk {id} at position {}, which the engine decompresses as it opens it",
            chunk.position
          )));
        }
        // No loader reads it, so the mark changes nothing.
        None => {}
      }
    }

    Ok(())
  }

  /// Explains trailing bytes, or returns the error a strict walk would have raised on them.
  fn account_for<T: ByteOrder, D: ChunkDataSource>(
    reader: &ChunkReader<D>,
    chunks: &mut Vec<ChunkReader<D>>,
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

    if let Some(path) = Self::read_split_motion_ref::<T, _>(chunks, &bytes) {
      return Ok(Self {
        position,
        bytes,
        cause: OgfResidueCause::SplitMotionRef { path },
      });
    }

    if let Some(residue) = Self::widen_understated_motion_refs::<T, _>(reader, chunks, position + size) {
      return Ok(residue);
    }

    match Self::clip_overstated_motion_refs::<T, _>(reader, chunks, position, &bytes) {
      Some(residue) => Ok(residue),
      None => Err(error),
    }
  }

  /// The motion reference path these bytes complete, when they complete one past the count the loader reads.
  fn read_split_motion_ref<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
    bytes: &[u8],
  ) -> Option<String> {
    let last: &ChunkReader<D> = chunks.last()?;

    if !Self::is_kinematics(last.id) {
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

    decode_w1251_bytes_to_string(path).ok()
  }

  /// Widens a last motion refs chunk whose counted references run on past its declared size to the end of the file,
  /// which is where `CKinematicsAnimated::Load`'s `r_stringZ` reads them from (`SkeletonAnimated.cpp`).
  ///
  /// Only when the declared size cannot hold the count, and the references end exactly where the file does: anything
  /// after them would be bytes nothing accounts for.
  fn widen_understated_motion_refs<T: ByteOrder, D: ChunkDataSource>(
    reader: &ChunkReader<D>,
    chunks: &mut [ChunkReader<D>],
    end: u64,
  ) -> Option<Self> {
    let last: &mut ChunkReader<D> = chunks.last_mut()?;

    if !Self::is_kinematics(last.id) || OgfKinematicsChunk::read::<T, _>(&mut last.clone(), last.id).is_ok() {
      return None;
    }

    let widened: ChunkReader<D> = ChunkReader {
      id: last.id,
      size: end - last.position,
      position: last.position,
      data: reader.data.slice(last.position..end),
    };

    if !OgfKinematicsChunk::read::<T, _>(&mut widened.clone(), widened.id)
      .ok()?
      .trailing
      .is_empty()
    {
      return None;
    }

    let declared: u32 = u32::try_from(last.size).ok()?;
    let header: u64 = last.position - CHUNK_HEADER_SIZE;

    *last = widened;

    Some(Self {
      position: header,
      bytes: Vec::new(),
      cause: OgfResidueCause::UnderstatedMotionRefs { declared },
    })
  }

  /// Clips a last motion refs chunk that declares more than the file holds to the bytes it does, when its references
  /// end exactly where the file does: `CKinematicsAnimated::Load` finds the chunk and reads them off the visual's own
  /// reader, so the size it overstates is never used.
  ///
  /// Any other chunk cut short stays refused, found or opened: a payload that does not read whole is a truncated file,
  /// not a size the engine ignores.
  fn clip_overstated_motion_refs<T: ByteOrder, D: ChunkDataSource>(
    reader: &ChunkReader<D>,
    chunks: &mut Vec<ChunkReader<D>>,
    position: u64,
    bytes: &[u8],
  ) -> Option<Self> {
    let chunk_id: u32 = T::read_u32(bytes.get(..4)?) & CHUNK_ID_MASK;
    let declared: u32 = T::read_u32(bytes.get(4..8)?);
    let payload: u64 = bytes.len() as u64 - CHUNK_HEADER_SIZE;

    if !Self::is_kinematics(chunk_id) || u64::from(declared) <= payload {
      return None;
    }

    let start: u64 = position + CHUNK_HEADER_SIZE;
    let clipped: ChunkReader<D> = ChunkReader {
      id: chunk_id,
      size: payload,
      position: start,
      data: reader.data.slice(start..start + payload),
    };

    if !OgfKinematicsChunk::read::<T, _>(&mut clipped.clone(), chunk_id)
      .ok()?
      .trailing
      .is_empty()
    {
      return None;
    }

    chunks.push(clipped);

    Some(Self {
      position,
      bytes: Vec::new(),
      cause: OgfResidueCause::OverstatedMotionRefs { declared },
    })
  }

  const fn is_kinematics(chunk_id: u32) -> bool {
    chunk_id == OgfKinematicsChunk::CHUNK_ID || chunk_id == OgfKinematicsChunk::CHUNK_ID_OLD
  }
}
