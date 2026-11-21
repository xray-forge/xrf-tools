use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};

use xrf_app_core::TauriResult;
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

/// Reads spawned objects' visuals through one search into a session's [`SpawnVisuals`], opening the shader library
/// once, and only when a visual is read.
pub(crate) struct SpawnVisualReader {
  visuals: Arc<SpawnVisuals>,
  search: XraySearch,
  /// The level's own directory, the engine's `$level$`, holding its compiled breakables; none for a scene.
  level: Option<XrayLogicalPath>,
  scope: XrayTextureScope,
  /// Opened on the first read, which may run on a pool worker.
  resolver: OnceLock<XraySurfaceResolver>,
}

impl SpawnVisualReader {
  /// The cycle a lamp plays from spawn (`CHangingLamp::net_Spawn`); its first frame is the rest pose.
  const IDLE_MOTION: &'static str = "idle";

  /// A reader into `visuals`, resolving textures in `scope` and finding visuals in the shared meshes alone until
  /// [`Self::beside`] names a level's directory.
  pub(crate) fn new(visuals: Arc<SpawnVisuals>, search: XraySearch, scope: XrayTextureScope) -> Self {
    Self {
      visuals,
      search,
      level: None,
      scope,
      resolver: OnceLock::new(),
    }
  }

  /// The same reader looking in the level directory `level` first, as the engine does.
  pub(crate) fn beside(self, level: Option<XrayLogicalPath>) -> Self {
    Self { level, ..self }
  }

  /// A spawned object's visual, read and packed on first ask and kept.
  ///
  /// # Errors
  ///
  /// Returns why it cannot be read; the failure is reported once and kept, not retried.
  pub(crate) fn get(&self, name: &str) -> TauriResult<Arc<SpawnVisual>> {
    self.visuals.get_or_read(name, || {
      self
        .read(name)
        .map(Arc::new)
        .inspect_err(|error| LevelReport::undrawn_visual(name, error))
    })
  }

  /// Whether [`Self::get`] would find a file for `name`, without reading it.
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
      // Resolved as level surfaces are, so a model shades with its base's bump pair and detail.
      textures: LevelTextureReference::resolve_bound(references, &surfaces, search, &self.scope),
      package,
      rest,
      surfaces,
    })
  }

  /// The visual's file, parsed through the search's cache, shared with other levels, scenes and the model viewer.
  fn read_file(&self, name: &str) -> XrfResult<Arc<OgfFile>> {
    let asset: XrayAsset = self.find_asset(name)?;

    self.search.read_parsed(&asset, OgfFile::read_from_bytes)
  }

  /// `CModelPool::Instance_Load`: the level directory first, then the shared meshes; a name no level path can spell
  /// goes to the shared meshes alone, as a texture's does.
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

  /// `PlayCycle("idle")` then `CalculateBones`: the idle cycle's first frame, else the bind pose.
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
