use std::collections::BTreeSet;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;

use crate::cform::level_cform_face::LevelCformFace;
use crate::cform::level_cform_file::LevelCformHeader;
use crate::cform::level_cform_geometry::LevelCformGeometry;

/// The game materials a collision form's faces are made of, by their `gamemtl.xr` ids, which the engine resolves to
/// library entries on load (`CLevel::Load_GameSpecific_CFORM`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelCformMaterials {
  ids: BTreeSet<u16>,
}

impl LevelCformMaterials {
  /// Reads the payload after a header one face at a time, keeping only each face's material: a form runs to millions of
  /// faces, and nothing else of it is wanted.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload holds fewer vertices or faces than the header counts.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    header: &LevelCformHeader,
  ) -> XrfResult<Self> {
    reader.read_bytes(header.vertex_count as usize * LevelCformGeometry::VERTEX_SIZE)?;

    let mut ids: BTreeSet<u16> = BTreeSet::new();

    for _ in 0..header.face_count {
      ids.insert(reader.read_xr::<T, LevelCformFace>()?.material);
    }

    Ok(Self { ids })
  }

  /// Every material id some face names, ascending.
  pub fn get_ids(&self) -> &BTreeSet<u16> {
    &self.ids
  }
}
