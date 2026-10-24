use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::sync::Arc;

use xrf_app_core::assets::locate_referenced_asset;
use xrf_app_core::session::SessionSnapshot;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_ogf::OgfFile;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayLogicalPath, XraySearch};
use xrf_visual::{
  VisualDependencies, VisualDescription, VisualMotions, VisualPackage, VisualPacker, VisualRestPose, VisualSkeleton,
};

use crate::level_report::LevelReport;
use crate::parts::LevelSpawnVisualsPart;
use crate::state::{LevelSpawnVisual, LevelSpawnVisuals, LevelTextureReference, SelectedLevel};

/// Reads a level's spawned visuals within one search, opening the shader library once for all of them, and only if
/// one is read at all.
pub struct SpawnVisualReader {
  visuals: Arc<LevelSpawnVisuals>,
  search: XraySearch,
  /// The level's own directory, the engine's `$level$`, where the level's breakables are compiled to.
  level: Option<XrayLogicalPath>,
  scope: XrayTextureScope,
  resolver: OnceCell<XraySurfaceResolver>,
}

impl SpawnVisualReader {
  /// The cycle a lamp plays from the moment it spawns (`CHangingLamp::net_Spawn`), whose first frame it stands in.
  const IDLE_MOTION: &'static str = "idle";

  /// Runs `read` with a reader of the open level's spawned visuals, whatever it reads recorded as the visuals' part read
  /// it.
  pub fn with<T>(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    read: impl FnOnce(&SpawnVisualReader) -> T,
  ) -> T {
    let visuals: Arc<LevelSpawnVisuals> = current
      .parts
      .get_or_init::<LevelSpawnVisualsPart>(|| Arc::new(LevelSpawnVisuals::new()));

    current.parts.record::<LevelSpawnVisualsPart, T>(search, |search| {
      read(&SpawnVisualReader {
        visuals,
        search: search.clone(),
        level: current.source.get_logical_directory(),
        scope: current.source.get_texture_scope(),
        resolver: OnceCell::new(),
      })
    })
  }

  /// A spawned object's visual, read and packed the first time anything asks for it and kept with the level.
  ///
  /// # Errors
  ///
  /// Returns why it cannot be read, reported the once it is tried: it is not read again.
  pub fn get(&self, name: &str) -> Result<Arc<LevelSpawnVisual>, String> {
    self.visuals.get_or_read(name, || {
      self
        .read(name)
        .map(Arc::new)
        .inspect_err(|error| LevelReport::undrawn_visual(name, error))
    })
  }

  fn read(&self, name: &str) -> Result<LevelSpawnVisual, String> {
    let search: &XraySearch = &self.search;
    let file: Arc<OgfFile> = self
      .read_file(name)
      .map_err(|error| format!("Failed to read visual '{name}': {error}"))?;
    let package: VisualPackage = VisualPacker::pack(&file);
    let rest: Option<Arc<VisualRestPose>> = Self::to_rest_pose(search, name, &file, &package.description).map(Arc::new);
    let resolver: &XraySurfaceResolver = self
      .resolver
      .get_or_init(|| XraySurfaceResolver::open(search, self.scope.clone()));
    let surfaces: Vec<XraySurfaceDescriptor> = package
      .description
      .submeshes
      .iter()
      .map(|submesh| {
        let textures: Vec<String> = submesh.texture_name.clone().into_iter().collect();

        resolver.describe(submesh.shader_name.as_deref().unwrap_or_default(), &textures)
      })
      .collect();
    let references: BTreeSet<String> = package
      .description
      .submeshes
      .iter()
      .filter_map(|submesh| submesh.texture_name.clone())
      .filter(|reference| !reference.is_empty())
      .collect();

    Ok(LevelSpawnVisual {
      // Resolved as a level's own surfaces are, so a model shades with the bump pair and the detail its base declares.
      textures: LevelTextureReference::resolve_bound(references, &surfaces, search, &self.scope),
      package,
      rest,
      surfaces,
    })
  }

  /// `CModelPool::Instance_Load`: beside the level first, then the shared meshes; a name no path beside the level can
  /// spell is looked up in the shared meshes alone, as a texture's is. Parsed through the search, so a model another
  /// level or the model viewer read is served as it was parsed.
  fn read_file(&self, name: &str) -> XrfResult<Arc<OgfFile>> {
    let beside: Option<XrayAsset> = self
      .level
      .as_ref()
      .and_then(|level| self.search.find_beside(level, XrayAssetType::Ogf, name).ok().flatten());
    let asset: XrayAsset = match beside {
      Some(asset) => asset,
      None => locate_referenced_asset(&self.search, XrayAssetType::Ogf, name)?,
    };

    self
      .search
      .read_parsed(&asset, OgfFile::read_from_bytes::<XRayByteOrder>)
  }

  /// `PlayCycle("idle")` then `CalculateBones`: the first frame of the visual's idle cycle where it has one, its bind
  /// pose where it does not.
  fn to_rest_pose(
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
