//! The visuals the game's spawned objects stand as, packed once a level and shared by whatever reads them: the models
//! drawn, and the lights placed on their bones.

use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::sync::Arc;

use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver};
use xrf_ogf::OgfFile;
use xrf_spawn::{AlifeObject, AlifeObjectInherited, XRayByteOrder};
use xrf_vfs::{XrayAssetType, XrayLogicalPath, XrayProbe};
use xrf_visual::{VisualDependencies, VisualDescription, VisualPackage, VisualPacker, VisualRestPose};

use crate::core::assets::read_referenced_asset;
use crate::plugins::levels::report::{report_bind_rest_pose, report_empty_rest_motion, report_undrawn_visual};
use crate::plugins::levels::state::{LevelSpawnVisual, LevelTextureReference, SelectedLevel};
use crate::plugins::levels::textures::resolve_reference;
use crate::plugins::visuals::pose::bake_named_motion;
use crate::plugins::visuals::skeleton::SelectedSkeleton;

/// The cycle a lamp plays from the moment it spawns (`CHangingLamp::net_Spawn`), whose first frame it stands in.
const IDLE_MOTION: &str = "idle";

/// The visual a spawned object is drawn as, or `None` for one the viewer draws no model of yet: only a hanging lamp
/// the engine spawns on R2 is drawn today.
pub fn get_drawn_visual(object: &AlifeObject) -> Option<&str> {
  match &object.inherited {
    AlifeObjectInherited::CseAlifeObjectHangingLamp(lamp) if lamp.is_spawned_on_r2() => object.inherited.get_visual(),
    _ => None,
  }
}

/// Reads a level's spawned visuals within one probe, opening the shader library once for all of them, and only if
/// one is read at all.
pub struct SpawnVisualReader<'probe, 'vfs> {
  current: &'probe SelectedLevel,
  probe: &'probe XrayProbe<'vfs>,
  directory: Option<XrayLogicalPath>,
  resolver: OnceCell<XraySurfaceResolver<'probe, 'vfs>>,
}

impl<'probe, 'vfs> SpawnVisualReader<'probe, 'vfs> {
  pub fn new(current: &'probe SelectedLevel, probe: &'probe XrayProbe<'vfs>) -> Self {
    Self {
      current,
      probe,
      directory: current.source.get_logical_directory(),
      resolver: OnceCell::new(),
    }
  }

  /// A spawned object's visual, read and packed the first time anything asks for it and kept with the level; `None`
  /// for one that cannot be read, which is not asked again.
  pub fn get(&self, name: &str) -> Option<Arc<LevelSpawnVisual>> {
    self
      .current
      .spawn_visuals
      .get_or_read(name, || match self.read(name) {
        Ok(visual) => Some(Arc::new(visual)),
        Err(error) => {
          report_undrawn_visual(name, &error);

          None
        }
      })
      .inspect_err(|error| report_undrawn_visual(name, error))
      .ok()
      .flatten()
  }

  fn read(&self, name: &str) -> Result<LevelSpawnVisual, String> {
    let probe: &XrayProbe = self.probe;
    let file: OgfFile = read_referenced_asset(probe, XrayAssetType::Ogf, name)
      .and_then(OgfFile::read_from_bytes::<XRayByteOrder>)
      .map_err(|error| format!("Failed to read visual '{name}': {error}"))?;
    let package: VisualPackage = VisualPacker::pack(&file);
    let rest: Option<Arc<VisualRestPose>> = to_rest_pose(probe, name, &file, &package.description).map(Arc::new);
    let resolver: &XraySurfaceResolver = self.resolver.get_or_init(|| XraySurfaceResolver::open(probe));
    let surfaces: Vec<XraySurfaceDescriptor> = package
      .description
      .submeshes
      .iter()
      .map(|submesh| {
        let textures: Vec<String> = submesh.texture_name.clone().into_iter().collect();

        resolver.describe(submesh.shader_name.as_deref().unwrap_or_default(), &textures)
      })
      .collect();
    let references: BTreeSet<&str> = package
      .description
      .submeshes
      .iter()
      .filter_map(|submesh| submesh.texture_name.as_deref())
      .filter(|reference| !reference.is_empty())
      .collect();

    Ok(LevelSpawnVisual {
      textures: references
        .into_iter()
        .map(|reference| LevelTextureReference {
          logical_path: resolve_reference(probe, self.directory.as_ref(), reference),
          reference: reference.to_owned(),
        })
        .collect(),
      package,
      rest,
      surfaces,
    })
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

      VisualRestPose::of_bind(file)
    }),
    Err(error) => {
      report_bind_rest_pose(name, &error);

      VisualRestPose::of_bind(file)
    }
  }
}
