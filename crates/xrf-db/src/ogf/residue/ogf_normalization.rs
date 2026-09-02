use byteorder::ByteOrder;
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_format_size;

use crate::ogf::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::ogf::ogf_file::OgfFile;
use crate::ogf::residue::ogf_residue::{CHUNK_HEADER_SIZE, OgfResidue};

/// A visual rewritten into bytes the engine loads identically, beside what the rewrite took out.
///
/// The residue travels with the bytes so a caller about to overwrite the source can name the discarded path first:
/// once the write lands, that report is the only record the path was ever there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfNormalization {
  /// Well-formed bytes, identical to the source for a visual carrying nothing the engine ignores.
  pub bytes: Vec<u8>,
  /// Root residue the source carried, `None` when it ended on a well-formed chunk.
  pub residue: Option<OgfResidue>,
}

impl OgfNormalization {
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
  pub fn normalize<T: ByteOrder>(original: &[u8]) -> XrfResult<Self> {
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
      return Ok(Self {
        bytes: original.to_vec(),
        residue,
      });
    }

    let end: usize = match &residue {
      Some(residue) => usize::try_from(residue.position)
        .map_err(|_| XrfError::new_invalid_error("Residue offset exceeds the supported range"))?,
      None => original.len(),
    };

    let Some((position, size, trailing_length)) = kinematics else {
      return Ok(Self {
        bytes: original[..end].to_vec(),
        residue,
      });
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

    let mut bytes: Vec<u8> = Vec::with_capacity(end);

    bytes.extend_from_slice(&original[..header_at + 4]);
    bytes.extend_from_slice(&size_field);
    bytes.extend_from_slice(&original[payload_at..payload_at + kept]);
    bytes.extend_from_slice(&original[payload_end..end]);

    Ok(Self { bytes, residue })
  }

  /// The normalized bytes alone; see [`Self::normalize`] for the residue they leave behind.
  ///
  /// # Errors
  ///
  /// As [`Self::normalize`].
  pub fn normalize_bytes<T: ByteOrder>(original: &[u8]) -> XrfResult<Vec<u8>> {
    Self::normalize::<T>(original).map(|normalization| normalization.bytes)
  }

  /// Whether the source carried anything the engine ignores.
  pub fn is_changed_from(&self, original: &[u8]) -> bool {
    self.bytes != original
  }

  /// Bytes the source carried that the engine never read, inside the motion refs chunk and after it.
  pub fn get_discarded_size(&self, original: &[u8]) -> usize {
    original.len().saturating_sub(self.bytes.len())
  }

  /// Guard that the normalized bytes still read as the visual `original` was, with nothing left over.
  ///
  /// Normalizing touches only the motion refs chunk and the tail, so the references the engine would load are the
  /// whole of what it could have changed; a difference there is a defect in the byte surgery, not in the source.
  ///
  /// # Errors
  ///
  /// Returns a verify error when the normalized bytes do not parse, still carry residue, or yield other motion refs.
  pub fn assert_engine_reads_the_same<T: ByteOrder>(&self, original: &[u8]) -> XrfResult {
    let before: OgfFile = OgfFile::read_from_bytes::<T>(original.to_vec())?;
    let after: OgfFile = OgfFile::read_from_bytes::<T>(self.bytes.clone())?;

    if after.residue.is_some() {
      return Err(XrfError::new_verify_error(
        "Normalized ogf bytes still carry residue the engine never reads",
      ));
    }

    let before_refs: Option<&Vec<String>> = before.kinematics.as_ref().map(|it| &it.motion_refs);
    let after_refs: Option<&Vec<String>> = after.kinematics.as_ref().map(|it| &it.motion_refs);

    if before_refs != after_refs {
      return Err(XrfError::new_verify_error(format!(
        "Normalized ogf bytes read back motion refs {after_refs:?} instead of {before_refs:?}"
      )));
    }

    Ok(())
  }
}
