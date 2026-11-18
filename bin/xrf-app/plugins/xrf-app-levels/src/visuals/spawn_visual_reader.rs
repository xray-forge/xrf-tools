use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};

use xrf_app_core::TauriResult;
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_ogf::OgfFile;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayLogicalPath, XraySearch};
use xrf_visual::{
  VisualDependencies, VisualDescription, VisualMotions, VisualPackage, VisualPacker, VisualRestPose, VisualSkeleton,
  VisualSurfaces,
};

use crate::parts::LevelTextureReference;
use crate::report::LevelReport;
use crate::visuals::{SpawnVisual, SpawnVisuals};

/// Reads spawned objects' visuals within one search into the visuals they are kept in, opening the shader library once
/// for all of them, and only if one is read at all. A level and a scene each hand it the visuals their session keeps
/// and the search their game data is read through.
pub(crate) struct SpawnVisualReader {
  visuals: Arc<SpawnVisuals>,
  search: XraySearch,
  /// The level's own directory, the engine's `$level$`, where a level's breakables are compiled to; none for a scene,
  /// whose visuals are the game's alone.
  level: Option<XrayLogicalPath>,
  scope: XrayTextureScope,
  /// Shared with the pool a visual is read on, so held in a lock.
  resolver: OnceLock<XraySurfaceResolver>,
}

impl SpawnVisualReader {
  /// The cycle a lamp plays from the moment it spawns (`CHangingLamp::net_Spawn`), whose first frame it stands in.
  const IDLE_MOTION: &'static str = "idle";

  /// A reader of visuals into `visuals` through `search`, their textures resolved in `scope`, looked for in the shared
  /// meshes alone until [`Self::beside`] names a level's directory.
  pub(crate) fn new(visuals: Arc<SpawnVisuals>, search: XraySearch, scope: XrayTextureScope) -> Self {
    Self {
      visuals,
      search,
      level: None,
      scope,
      resolver: OnceLock::new(),
    }
  }

  /// The same reader looking for each visual beside `level` first, a level's own directory, as the engine does.
  pub(crate) fn beside(self, level: Option<XrayLogicalPath>) -> Self {
    Self { level, ..self }
  }

  /// A spawned object's visual, read and packed the first time anything asks for it and kept with the visuals.
  ///
  /// # Errors
  ///
  /// Returns why it cannot be read, reported the once it is tried: it is not read again.
  pub(crate) fn get(&self, name: &str) -> TauriResult<Arc<SpawnVisual>> {
    self.visuals.get_or_read(name, || {
      self
        .read(name)
        .map(Arc::new)
        .inspect_err(|error| LevelReport::undrawn_visual(name, error))
    })
  }

  /// Whether the visual `name` has a file where [`Self::get`] would read it, looked up without reading it.
  pub(crate) fn is_found(&self, name: &str) -> bool {
    self.find_asset(name).is_ok()
  }

  fn read(&self, name: &str) -> TauriResult<SpawnVisual> {
    let search: &XraySearch = &self.search;
    let file: Arc<OgfFile> = self
      .read_file(name)
      .map_err(|error| format!("Failed to read visual '{name}': {error}"))?;
    let package: VisualPackage = VisualPacker::pack(&file);
    let rest: Option<Arc<VisualRestPose>> =
      Self::find_rest_pose(search, name, &file, &package.description).map(Arc::new);
    let resolver: &XraySurfaceResolver = self
      .resolver
      .get_or_init(|| XraySurfaceResolver::open(search, self.scope.clone()));
    let surfaces: Vec<XraySurfaceDescriptor> = VisualSurfaces::describe(resolver, &package.description);
    let references: BTreeSet<String> = package
      .description
      .submeshes
      .iter()
      .filter_map(|submesh| submesh.texture_name.clone())
      .filter(|reference| !reference.is_empty())
      .collect();

    Ok(SpawnVisual {
      // Resolved as a level's own surfaces are, so a model shades with the bump pair and the detail its base declares.
      textures: LevelTextureReference::resolve_bound(references, &surfaces, search, &self.scope),
      package,
      rest,
      surfaces,
    })
  }

  /// The visual's file, parsed through the search, so a model another level, scene or the model viewer read is served
  /// as it was parsed.
  fn read_file(&self, name: &str) -> XrfResult<Arc<OgfFile>> {
    let asset: XrayAsset = self.find_asset(name)?;

    self
      .search
      .read_parsed(&asset, OgfFile::read_from_bytes::<XRayByteOrder>)
  }

  /// `CModelPool::Instance_Load`: beside the level first, then the shared meshes; a name no path beside the level can
  /// spell is looked up in the shared meshes alone, as a texture's is.
  fn find_asset(&self, name: &str) -> XrfResult<XrayAsset> {
    let beside: Option<XrayAsset> = self
      .level
      .as_ref()
      .and_then(|level| self.search.find_beside(level, XrayAssetType::Ogf, name).ok().flatten());

    match beside {
      Some(asset) => Ok(asset),
      None => self
        .search
        .resolve(XrayAssetType::Ogf, name)?
        .into_asset()
        .ok_or_else(|| XrfError::new_asset_error(format!("'{name}' resolves to no model in the mounted roots"))),
    }
  }

  /// `PlayCycle("idle")` then `CalculateBones`: the first frame of the visual's idle cycle where it has one, its bind
  /// pose where it does not.
  fn find_rest_pose(
    search: &XraySearch,
    name: &str,
    file: &OgfFile,
    description: &VisualDescription,
  ) -> Option<VisualRestPose> {
    let motions: VisualMotions = VisualMotions::of(file)?;
    let names: Vec<String> = motions.bones.iter().map(|bone| bone.name.clone()).collect();
    let dependencies: VisualDependencies = VisualDependencies::resolve(description, search);

    match motions.bake_named(search, &dependencies, Self::IDLE_MOTION) {
      Ok(baked) => VisualRestPose::of_floats(names, &baked.transforms).or_else(|| {
        LevelReport::empty_rest_motion(name, Self::IDLE_MOTION);

        VisualSkeleton::get_rest_pose(file)
      }),
      Err(error) => {
        LevelReport::bind_rest_pose(name, &error);

        VisualSkeleton::get_rest_pose(file)
      }
    }
  }
}
