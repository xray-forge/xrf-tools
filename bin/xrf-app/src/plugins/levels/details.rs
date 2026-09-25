//! A level's grass, packed from its detail library and its collision form.

use std::collections::{BTreeSet, HashSet};

use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelCformFile, LevelCformGeometry, LevelDetailsFile};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver};
use xrf_spawn::XRayByteOrder;
use xrf_vfs::XrayProbe;
use xrf_visual::{DetailsPackage, DetailsPacker};

use crate::core::types::TauriResult;
use crate::plugins::levels::read::{read_asset, read_file};
use crate::plugins::levels::state::{COLLISION_FILE, DETAILS_FILE, LevelSource, LevelTextureReference};
use crate::plugins::levels::textures::resolve_reference;

/// The game material library, whose flags say which materials a planting falls through.
const GAME_MATERIALS_FILE: &str = "gamemtl.xr";

/// `SGameMtl::flPassable`.
const PASSABLE_FLAG: u32 = 1 << 7;

/// A level's grass, packed and dressed.
pub struct PackedLevelDetails {
  pub package: DetailsPackage,
  pub surfaces: Vec<XraySurfaceDescriptor>,
  pub textures: Vec<LevelTextureReference>,
}

/// Packs a level's grass, or answers `None` for a level with no detail library.
pub fn pack_details(
  source: &LevelSource,
  probe: &XrayProbe,
  directory: Option<&str>,
) -> TauriResult<Option<PackedLevelDetails>> {
  let Ok(details) = read_file(source, probe, DETAILS_FILE) else {
    return Ok(None);
  };

  let details: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(details).map_err(|error| {
    format!(
      "Failed to read '{DETAILS_FILE}' of level '{}': {error}",
      source.get_label()
    )
  })?;
  let (_, collision): (LevelCformFile, LevelCformGeometry) = LevelCformFile::read_with_geometry_from_bytes::<
    XRayByteOrder,
  >(read_file(source, probe, COLLISION_FILE)?)
  .map_err(|error| {
    format!(
      "Failed to read '{COLLISION_FILE}' of level '{}': {error}",
      source.get_label()
    )
  })?;
  let passable: HashSet<u16> = read_passable_materials(probe);
  let is_passable = |material: u16| passable.contains(&material);
  let package: DetailsPackage = DetailsPacker::new(&details, &collision, &is_passable).pack();

  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(probe);
  let surfaces: Vec<XraySurfaceDescriptor> = details
    .objects
    .iter()
    .map(|model| resolver.describe(&model.shader, std::slice::from_ref(&model.texture)))
    .collect();
  let references: BTreeSet<&str> = details
    .objects
    .iter()
    .map(|model| model.texture.as_str())
    .filter(|texture| !texture.is_empty())
    .collect();

  Ok(Some(PackedLevelDetails {
    package,
    surfaces,
    textures: references
      .into_iter()
      .map(|reference| LevelTextureReference {
        logical_path: resolve_reference(probe, directory, reference),
        reference: reference.to_owned(),
      })
      .collect(),
  }))
}

/// The ids of the game materials a planting falls through, none where the library cannot be read: the engine would
/// have refused to load the level at all without one.
fn read_passable_materials(probe: &XrayProbe) -> HashSet<u16> {
  let materials: Option<GameMtlFile> = read_asset(probe, GAME_MATERIALS_FILE)
    .ok()
    .and_then(|bytes| GameMtlFile::read_from_bytes::<XRayByteOrder>(bytes).ok());

  let Some(materials) = materials else {
    log::warn!("No readable '{GAME_MATERIALS_FILE}', so every collision triangle is taken as solid ground");

    return HashSet::new();
  };

  materials
    .materials
    .iter()
    .filter(|material| material.flags & PASSABLE_FLAG != 0)
    .filter_map(|material| u16::try_from(material.id).ok())
    .collect()
}
