use serde::Serialize;
use xrf_error::XrfResult;
use xrf_gamemtl::GameMtlFile;
use xrf_spawn::XRayByteOrder;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::gamemtl::archive_game_mtl_material::ArchiveGameMtlMaterial;
use crate::plugins::archives::describe::gamemtl::archive_game_mtl_property::ArchiveGameMtlProperty;

/// Everything the viewer says about the game material library.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveGameMtlDescription {
  pub version: u16,
  pub materials: Vec<ArchiveGameMtlMaterial>,
  /// Pairings of two materials, which decide what is heard and seen where they meet.
  pub pairs: usize,
  /// Pairings that declare nothing of their own and take everything from the pairing they name as parent.
  pub inheriting_pairs: usize,
  /// What the pairings declare, counted per property rather than listed.
  pub properties: Vec<ArchiveGameMtlProperty>,
}

impl ArchiveGameMtlDescription {
  /// Reads the material library an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a material library this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      version: file.version,
      pairs: file.pairs.len(),
      inheriting_pairs: file.get_inheriting_pairs_count(),
      properties: ArchiveGameMtlProperty::of_all(&file.pairs),
      materials: ArchiveGameMtlMaterial::of_all(&file.materials),
    })
  }
}
