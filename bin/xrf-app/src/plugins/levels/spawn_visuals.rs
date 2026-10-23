//! The visuals the game's spawned objects stand as, packed once a level and shared by whatever reads them: the models
//! drawn, and the lights placed on their bones.

use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::sync::Arc;

use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_ogf::OgfFile;
use xrf_vfs::{XrayAssetType, XrayLogicalPath, XrayProbe};
use xrf_visual::{VisualDependencies, VisualDescription, VisualPackage, VisualPacker, VisualRestPose, VisualSkeleton};

use crate::core::assets::read_referenced_asset;
use crate::core::session::{SessionPart, SessionSnapshot};
use crate::plugins::levels::report::{report_bind_rest_pose, report_empty_rest_motion, report_undrawn_visual};
use crate::plugins::levels::state::{LevelSpawnVisual, LevelSpawnVisuals, SelectedLevel};
use crate::plugins::levels::textures::resolve_surface_textures;
use crate::plugins::visuals::pose::bake_named_motion;
use crate::plugins::visuals::skeleton::SelectedSkeleton;

/// The cycle a lamp plays from the moment it spawns (`CHangingLamp::net_Spawn`), whose first frame it stands in.
const IDLE_MOTION: &str = "idle";

/// Each visual a level's spawned objects stand as, read the first time anything asks and kept, or why it cannot be.
struct LevelSpawnVisualsPart;

impl SessionPart for LevelSpawnVisualsPart {
  type Value = Arc<LevelSpawnVisuals>;

  // Built empty, its visuals each read into a cell of its own later, so its build never reaches Rayon.
  const IS_POOLED: bool = false;
}

/// Reads a level's spawned visuals within one probe, opening the shader library once for all of them, and only if
/// one is read at all.
pub struct SpawnVisualReader<'probe, 'vfs> {
  visuals: Arc<LevelSpawnVisuals>,
  probe: &'probe XrayProbe<'vfs>,
  /// The level's own directory, the engine's `$level$`, where the level's breakables are compiled to.
  level: Option<XrayLogicalPath>,
  scope: XrayTextureScope,
  resolver: OnceCell<XraySurfaceResolver<'probe, 'vfs>>,
}

impl<'probe, 'vfs> SpawnVisualReader<'probe, 'vfs> {
  /// Runs `read` with a reader of the open level's spawned visuals, whatever it reads recorded as the visuals' part read
  /// it.
  pub fn with<T>(
    current: &SessionSnapshot<SelectedLevel>,
    probe: &XrayProbe,
    read: impl FnOnce(&SpawnVisualReader) -> T,
  ) -> T {
    let visuals: Arc<LevelSpawnVisuals> = current
      .parts
      .get_or_init::<LevelSpawnVisualsPart>(|| Arc::new(LevelSpawnVisuals::new()));

    current.parts.record::<LevelSpawnVisualsPart, T>(probe, |probe| {
      read(&SpawnVisualReader {
        visuals,
        probe,
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
        .inspect_err(|error| report_undrawn_visual(name, error))
    })
  }

  fn read(&self, name: &str) -> Result<LevelSpawnVisual, String> {
    let probe: &XrayProbe = self.probe;
    let file: OgfFile = self
      .read_bytes(name)
      .and_then(OgfFile::read_from_bytes::<XRayByteOrder>)
      .map_err(|error| format!("Failed to read visual '{name}': {error}"))?;
    let package: VisualPackage = VisualPacker::pack(&file);
    let rest: Option<Arc<VisualRestPose>> = to_rest_pose(probe, name, &file, &package.description).map(Arc::new);
    let resolver: &XraySurfaceResolver = self
      .resolver
      .get_or_init(|| XraySurfaceResolver::open(probe, self.scope.clone()));
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
      textures: resolve_surface_textures(references, &surfaces, probe, &self.scope),
      package,
      rest,
      surfaces,
    })
  }

  /// `CModelPool::Instance_Load`: beside the level first, then the shared meshes; a name no path beside the level can
  /// spell is looked up in the shared meshes alone, as a texture's is.
  fn read_bytes(&self, name: &str) -> XrfResult<Vec<u8>> {
    if let Some(level) = &self.level
      && let Ok(beside) = self.probe.find_beside(level, XrayAssetType::Ogf, name)
      && let Some(asset) = beside.get_asset()
    {
      return self.probe.read_asset_bytes(asset);
    }

    read_referenced_asset(self.probe, XrayAssetType::Ogf, name)
  }
}

/// `PlayCycle("idle")` then `CalculateBones`: the first frame of the visual's idle cycle where it has one, its bind
/// pose where it does not.
fn to_rest_pose(
  probe: &XrayProbe,
  name: &str,
  file: &OgfFile,
  description: &VisualDescription,
) -> Option<VisualRestPose> {
  let skeleton: SelectedSkeleton = SelectedSkeleton::of(file)?;
  let names: Vec<String> = skeleton.bones.iter().map(|bone| bone.name.clone()).collect();
  let dependencies: VisualDependencies = VisualDependencies::resolve(description, probe);

  match bake_named_motion(probe, &skeleton, &dependencies, IDLE_MOTION) {
    Ok(baked) => VisualRestPose::of_floats(names, &baked.transforms).or_else(|| {
      report_empty_rest_motion(name, IDLE_MOTION);

      VisualSkeleton::get_rest_pose(file)
    }),
    Err(error) => {
      report_bind_rest_pose(name, &error);

      VisualSkeleton::get_rest_pose(file)
    }
  }
}
