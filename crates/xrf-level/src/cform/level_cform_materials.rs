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
  /// Every material id a face can name.
  const IDS: usize = LevelCformFace::MATERIAL_MASK as usize + 1;

  /// Reads the payload after a header, keeping only each face's material: a form runs to millions of faces, and
  /// nothing else of it is wanted.
  ///
  /// The vertices are stepped over and the faces read where they lie, each face marking its material in a table of every
  /// id a face can name: a level names a few dozen materials across millions of faces, so a mark per face is what keeps
  /// this from costing a set lookup per face.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload holds fewer vertices or faces than the header counts.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    header: &LevelCformHeader,
  ) -> XrfResult<Self> {
    reader.skip_bytes(header.vertex_count as usize * LevelCformGeometry::VERTEX_SIZE)?;

    let named: Box<[bool; Self::IDS]> =
      reader.read_with(header.face_count as usize * LevelCformFace::SERIALIZED_SIZE, |faces| {
        let mut named: Box<[bool; Self::IDS]> = Box::new([false; Self::IDS]);

        for face in faces.as_chunks::<{ LevelCformFace::SERIALIZED_SIZE }>().0 {
          named[usize::from(LevelCformFace::read_material::<T>(face))] = true;
        }

        named
      })?;

    let ids: BTreeSet<u16> = (0..=LevelCformFace::MATERIAL_MASK as u16)
      .filter(|&id| named[usize::from(id)])
      .collect();

    Ok(Self { ids })
  }

  /// Every material id some face names, ascending.
  pub fn get_ids(&self) -> &BTreeSet<u16> {
    &self.ids
  }
}
