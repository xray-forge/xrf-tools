//! A level's grass, packed from its detail library and its collision form.

use std::collections::{BTreeSet, HashSet};

use xrf_chunk::XRayByteOrder;
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelCformFile, LevelCformGeometry, LevelDetailsFile};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_vfs::XraySearch;
use xrf_visual::{DetailsPackage, DetailsPacker};

use crate::core::assets::read_located_asset;
use crate::core::session::{SessionPart, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::read::{read_file, read_optional_file};
use crate::plugins::levels::report::report_unreadable_materials;
use crate::plugins::levels::state::{COLLISION_FILE, DETAILS_FILE, LevelSource, LevelTextureReference, SelectedLevel};

/// The game material library, whose flags say which materials a planting falls through.
const GAME_MATERIALS_FILE: &str = "gamemtl.xr";

/// What a change event calls what a level's grass was read from when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_DETAILS_PART: &str = "details";

/// What a level's grass was read from: `level.details`, the collision form it is planted onto, the game materials and
/// the shaders and textures that dress it; the renderer holds the grass itself.
struct LevelDetailsPart;

impl SessionPart for LevelDetailsPart {
  type Value = ();

  const NAME: &'static str = LEVEL_DETAILS_PART;

  const IS_POOLED: bool = false;
}

/// A level's grass, packed and dressed.
pub struct PackedLevelDetails {
  pub package: DetailsPackage,
  pub surfaces: Vec<XraySurfaceDescriptor>,
  pub textures: Vec<LevelTextureReference>,
}

/// Packs the open level's grass, what it is read from recorded as its details part, or answers `None` for a level with
/// no detail library.
pub fn pack_details(
  current: &SessionSnapshot<SelectedLevel>,
  search: &XraySearch,
) -> TauriResult<Option<PackedLevelDetails>> {
  current
    .parts
    .record_read::<LevelDetailsPart, _>(search, |search| pack_recorded(&current.source, search))
}

/// The grass packed, every read through `search`.
fn pack_recorded(source: &LevelSource, search: &XraySearch) -> TauriResult<Option<PackedLevelDetails>> {
  let Some(details) = read_optional_file(source, search, DETAILS_FILE)? else {
    return Ok(None);
  };

  let details: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(details).map_err(|error| {
    format!(
      "Failed to read '{DETAILS_FILE}' of level '{}': {error}",
      source.get_label()
    )
  })?;
  let collision: LevelCformGeometry =
    LevelCformFile::read_geometry_from_bytes::<XRayByteOrder>(read_file(source, search, COLLISION_FILE)?).map_err(
      |error| {
        format!(
          "Failed to read '{COLLISION_FILE}' of level '{}': {error}",
          source.get_label()
        )
      },
    )?;
  let passable: HashSet<u16> = read_passable_materials(search);
  let is_passable = |material: u16| passable.contains(&material);
  let package: DetailsPackage = DetailsPacker::new(&details, &collision, &is_passable).pack();

  let scope: XrayTextureScope = source.get_texture_scope();
  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(search, scope.clone());
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
        logical_path: scope.locate_texture(search, reference),
        reference: reference.to_owned(),
      })
      .collect(),
  }))
}

/// The ids of the game materials a planting falls through, none where the library cannot be read: the engine would
/// have refused to load the level at all without one.
fn read_passable_materials(search: &XraySearch) -> HashSet<u16> {
  let materials: GameMtlFile =
    match read_located_asset(search, GAME_MATERIALS_FILE).and_then(GameMtlFile::read_from_bytes::<XRayByteOrder>) {
      Ok(materials) => materials,
      Err(error) => {
        report_unreadable_materials(GAME_MATERIALS_FILE, &error);

        return HashSet::new();
      }
    };

  materials
    .materials
    .iter()
    .filter(|material| material.is_passable())
    .filter_map(|material| u16::try_from(material.id).ok())
    .collect()
}
