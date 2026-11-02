use byteorder::ByteOrder;
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};

use crate::ogf_file::OgfFile;
use crate::patch::ogf_raw_patch::OgfRawPatch;
use crate::residue::ogf_residue::OgfResidue;

/// A visual rewritten into bytes the engine loads identically, beside what the rewrite took out.
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
  /// # Errors
  ///
  /// Returns the strict walk's error for a visual whose departures cannot be accounted for.
  pub fn normalize<T: ByteOrder>(original: &[u8]) -> XrfResult<Self> {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(original)?;
    let (chunks, residue) = OgfResidue::read_root_chunks::<T, _>(&mut reader)?;
    let mut bytes: Vec<u8> = Vec::with_capacity(original.len());

    for chunk in &chunks {
      let payload: Vec<u8> = match OgfRawPatch::is_kinematics_chunk(chunk.id) {
        true => OgfRawPatch::read_normalized_kinematics_payload::<T, _>(chunk)?,
        false => OgfRawPatch::read_payload(chunk)?,
      };

      OgfRawPatch::write_chunk::<T>(&mut bytes, chunk.id, payload)?;
    }

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
