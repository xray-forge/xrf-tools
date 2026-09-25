//! The visuals the game's spawned objects stand as, packed once a level and shared by whatever reads them: the models
//! drawn, and the lights placed on their bones.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, MutexGuard};

use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver};
use xrf_ogf::OgfFile;
use xrf_spawn::{AlifeObject, AlifeObjectInherited, XRayByteOrder};
use xrf_vfs::XrayProbe;
use xrf_visual::{VisualDependencies, VisualDescription, VisualPackage, VisualPacker, VisualRestPose};

use crate::plugins::levels::read::read_asset;
use crate::plugins::levels::state::{LevelSpawnVisual, LevelTextureReference, SelectedLevel};
use crate::plugins::levels::textures::resolve_reference;
use crate::plugins::visuals::pose::bake_named_motion;
use crate::plugins::visuals::skeleton::SelectedSkeleton;

/// `CSE_ALifeObjectHangingLamp::flR2`: a lamp the engine spawns on R2 and later, and so draws.
const LAMP_R2_FLAG: u16 = 1 << 3;

/// The cycle a lamp plays from the moment it spawns (`CHangingLamp::net_Spawn`), whose first frame it stands in.
const IDLE_MOTION: &str = "idle";

/// The visual a spawned object is drawn as, or `None` for one the viewer draws no model of yet: a hanging lamp the
/// engine spawns today.
pub fn get_drawn_visual(object: &AlifeObject) -> Option<&str> {
  match &object.inherited {
    AlifeObjectInherited::CseAlifeObjectHangingLamp(lamp)
      if lamp.light_flags & LAMP_R2_FLAG != 0 && !lamp.base.visual_name.is_empty() =>
    {
      Some(lamp.base.visual_name.as_str())
    }
    _ => None,
  }
}

/// A spawned object's visual, read and packed the first time anything asks for it and kept with the level; `None` for
/// one that cannot be read, which is not asked again.
pub fn get_spawn_visual(current: &SelectedLevel, probe: &XrayProbe, name: &str) -> Option<Arc<LevelSpawnVisual>> {
  let mut visuals: MutexGuard<HashMap<String, Option<Arc<LevelSpawnVisual>>>> = current.spawn_visuals.lock().ok()?;

  if let Some(visual) = visuals.get(name) {
    return visual.clone();
  }

  let directory: Option<String> = current.source.get_logical_directory();
  let visual: Option<Arc<LevelSpawnVisual>> = match read_spawn_visual(probe, directory.as_deref(), name) {
    Ok(visual) => Some(Arc::new(visual)),
    Err(error) => {
      log::warn!("Spawned visual '{name}' is not drawn: {error}");

      None
    }
  };

  visuals.insert(name.to_owned(), visual.clone());

  visual
}

fn read_spawn_visual(probe: &XrayProbe, directory: Option<&str>, name: &str) -> Result<LevelSpawnVisual, String> {
  let path: String = format!("meshes\\{}.ogf", name.trim_end_matches(".ogf"));
  let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(read_asset(probe, &path)?)
    .map_err(|error| format!("Failed to read '{path}': {error}"))?;
  let package: VisualPackage = VisualPacker::pack(&file);
  let rest: Option<Arc<VisualRestPose>> = to_rest_pose(probe, &file, &package.description).map(Arc::new);
  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(probe);
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
        logical_path: resolve_reference(probe, directory, reference),
        reference: reference.to_owned(),
      })
      .collect(),
    package,
    rest,
    surfaces,
  })
}

/// `PlayCycle("idle")` then `CalculateBones`: the first frame of the visual's idle cycle where it has one, its bind
/// pose where it does not.
fn to_rest_pose(probe: &XrayProbe, file: &OgfFile, description: &VisualDescription) -> Option<VisualRestPose> {
  let skeleton: SelectedSkeleton = SelectedSkeleton::of(file)?;
  let names: Vec<String> = skeleton.bones.iter().map(|bone| bone.name.clone()).collect();
  let dependencies: VisualDependencies = VisualDependencies::resolve(description, probe);

  match bake_named_motion(probe, &skeleton, &dependencies, IDLE_MOTION) {
    Ok(baked) => VisualRestPose::of_floats(names, &baked.transforms).or_else(|| VisualRestPose::of_bind(file)),
    Err(_) => VisualRestPose::of_bind(file),
  }
}
