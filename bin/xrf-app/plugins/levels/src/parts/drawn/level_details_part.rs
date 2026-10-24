use std::collections::{BTreeSet, HashSet};

use xrf_app_core::assets::read_located_asset;
use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_chunk::XRayByteOrder;
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelCformGeometry, LevelDetailsFile};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_vfs::XraySearch;
use xrf_visual::{DetailsPackage, DetailsPacker};

use crate::parts::{LevelCollisionPart, PackedLevelDetails};
use crate::report::report_unreadable_materials;
use crate::state::{DETAILS_FILE, LevelSource, LevelTextureReference, SelectedLevel};

/// What a change event calls what a level's grass was read from when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_DETAILS_PART: &str = "details";

/// What a level's grass was read from: `level.details`, the collision form it is planted onto, the game materials and
/// the shaders and textures that dress it; the renderer holds the grass itself.
pub(crate) struct LevelDetailsPart;

impl SessionPart for LevelDetailsPart {
  type Value = ();

  const NAME: &'static str = LEVEL_DETAILS_PART;

  const IS_POOLED: bool = false;
}

impl LevelDetailsPart {
  /// The game material library, whose flags say which materials a planting falls through.
  const GAME_MATERIALS_FILE: &'static str = "gamemtl.xr";

  /// Packs the open level's grass, what it is read from recorded as its details part, or answers `None` for a level
  /// with no detail library.
  ///
  /// # Errors
  ///
  /// Returns why the detail library or the collision form it is planted onto cannot be read.
  pub fn pack(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
  ) -> TauriResult<Option<PackedLevelDetails>> {
    current
      .parts
      .record_read::<Self, _>(search, |search| Self::pack_recorded(&current.source, search))
  }

  /// The grass packed, every read through `search`.
  fn pack_recorded(source: &LevelSource, search: &XraySearch) -> TauriResult<Option<PackedLevelDetails>> {
    let Some(details) = source.read_optional_file(search, DETAILS_FILE)? else {
      return Ok(None);
    };

    let details: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(details)
      .map_err(|error| source.describe_read_failure(DETAILS_FILE, &error))?;
    let collision: LevelCformGeometry = LevelCollisionPart::read_geometry(source, search)?;
    let passable: HashSet<u16> = Self::read_passable_materials(search);
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
        .map(|reference| LevelTextureReference::locate(&scope, search, reference.to_owned()))
        .collect(),
    }))
  }

  /// The ids of the game materials a planting falls through, none where the library cannot be read: the engine would
  /// have refused to load the level at all without one.
  fn read_passable_materials(search: &XraySearch) -> HashSet<u16> {
    let materials: GameMtlFile = match read_located_asset(search, Self::GAME_MATERIALS_FILE)
      .and_then(GameMtlFile::read_from_bytes::<XRayByteOrder>)
    {
      Ok(materials) => materials,
      Err(error) => {
        report_unreadable_materials(Self::GAME_MATERIALS_FILE, &error);

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
}
